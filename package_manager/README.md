# FLUX Package Manager (Phase 5)

The FLUX package manager handles library installation and dependency
tracking for a project. It's implemented directly in `flux-cli` — there's
no separate service or daemon.

## Status

**Implemented, backed by a local file-based registry** — there's no
network registry yet (see "Local registry" below), but `fx install`,
`fx remove`, and `fx update` are real: they copy files, and they read and
write your project's `flux.toml`.

## Commands

```bash
fx install FXstrings     # Install a package by name, from the registry
fx install ./MyLibrary   # Install a package from a local path
fx remove FXstrings      # Remove an installed package
fx update                # Re-install every dependency at its latest version
```

Run these from inside a FLUX project (a directory with a `flux.toml` —
see `fx create`).

## What `fx install` does

1. Finds the package — either at the given local path, or by name inside
   the registry (see below).
2. Reads the package's own `flux.toml` (`[project] name`/`version`).
3. Copies the whole package directory into `flux_modules/<name>/` in your
   project, replacing any previous install of that package.
4. Adds (or updates) a `name = "version"` line under `[dependencies]` in
   your project's `flux.toml`.

From FLUX code, an installed package is imported exactly like a built-in
module:

```flux
IMPORT FXstrings

PRINT FXstrings.reverse("flux")
```

`IMPORT <Name>` first checks FLUX's built-in modules (`math`, `io`, `sys`,
`FXwindows`, `FXterminal`); if `<Name>` isn't one of those, the
interpreter looks for `flux_modules/<Name>/` and runs its
`src/library.fx` entry point, exporting every top-level name it defines.
See `documentation/LANGUAGE_SPEC.md` §7 for the full package layout.

`fx remove <name>` deletes `flux_modules/<name>/` and drops it from
`[dependencies]`. `fx update` walks every dependency in `[dependencies]`,
and for each one found in the registry, re-copies it and rewrites its
version if it changed; dependencies installed from a local path are left
alone (there's no path recorded to re-copy from — reinstall with
`fx install <path>` to update those).

## Local registry

There's no hosted registry yet, so "install by name" resolves against a
local, file-based one instead — literally a folder of package
directories, each with its own `flux.toml` + `src/`, no publishing step.
`fx install <name>` looks for `<name>` in, in order:

1. `$FLUX_REGISTRY`, if set.
2. `registry/` next to the running `flux`/`fx` executable.
3. `./registry` in the current directory (this is what lets `fx install
   FXstrings` work from inside the FLUX repo itself, where the example
   package lives — see [`registry/FXstrings`](../registry/FXstrings)).

Each version is a full copy, not a diff — fine for a bundled example
registry, but a real registry would need proper versioning and a
resolver for shared transitive dependencies. That's future work, along
with actually publishing to a hosted registry over the network.

## Local Development

```bash
fx install ./MyLibrary   # Install straight from a local path — works today
```
