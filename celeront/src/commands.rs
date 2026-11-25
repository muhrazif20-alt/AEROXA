use std::env;
use std::path::PathBuf;
use std::process::Command;

use crate::config::ProjectConfig;

pub fn run() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("--help") | Some("-h") | Some("help") => {
            show_help();
        }

        Some("--version") | Some("-V") => {
            println!("CELERONT v0.1.0");
            println!("AEROXA Package Manager");
        }

        Some("init") => {
            let name = args
                .get(2)
                .cloned()
                .unwrap_or_else(|| "my_project".to_string());

            let config = ProjectConfig::new(name);

            match config.save() {
                Ok(()) => println!("AEROXA project initialized successfully!"),
                Err(error) => eprintln!("Error: {}", error),
            }
        }

        Some("run") => {
            run_aeroxa(&args);
        }

        Some("install") => {
            println!("Package installation requested.");
        }

        Some("remove") => {
            println!("Package removal requested.");
        }

        Some("update") => {
            println!("Package update requested.");
        }

        Some("search") => {
            println!("Searching packages...");
        }

        Some("list") => {
            println!("Listing installed packages...");
        }

        Some(command) => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Run 'celeront --help' for usage.");
        }

        None => {
            show_help();
        }
    }
}

fn run_aeroxa(args: &[String]) {
    let file = match args.get(2) {
        Some(file) => file,
        None => {
            eprintln!("Usage: celeront run <file.ex>");
            return;
        }
    };

    let file_path = PathBuf::from(file);

    if !file_path.exists() {
        eprintln!("AEROXA source file not found: {}", file);
        return;
    }

    if file_path.extension().and_then(|ext| ext.to_str()) != Some("ex") {
        eprintln!("Error: AEROXA only accepts .ex files.");
        return;
    }

    let compiler = PathBuf::from(
        r"D:\Cargo-target\release\Aeroxa.exe"
    );

    if !compiler.exists() {
        eprintln!("AEROXA compiler executable not found:");
        eprintln!("{}", compiler.display());
        return;
    }

    match Command::new(&compiler)
        .arg(&file_path)
        .output()
    {
        Ok(result) => {
            print!("{}", String::from_utf8_lossy(&result.stdout));
            eprint!("{}", String::from_utf8_lossy(&result.stderr));

            if !result.status.success() {
                eprintln!("AEROXA compilation or execution failed.");
            }
        }

        Err(error) => {
            eprintln!("Failed to execute AEROXA compiler: {}", error);
        }
    }
}

fn show_help() {
    println!("CELERONT v0.1.0");
    println!("AEROXA Package Manager");
    println!();

    println!("USAGE:");
    println!("    celeront <COMMAND>");
    println!();

    println!("COMMANDS:");
    println!("    init <name>       Create a new AEROXA project");
    println!("    run <file.ex>     Compile and run an AEROXA program");
    println!("    install           Install a package");
    println!("    remove            Remove a package");
    println!("    update            Update packages");
    println!("    search            Search available packages");
    println!("    list              List installed packages");
    println!("    help              Display help information");
    println!();

    println!("OPTIONS:");
    println!("    -h, --help        Show help");
    println!("    -V, --version     Show version");
}