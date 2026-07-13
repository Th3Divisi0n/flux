# FLUX

**Fast, Lightweight, Universal eXecution**

FLUX is a modern programming language that combines Python-like readability with compiled-language performance. It is designed for beginners, advanced developers, and anyone building desktop apps, games, tools, or servers.

## Quick Start

```bash
# Run a program (development mode)
fx run examples/hello.fx

# Check version
fx --version

# Create a new project
fx create MyProject
```

## Official Commands

| Command | Description |
|---------|-------------|
| `fx run hello.fx` | Run a FLUX program instantly |
| `fx build hello.fx` | Compile to native executable (Phase 3) |
| `fx create MyProject` | Create a new FLUX project |
| `fx install PackageName` | Install a package |
| `fx remove PackageName` | Remove a package |
| `fx update` | Update installed packages |
| `fx --version` | Show version information |

Both `flux` and `fx` work identically.

## Example

```flux
PRINT "Hello World"

name = "Alex"
age = 20

DEF greet(name):
    RETURN "Hello " + name

PRINT greet(name)

IF age >= 18:
    PRINT "Adult"
ELSE:
    PRINT "Minor"

FOR i IN RANGE(5):
    PRINT i
```

Save as `hello.fx` and run with `fx run hello.fx`.

## Project Structure

```
FLUX/
├── compiler/          # Native compiler backend (Phase 3)
├── flux-lexer/        # Tokenizer
├── flux-parser/       # Parser
├── flux-ast/          # Abstract syntax tree
├── flux-interpreter/  # Development-mode interpreter
├── flux-jit/          # JIT engine (future)
├── flux-runtime/      # Runtime support
├── flux-gui/          # Window rendering backend for FXwindows and FXterminal (winit + egui)
├── flux-cli/          # flux / fx command-line tool
├── standard_library/  # Built-in modules
├── libraries/         # Public FLUX libraries (e.g. FXwindows, FXterminal)
├── package_manager/   # Package registry and resolver
├── registry/          # Bundled local package registry used by `fx install <name>`
├── vscode_extension/  # Official VS Code extension
├── installer/         # Windows installer scripts
├── documentation/     # Language specification and guides
├── examples/          # Example programs
└── tests/             # Integration tests
```

## Building from Source

Requires [Rust](https://rustup.rs/) 1.70+, and network access the first time
you build (to fetch dependencies, including `flux-gui`'s windowing crates).

```bash
cd FLUX
cargo build --release
```

Binaries are placed in `target/release/flux` and `target/release/fx`.

FXwindows (see [`libraries/FXwindows`](libraries/FXwindows/README.md)) opens
real windows by default, which on Linux needs a few system packages to build
and a display to run — see that library's README for details. Build with
`cargo build --release --no-default-features -p flux-interpreter` (or set
`default-features = false` for `flux-interpreter` if you build the whole
workspace) to skip the windowing dependency entirely.

## Development Roadmap

- [x] **Phase 1** — Language specification
- [x] **Phase 2** — Working interpreter (lexer, parser, AST, variables, functions, control flow)
- [ ] **Phase 3** — Native compiler (LLVM backend)
- [x] **Phase 4** — Standard library (FXwindows GUI, FXterminal consoles; files, networking still to come)
- [x] **Phase 5** — Package manager (local install/remove/update against a file-based registry; a hosted network registry is still to come)
- [ ] **Phase 6** — VS Code extension (full IntelliSense)
- [x] **Phase 7** — Windows installer (FLUXSetup.exe)

## License

Apache 2.0 License — see LICENSE for more info