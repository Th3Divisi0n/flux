# FXterminal

FXterminal is FLUX's built-in library for opening real, terminal-styled
console windows and printing into them while a program keeps running.
It ships with the FLUX runtime (`flux-interpreter`), same as FXwindows —
just `IMPORT FXterminal` in any `.fx` script, nothing to install.

```flux
IMPORT FXterminal

console = FXterminal.create_console("Build Log", 640, 400)
console.show()

console.print("Starting build...")
console.print("Compiling flux-lexer...")
console.print("Compiling flux-parser...")
console.print("Done.")
```

Run it with the bundled demo:

```bash
fx run examples/fxterminal_demo.fx
```

## API

### `FXterminal.create_console(title, width, height)`

Creates a `Console` object.

| Field / Method | Description |
|---|---|
| `.title`, `.width`, `.height` | Console properties, set at creation |
| `.show()` | Opens the console window |
| `.print(text)` | Appends one line of text |
| `.clear()` | Clears everything printed so far |
| `.close()` | Closes the window |

## How this differs from FXwindows

`FXwindows.window.show()` blocks until the user closes the window — it's
called once, after every widget has already been added. A console is
meant to be printed to *while it's open and your program is still doing
things*, so `Console.show()` is intentionally **non-blocking**: it opens
the window on a background thread and returns immediately, and the
`.print(...)` calls that follow keep feeding it new lines for as long as
the window stays open.

## Current status

With the default `gui` feature enabled, `.show()` opens a real OS window
(via the same `flux-gui` crate FXwindows uses — `winit` + `egui`) styled
like a dark terminal, with monospace text that auto-scrolls to the newest
line. Built with `--no-default-features` (or on a headless box), there's
no window: `.print(text)` just writes to stdout instead, prefixed with
the console's title, and `.show()`/`.clear()`/`.close()` are no-ops.

Known limitations, in the same spirit as FXwindows' documented gaps:

- **macOS threading.** `winit`'s event loop generally wants to run on the
  process's main thread. FXterminal opens its window on a background
  thread so `.print()` calls can keep flowing after `.show()` returns,
  which works on Windows and Linux but may need a main-thread-handoff
  redesign to be fully solid on macOS.
- **No colored/styled text yet.** Every line renders the same monospace
  green-on-black style; there's no way (yet) to mark a line as an error,
  a warning, etc.
- **No input.** FXterminal is output-only for now — there's no
  `console.read_line()` equivalent.
