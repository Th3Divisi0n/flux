# FXwindows

FXwindows is FLUX's built-in library for creating simple desktop-style
windows and UI elements. It ships with the FLUX runtime (`flux-interpreter`),
so there is nothing extra to install — just `IMPORT FXwindows` in any `.fx`
script.

```flux
IMPORT FXwindows

window = FXwindows.create_window("My FLUX App", 800, 600)

button = FXwindows.Button("Click Me", 100, 50)
label = FXwindows.Label("Welcome to FLUX", 20, 20)
checkbox = FXwindows.Checkbox("Enable sound", false, 20, 80)

window.add(button)
window.add(label)
window.add(checkbox)

button.text = "New Text"
PRINT button.text

window.show()
```

Run it with the bundled demo:

```bash
fx run examples/fxwindows_demo.fx
```

## API

### `FXwindows.create_window(title, width, height)`

Creates a `Window` object.

| Field / Method | Description |
|---|---|
| `.title`, `.width`, `.height` | Window properties (settable) |
| `.add(widget)` | Adds a widget to the window |
| `.show()` | Displays the window |

### Widgets

Each widget constructor returns an object whose fields you can read and
reassign like any other FLUX object (e.g. `button.text = "New Text"`).

| Constructor | Fields (in order) |
|---|---|
| `FXwindows.Button(text, width, height)` | `text`, `width`, `height` |
| `FXwindows.Label(text, x, y)` | `text`, `x`, `y` |
| `FXwindows.TextBox(placeholder, x, y)` | `placeholder`, `x`, `y` |
| `FXwindows.Image(path, x, y)` | `path`, `x`, `y` |
| `FXwindows.Panel(x, y, width, height)` | `x`, `y`, `width`, `height` |
| `FXwindows.Slider(min, max, value, x, y)` | `min`, `max`, `value`, `x`, `y` |
| `FXwindows.Checkbox(label, checked, x, y)` | `label`, `checked`, `x`, `y` |

Add widgets to a window with `window.add(widget)`, then call
`window.show()` to display it.

## Current status

`window.show()` now opens a **real, interactive OS window** via the new
`flux-gui` crate (built on `winit` + `egui`), and blocks until the user
closes it. Buttons, labels, checkboxes, sliders, text boxes, and panels
are all actually drawn — sliders/checkboxes/text boxes are interactive
within the window while it's open.

Two known limitations of the current rendering:

- **Images aren't loaded from disk yet.** `FXwindows.Image(path, x, y)`
  renders as a bordered placeholder box labeled with the path, rather
  than the actual image file. Wiring up `egui`'s image loader is a small
  follow-up.
- **Widget interaction doesn't write back into FLUX.** Dragging a slider
  or toggling a checkbox updates it visually in the window, but there's
  no callback/event system in the language yet for that interaction to
  feed back into your FLUX variables. That would need FLUX to support
  passing functions/closures as event handlers — a bigger, separate
  language feature.

### Build & run requirements

- **First build needs network access.** `flux-gui` depends on `eframe`
  (which pulls in `winit`, `egui`, and a GPU backend). `cargo build` will
  need to fetch these from crates.io the first time, and `Cargo.lock`
  will be updated with the new dependency tree.
- **Linux needs some system packages** for winit/glow to build and link,
  typically something like:
  ```bash
  sudo apt install libxkbcommon-dev libxcb1-dev libx11-dev libgl1-mesa-dev
  ```
  (exact package names vary by distro).
- **Running it needs a display** — a normal desktop session on
  Linux/macOS/Windows, or something like `Xvfb` if you need to run it
  headless.
- Don't want any of this? Build with `cargo build --no-default-features`
  to disable the `gui` feature — `.show()` then falls back to printing a
  one-line text summary instead of opening a window, and you avoid the
  `winit`/`egui` dependency entirely.

## Where the implementation lives

The FLUX-facing object model (window/widget creation, `.add()`, field
mutation) is implemented natively in Rust inside
`flux-interpreter/src/lib.rs` — see `builtin_fxwindows_module`,
`widget_constructor`, and `window_object_to_gui_spec`. The actual
rendering — opening the OS window and drawing widgets every frame —
lives in the separate `flux-gui` crate (`flux-gui/src/lib.rs`), which
knows nothing about the FLUX language; it just takes a plain
`WindowSpec` and renders it. That separation keeps the interpreter free
of GUI-toolkit details and means `flux-gui` could be reused as a
rendering backend outside of FLUX if needed.
