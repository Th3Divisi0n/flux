use clap::{Parser, Subcommand};
use flux_interpreter::interpret;
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

fn install_package(package: &str) {
    eprintln!("Package manager is planned for Phase 5.");
    eprintln!("Would install package: {package}");
    eprintln!();
    eprintln!("Package layout:");
    eprintln!("  {package}/");
    eprintln!("  ├── flux.toml");
    eprintln!("  ├── src/");
    eprintln!("  │   └── library.fx");
    eprintln!("  └── README.md");
    process::exit(0);
}

fn remove_package(package: &str) {
    eprintln!("Package manager is planned for Phase 5.");
    eprintln!("Would remove package: {package}");
    process::exit(0);
}

fn update_packages() {
    eprintln!("Package manager is planned for Phase 5.");
    eprintln!("Would update all installed packages.");
    process::exit(0);
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
