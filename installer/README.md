# FLUX Windows Installer

Build script for `FLUXSetup.exe` — the official Windows installer.

## Installer Features

- Install FLUX compiler and runtime
- Add `flux` and `fx` to PATH
- Install standard library
- Install VS Code extension
- Create examples folder
- Create project templates

## Build (Future)

Installer will be built with [WiX Toolset](https://wixtoolset.org/) or [Inno Setup](https://jrsoftware.org/isinfo.php).

## Manual Install (Development)

```powershell
cd FLUX
cargo build --release

# Add to PATH (PowerShell)
$env:PATH += ";F:\FLUX\target\release"

# Verify
fx --version
```

## Post-Install

```powershell
fx --version
# FLUX 1.0.0
# Fast, Lightweight, Universal eXecution

fx run examples\hello.fx
```
