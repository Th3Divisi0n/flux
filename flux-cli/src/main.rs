use clap::{Parser, Subcommand};
use flux_interpreter::interpret;
use flux_runtime::manifest;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const TAGLINE: &str = "Fast, Lightweight, Universal eXecution";

#[derive(Parser)]
#[command(name = "flux")]
#[command(version, about = "FLUX — Fast, Lightweight, Universal eXecution", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a FLUX program instantly (development mode)
    Run {
        /// Path to the .fx source file
        file: PathBuf,
    },
    /// Compile a FLUX program to a native executable (production mode)
    Build {
        /// Path to the .fx source file
        file: PathBuf,
        /// Output executable name
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Create a new FLUX project
    Create {
        /// Project name
        name: String,
    },
    /// Install a package
    Install {
        /// Package name
        package: String,
    },
    /// Remove an installed package
    Remove {
        /// Package name
        package: String,
    },
    /// Update installed packages
    Update,
}

fn main() {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        match command {
            Commands::Run { file } => run_file(&file),
            Commands::Build { file, output } => build_file(&file, output.as_ref()),
            Commands::Create { name } => create_project(&name),
            Commands::Install { package } => install_package(&package),
            Commands::Remove { package } => remove_package(&package),
            Commands::Update => update_packages(),
        }
    } else {
        print_help();
    }
}

fn run_file(path: &Path) {
    if path.extension().and_then(|e| e.to_str()) != Some("fx") {
        eprintln!("error: FLUX source files must use the .fx extension");
        process::exit(1);
    }

    let source = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("error: could not read '{}': {e}", path.display());
            process::exit(1);
        }
    };

    if let Err(e) = interpret(&source) {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn build_file(path: &Path, output: Option<&PathBuf>) {
    if path.extension().and_then(|e| e.to_str()) != Some("fx") {
        eprintln!("error: FLUX source files must use the .fx extension");
        process::exit(1);
    }

    let output_name = output.cloned().unwrap_or_else(|| {
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        if cfg!(windows) {
            PathBuf::from(format!("{stem}.exe"))
        } else {
            PathBuf::from(stem.to_string())
        }
    });

    eprintln!("FLUX compiler backend (LLVM) is planned for Phase 3.");
    eprintln!("Building '{}' -> '{}'...", path.display(), output_name.display());
    eprintln!();
    eprintln!("For now, use development mode:");
    eprintln!("  fx run {}", path.display());
    eprintln!();
    eprintln!("Native compilation will produce optimized executables in a future release.");
    process::exit(0);
}

fn create_project(name: &str) {
    let project_dir = PathBuf::from(name);

    if project_dir.exists() {
        eprintln!("error: directory '{}' already exists", project_dir.display());
        process::exit(1);
    }

    let dirs = ["src"];
    for dir in dirs {
        fs::create_dir_all(project_dir.join(dir)).expect("failed to create directory");
    }

    let flux_toml = format!(
        r#"[project]
name = "{name}"
version = "0.1.0"
description = "A FLUX project"
authors = ["Developer"]
flux_version = "1.0.0"

[dependencies]
"#
    );

    let main_fx = r#"PRINT "Hello from FLUX!"

DEF greet(name):
    RETURN "Hello, " + name

message = greet("World")
PRINT message
"#;

    let readme = format!(
        r#"# {name}

A FLUX project.

## Run

```bash
fx run src/main.fx
```

## Build

```bash
fx build src/main.fx
```
"#
    );

    fs::write(project_dir.join("flux.toml"), flux_toml).expect("failed to write flux.toml");
    fs::write(project_dir.join("src/main.fx"), main_fx).expect("failed to write main.fx");
    fs::write(project_dir.join("README.md"), readme).expect("failed to write README.md");

    println!("Created FLUX project '{name}'");
    println!();
    println!("  {name}/");
    println!("  ├── flux.toml");
    println!("  ├── src/");
    println!("  │   └── main.fx");
    println!("  └── README.md");
    println!();
    println!("Get started:");
    println!("  cd {name}");
    println!("  fx run src/main.fx");
}

/// Where `fx install <name>` (as opposed to `fx install <path>`) looks
/// for packages by name. Checked in order:
///   1. `$FLUX_REGISTRY`, if set — lets anyone point at their own registry.
///   2. `registry/` next to the running `fx`/`flux` executable — where an
///      installed FLUX toolchain would bundle it.
///   3. `./registry` in the current directory — where FLUX's own repo
///      keeps its bundled example packages during development.
/// This is a local, file-based stand-in for a real network registry (see
/// `package_manager/README.md`); there's no publishing step, just
/// pre-populated package directories.
fn registry_dir() -> PathBuf {
    if let Ok(path) = std::env::var("FLUX_REGISTRY") {
        return PathBuf::from(path);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join("registry");
            if candidate.is_dir() {
                return candidate;
            }
        }
    }
    PathBuf::from("registry")
}

/// Recursively copies `src` into `dst`, creating directories as needed.
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Reads the project's `flux.toml` in the current directory, or exits
/// with an error if this isn't inside a FLUX project.
fn require_project_manifest() -> PathBuf {
    let path = PathBuf::from("flux.toml");
    if !path.exists() {
        eprintln!("error: no flux.toml found in this directory.");
        eprintln!("Run `fx create <name>` to start a new project, or `cd` into one first.");
        process::exit(1);
    }
    path
}

fn install_package(package: &str) {
    let project_manifest_path = require_project_manifest();

    let looks_like_path =
        package.starts_with('.') || package.starts_with('/') || package.starts_with('~');

    let source_dir = if looks_like_path || Path::new(package).is_dir() {
        let dir = PathBuf::from(package);
        if !dir.is_dir() {
            eprintln!("error: local package path '{package}' does not exist or isn't a directory");
            process::exit(1);
        }
        dir
    } else {
        let candidate = registry_dir().join(package);
        if !candidate.is_dir() {
            eprintln!(
                "error: package '{package}' not found in the registry ({})",
                registry_dir().display()
            );
            eprintln!("Tip: to install from a local path instead, run `fx install ./path/to/{package}`");
            process::exit(1);
        }
        candidate
    };

    let manifest_text = match fs::read_to_string(source_dir.join("flux.toml")) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("error: '{}' has no flux.toml ({e})", source_dir.display());
            process::exit(1);
        }
    };

    let name = manifest::get_field(&manifest_text, "project", "name")
        .unwrap_or_else(|| package.to_string());
    let version =
        manifest::get_field(&manifest_text, "project", "version").unwrap_or_else(|| "0.0.0".to_string());

    let dest_dir = PathBuf::from("flux_modules").join(&name);
    if let Err(e) = fs::remove_dir_all(&dest_dir) {
        if e.kind() != std::io::ErrorKind::NotFound {
            eprintln!("error: could not clear previous install at '{}': {e}", dest_dir.display());
            process::exit(1);
        }
    }
    if let Err(e) = copy_dir_recursive(&source_dir, &dest_dir) {
        eprintln!("error: failed to install '{name}': {e}");
        process::exit(1);
    }

    let project_toml = fs::read_to_string(&project_manifest_path).unwrap_or_default();
    let updated = manifest::upsert_field(&project_toml, "dependencies", &name, &version);
    if let Err(e) = fs::write(&project_manifest_path, updated) {
        eprintln!("error: installed files but failed to update flux.toml: {e}");
        process::exit(1);
    }

    println!("Installed {name} {version} -> flux_modules/{name}/");
    println!("Use it in FLUX with: IMPORT {name}");
}

fn remove_package(package: &str) {
    let project_manifest_path = require_project_manifest();

    let dest_dir = PathBuf::from("flux_modules").join(package);
    let was_installed = dest_dir.is_dir();
    if was_installed {
        if let Err(e) = fs::remove_dir_all(&dest_dir) {
            eprintln!("error: failed to remove '{}': {e}", dest_dir.display());
            process::exit(1);
        }
    }

    let project_toml = fs::read_to_string(&project_manifest_path).unwrap_or_default();
    let updated = manifest::remove_field(&project_toml, "dependencies", package);
    if let Err(e) = fs::write(&project_manifest_path, updated) {
        eprintln!("error: failed to update flux.toml: {e}");
        process::exit(1);
    }

    if was_installed {
        println!("Removed {package}");
    } else {
        println!("{package} wasn't installed (cleared from flux.toml dependencies, if it was listed)");
    }
}

fn update_packages() {
    let project_manifest_path = require_project_manifest();
    let mut project_toml = fs::read_to_string(&project_manifest_path).unwrap_or_default();

    let deps = manifest::section_pairs(&project_toml, "dependencies");
    if deps.is_empty() {
        println!("No dependencies to update.");
        return;
    }

    let registry = registry_dir();
    for (name, current_version) in deps {
        let candidate = registry.join(&name);
        if !candidate.is_dir() {
            println!(
                "{name}: skipped — not in the registry ({}). If it was installed from a local \
                 path, run `fx install <path>` again to pick up changes.",
                registry.display()
            );
            continue;
        }

        let manifest_text = fs::read_to_string(candidate.join("flux.toml")).unwrap_or_default();
        let latest_version =
            manifest::get_field(&manifest_text, "project", "version").unwrap_or_else(|| current_version.clone());

        let dest_dir = PathBuf::from("flux_modules").join(&name);
        let _ = fs::remove_dir_all(&dest_dir);
        if let Err(e) = copy_dir_recursive(&candidate, &dest_dir) {
            eprintln!("{name}: update failed: {e}");
            continue;
        }

        project_toml = manifest::upsert_field(&project_toml, "dependencies", &name, &latest_version);

        if latest_version == current_version {
            println!("{name}: already up to date ({current_version})");
        } else {
            println!("{name}: {current_version} -> {latest_version}");
        }
    }

    if let Err(e) = fs::write(&project_manifest_path, project_toml) {
        eprintln!("error: failed to update flux.toml: {e}");
        process::exit(1);
    }
}

fn print_help() {
    println!("FLUX {VERSION}");
    println!("{TAGLINE}");
    println!();
    println!("Usage: flux [COMMAND]");
    println!();
    println!("Commands:");
    println!("  run <file.fx>        Run a FLUX program instantly");
    println!("  build <file.fx>      Compile to native executable");
    println!("  create <name>        Create a new FLUX project");
    println!("  install <package>    Install a package");
    println!("  remove <package>    Remove a package");
    println!("  update               Update installed packages");
    println!();
    println!("Options:");
    println!("  --version            Show version information");
    println!("  --help               Show this help message");
    println!();
    println!("Both 'flux' and 'fx' work identically.");
}
