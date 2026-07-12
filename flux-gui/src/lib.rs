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
