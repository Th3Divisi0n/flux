//! Real window rendering for FLUX's `FXwindows` library.
//!
//! This crate knows nothing about the FLUX language or interpreter — it
//! takes a plain description of a window and its widgets ([`WindowSpec`])
//! and opens a real OS window for it using `eframe` (winit + egui). Keeping
//! it decoupled like this means `flux-interpreter` just has to build a
//! `WindowSpec` from its own `Value`/`ObjectValue` types and hand it off.
//!
//! `show_window` is blocking: it runs the window's event loop and only
//! returns once the user closes the window, which matches how
//! `FXwindows.window.show()` behaves in FLUX code today.

use eframe::egui;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::JoinHandle;

/// A single UI element inside a window. Fields mirror the FXwindows
/// constructors documented in `libraries/FXwindows/README.md`.
#[derive(Debug, Clone)]
pub enum Widget {
    Button { text: String, width: f32, height: f32 },
    Label { text: String, x: f32, y: f32 },
    TextBox { placeholder: String, x: f32, y: f32 },
    /// Rendering an actual image from disk is left as follow-up work (it
    /// needs egui's image-loader plumbing wired up); for now this draws a
    /// labeled placeholder box so the layout is still visible.
    Image { path: String, x: f32, y: f32 },
    Panel { x: f32, y: f32, width: f32, height: f32 },
    Slider { min: f64, max: f64, value: f64, x: f32, y: f32 },
    Checkbox { label: String, checked: bool, x: f32, y: f32 },
}

/// Everything needed to draw one FXwindows window.
#[derive(Debug, Clone)]
pub struct WindowSpec {
    pub title: String,
    pub width: f32,
    pub height: f32,
    pub widgets: Vec<Widget>,
}

/// Opens a real, interactive OS window for `spec` and blocks until it's
/// closed. Sliders/checkboxes/text boxes are genuinely interactive while
/// the window is open, but (for now) that interaction doesn't write back
/// into the FLUX variables that created them — see the crate docs.
pub fn show_window(spec: WindowSpec) -> Result<(), String> {
    let width = spec.width.max(200.0);
    let height = spec.height.max(150.0);
    let title = spec.title.clone();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([width, height]),
        ..Default::default()
    };

    eframe::run_native(
        &title,
        options,
        Box::new(|_creation_context| Ok(Box::new(FxApp::new(spec)))),
    )
    .map_err(|e| e.to_string())
}

struct FxApp {
    spec: WindowSpec,
    // Local interactive state for widgets that egui needs a `&mut` into
    // (sliders, checkboxes, text boxes). Seeded from the widget's initial
    // value; changes stay local to the window for now.
    checkbox_state: Vec<bool>,
    slider_state: Vec<f64>,
    textbox_state: Vec<String>,
}

impl FxApp {
    fn new(spec: WindowSpec) -> Self {
        let checkbox_state = spec
            .widgets
            .iter()
            .map(|w| matches!(w, Widget::Checkbox { checked: true, .. }))
            .collect();
        let slider_state = spec
            .widgets
            .iter()
            .map(|w| match w {
                Widget::Slider { value, .. } => *value,
                _ => 0.0,
            })
            .collect();
        let textbox_state = spec.widgets.iter().map(|_| String::new()).collect();

        Self {
            spec,
            checkbox_state,
            slider_state,
            textbox_state,
        }
    }
}

impl eframe::App for FxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Buttons don't carry an (x, y) in FXwindows today, so they're
            // stacked top-to-bottom in the order they were added.
            let mut next_button_y = 10.0;

            for (i, widget) in self.spec.widgets.iter().enumerate() {
                match widget {
                    Widget::Button { text, width, height } => {
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(10.0, next_button_y),
                            egui::vec2(*width, *height),
                        );
                        ui.put(rect, egui::Button::new(text));
                        next_button_y += height + 10.0;
                    }
                    Widget::Label { text, x, y } => {
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(*x, *y),
                            egui::vec2(220.0, 20.0),
                        );
                        ui.put(rect, egui::Label::new(text));
                    }
                    Widget::TextBox { placeholder, x, y } => {
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(*x, *y),
                            egui::vec2(180.0, 24.0),
                        );
                        ui.put(
                            rect,
                            egui::TextEdit::singleline(&mut self.textbox_state[i])
                                .hint_text(placeholder),
                        );
                    }
                    Widget::Image { path, x, y } => {
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(*x, *y),
                            egui::vec2(120.0, 90.0),
                        );
                        ui.put(
                            rect,
                            egui::Label::new(format!("[image: {path}]")).wrap(),
                        );
                        ui.painter().rect_stroke(
                            rect,
                            2.0,
                            egui::Stroke::new(1.0, egui::Color32::GRAY),
                        );
                    }
                    Widget::Panel { x, y, width, height } => {
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(*x, *y),
                            egui::vec2(*width, *height),
                        );
                        ui.painter().rect_stroke(
                            rect,
                            4.0,
                            egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
                        );
                    }
                    Widget::Slider { min, max, x, y, .. } => {
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(*x, *y),
                            egui::vec2(180.0, 24.0),
                        );
                        ui.put(
                            rect,
                            egui::Slider::new(&mut self.slider_state[i], *min..=*max),
                        );
                    }
                    Widget::Checkbox { label, x, y, .. } => {
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(*x, *y),
                            egui::vec2(200.0, 24.0),
                        );
                        ui.put(
                            rect,
                            egui::Checkbox::new(&mut self.checkbox_state[i], label),
                        );
                    }
                }
            }
        });
    }
}

// ---------------------------------------------------------------------
// FXterminal: real terminal-styled console windows.
// ---------------------------------------------------------------------
//
// Unlike `show_window` above, a console needs to stay open *while* the
// rest of the FLUX program keeps running and feeding it new lines — a
// script does `console.show()` once and then calls `console.print(...)`
// many times as it goes. So `open_console` can't block the calling
// thread the way `show_window` does: it spawns the window's event loop
// on a background thread and returns immediately, handing back a cheap,
// cloneable [`ConsoleHandle`] that `flux-interpreter` can capture in
// every native method on the `Console` object.

/// Plain description of a console window's starting size and title.
#[derive(Debug, Clone)]
pub struct ConsoleSpec {
    pub title: String,
    pub width: f32,
    pub height: f32,
}

/// A thread-safe handle to a (possibly not-yet-open) console window.
/// Cloning is cheap — it's just clones of the underlying `Arc`s — so the
/// same handle can be captured by `.print()`, `.clear()`, and `.close()`
/// closures independently, and by the background render thread itself.
#[derive(Clone)]
pub struct ConsoleHandle {
    lines: Arc<Mutex<Vec<String>>>,
    closed: Arc<AtomicBool>,
    // Populated once the window actually opens, so `print_line`/`clear`
    // can nudge it to repaint immediately instead of waiting for its next
    // scheduled frame.
    repaint_ctx: Arc<Mutex<Option<egui::Context>>>,
}

impl ConsoleHandle {
    pub fn new() -> Self {
        Self {
            lines: Arc::new(Mutex::new(Vec::new())),
            closed: Arc::new(AtomicBool::new(false)),
            repaint_ctx: Arc::new(Mutex::new(None)),
        }
    }

    /// Appends one line of text and wakes the window (if open) to redraw.
    pub fn print_line(&self, text: String) {
        self.lines.lock().unwrap().push(text);
        self.wake();
    }

    /// Clears all text currently shown in the console.
    pub fn clear(&self) {
        self.lines.lock().unwrap().clear();
        self.wake();
    }

    /// Requests the window close on its next frame. Safe to call whether
    /// or not the window is currently open.
    pub fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
        self.wake();
    }

    /// True once the window has closed, either via `.close()` or the user
    /// clicking the OS close button.
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    fn wake(&self) {
        if let Some(ctx) = self.repaint_ctx.lock().unwrap().as_ref() {
            ctx.request_repaint();
        }
    }
}

impl Default for ConsoleHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// Every console window's background thread, so [`wait_for_consoles`] can
/// block on them after a script finishes running.
fn console_threads() -> &'static Mutex<Vec<JoinHandle<()>>> {
    static CONSOLE_THREADS: OnceLock<Mutex<Vec<JoinHandle<()>>>> = OnceLock::new();
    CONSOLE_THREADS.get_or_init(|| Mutex::new(Vec::new()))
}

/// Opens a real, terminal-styled OS window for `spec` on a background
/// thread and returns immediately — `handle` stays live and can keep
/// receiving `.print_line()` calls for as long as the window is open.
///
/// **Known limitation:** `winit` (which `eframe` is built on) generally
/// expects its event loop to run on the process's main thread. This
/// works as-is on Windows and Linux, which is where FXterminal has been
/// exercised so far. A strictly main-thread-safe version on macOS would
/// need `flux-cli` to hand the console its main thread directly instead
/// of spawning one here — tracked as a follow-up, same spirit as the
/// image-loading and widget-callback gaps already noted in FXwindows.
pub fn open_console(spec: ConsoleSpec, handle: ConsoleHandle) -> Result<(), String> {
    let width = spec.width.max(240.0);
    let height = spec.height.max(160.0);
    let title = spec.title.clone();

    let join_handle = std::thread::Builder::new()
        .name("fxterminal-console".to_string())
        .spawn(move || {
            let options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default().with_inner_size([width, height]),
                ..Default::default()
            };

            let _ = eframe::run_native(
                &title,
                options,
                Box::new(move |creation_ctx| {
                    *handle.repaint_ctx.lock().unwrap() = Some(creation_ctx.egui_ctx.clone());
                    Ok(Box::new(ConsoleApp { handle }))
                }),
            );
        })
        .map_err(|e| e.to_string())?;

    console_threads().lock().unwrap().push(join_handle);

    Ok(())
}

/// Blocks until every FXterminal console opened so far has closed (via
/// `.close()` or the user hitting the OS close button).
///
/// `.show()` is deliberately non-blocking so a script can keep calling
/// `.print(...)` after it — but that means nothing otherwise keeps the
/// process alive once the script's last statement runs. Without this,
/// the whole process (and every open console with it) exits the instant
/// the script finishes, often before the window has even finished
/// opening. `flux-interpreter` calls this once, after a script's
/// top-level statements are done, so any console left open stays open
/// until it's actually closed.
pub fn wait_for_consoles() {
    let handles: Vec<_> = console_threads().lock().unwrap().drain(..).collect();
    for handle in handles {
        let _ = handle.join();
    }
}

struct ConsoleApp {
    handle: ConsoleHandle,
}

/// Blocks until every FXterminal console opened so far has closed (via
/// `.close()` or the user hitting the OS close button).
///
/// `.show()` is deliberately non-blocking so a script can keep calling
/// `.print(...)` after it — but that means nothing otherwise keeps the
/// process alive once the script's last statement runs. Without this,
/// the whole process (and every open console with it) exits the instant
/// the script finishes, often before the window has even finished
/// opening. `flux-interpreter` calls this once, after a script's
/// top-level statements are done, so any console left open stays open
/// until it's actually closed.
impl eframe::App for ConsoleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // The user clicking the OS close button surfaces here as a
        // close-requested viewport event rather than a separate app-trait
        // callback; mirror it onto the handle either way.
        if ctx.input(|i| i.viewport().close_requested()) {
            self.handle.closed.store(true, Ordering::SeqCst);
        }
        if self.handle.is_closed() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Console output can arrive from another thread at any time, so
        // keep repainting on a short timer as a fallback in addition to
        // the explicit `wake()` nudge in `print_line`/`clear`.
        ctx.request_repaint_after(std::time::Duration::from_millis(200));

        let bg = egui::Color32::from_rgb(18, 18, 18);
        let fg = egui::Color32::from_rgb(64, 220, 120);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(bg).inner_margin(egui::Margin::same(8.0)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let lines = self.handle.lines.lock().unwrap();
                        for line in lines.iter() {
                            ui.label(egui::RichText::new(line).monospace().color(fg));
                        }
                    });
            });
    }
}