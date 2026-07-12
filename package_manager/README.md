# FLUX Package Manager (Phase 5)

The FLUX package manager handles library discovery, installation, and dependency resolution.

## Commands

```bash
fx install MyLibrary    # Install a package
fx remove MyLibrary     # Remove a package
fx update               # Update all packages
```

## Package Registry

Packages are defined with `flux.toml` and published to the FLUX registry (planned).

## Local Development

```bash
fx install ./MyLibrary   # Install from local path (planned)
```

## Status

**Stub implementation in FLUX 1.0.0.** Commands print guidance until Phase 5 is complete.
