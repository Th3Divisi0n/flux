# FLUX Standard Library

Built-in modules available in FLUX 1.0.0.

## math

```flux
IMPORT math

PRINT math.abs(-5)
PRINT math.max(1, 5, 3)
PRINT math.min(1, 5, 3)
```

## io

```flux
IMPORT io

content = io.read_file("data.txt")
io.write_file("output.txt", "Hello FLUX")
```

## sys

```flux
IMPORT sys

PRINT sys.version
PRINT sys.platform
```

## FXwindows

Window creation and basic UI widgets (buttons, labels, checkboxes, etc).
See [`libraries/FXwindows/README.md`](../libraries/FXwindows/README.md)
for the full API and an example.

```flux
IMPORT FXwindows

window = FXwindows.create_window("My FLUX App", 800, 600)
window.show()
```

## Planned Modules

| Module | Purpose |
|--------|---------|
| `graphics` | 2D/3D rendering, images |
| `audio` | Sound playback and recording |
| `net` | HTTP, TCP, WebSocket |
| `db` | Database connectors |
| `fs` | Advanced file system operations |
| `process` | Process management |
| `thread` | Multithreading |
| `async` | Async I/O |
