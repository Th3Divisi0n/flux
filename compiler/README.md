# FLUX Compiler (Phase 3)

The FLUX native compiler will compile `.fx` source files to optimized native executables using LLVM.

## Pipeline

```
.fx source → Lexer → Parser → AST → IR Generator → LLVM → Native .exe
```

## Status

**Not yet implemented.** Use `fx run` for development.

## Planned Features

- Native executable generation (`fx build`)
- Optimization passes (-O1, -O2, -O3)
- Cross-platform targets (Windows, Linux, macOS)
- Debug symbols
- Static linking of FLUX runtime

## Implementation

Recommended stack:
- **Frontend:** Rust (flux-lexer, flux-parser, flux-ast)
- **Backend:** LLVM via inkwell or llvm-sys
- **Runtime:** flux-runtime (linked into executables)
