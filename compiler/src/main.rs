mod lexer;
mod diagnostics;
mod ast;
mod parser;
mod semantic;
mod symbol_table;
mod ir;
mod module_resolver;
mod ir_lowering;
mod backend;
mod ownership;
mod progress;

use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use lexer::tokenize;
use diagnostics::has_errors;
use parser::Parser;
use progress::Progress;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: aeroxa <file.ex> [--compile-only] [--debug-tokens]");
        return;
    }

    let file_path = Path::new(&args[1]);
    let compile_only = args.iter().any(|arg| arg == "--compile-only");
    let debug_tokens = args.iter().any(|arg| arg == "--debug-tokens");

    if file_path.extension().and_then(|ext| ext.to_str()) != Some("ex") {
        eprintln!("Error: AEROXA only accepts .ex files.");
        return;
    }

    // READ SOURCE
    let source = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(error) => {
            eprintln!("Cannot read source file: {}", error);
            return;
        }
    };

    // RESOLVE PROJECT DIRECTORY
    let compiler_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    let project_dir = match compiler_dir.parent() {
        Some(path) => path,
        None => {
            eprintln!("Cannot resolve AEROXA project directory.");
            return;
        }
    };

    // STORAGE DIRECTORIES
    let trash_dir = project_dir.join("trash");
    let build_dir = trash_dir.join("build");
    let temporary_dir = trash_dir.join("temporary");
    let cache_dir = trash_dir.join("cache");

    for directory in [&build_dir, &temporary_dir, &cache_dir] {
        if let Err(error) = fs::create_dir_all(directory) {
            eprintln!("Cannot create storage directory: {}", error);
            return;
        }
    }

    // IDENTIFY SOURCE FILE
    let absolute_source = match fs::canonicalize(file_path) {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Cannot resolve source path: {}", error);
            return;
        }
    };

    let mut source_hasher = DefaultHasher::new();
    absolute_source.hash(&mut source_hasher);

    let source_id = format!("{:016x}", source_hasher.finish());

    // SOURCE FINGERPRINT
    let mut fingerprint_hasher = DefaultHasher::new();
    source.hash(&mut fingerprint_hasher);

    let fingerprint = format!("{:016x}", fingerprint_hasher.finish());

    // BUILD PATHS
    let source_build_dir = build_dir.join(&source_id);

    if let Err(error) = fs::create_dir_all(&source_build_dir) {
        eprintln!("Cannot create build directory: {}", error);
        return;
    }

    let file_name = match file_path.file_stem() {
        Some(name) => name,
        None => {
            eprintln!("Invalid source filename.");
            return;
        }
    };

    let llvm_path = source_build_dir.join(file_name).with_extension("ll");
    let executable_path = source_build_dir.join(file_name).with_extension("exe");
    let cache_path = cache_dir.join(format!("{}.cache", source_id));

    // INCREMENTAL COMPILATION CACHE
    let cache_hit = fs::read_to_string(&cache_path)
        .map(|cached| cached == fingerprint)
        .unwrap_or(false);

    if cache_hit && llvm_path.exists() && executable_path.exists() {
        Progress::cache_hit();

        if compile_only {
            println!("AEROXA: Compilation-only mode. Execution skipped.");
            return;
        }

        run_executable(&executable_path);
        return;
    }

    // START COMPILER TIMER
    let compiler_start = Instant::now();

    // CINEMATIC PROGRESS
    let mut progress = Progress::new(7);

    // STAGE 1: LEXER
    progress.start("LEXICAL ANALYSIS");

    let lexical = tokenize(&source);

    // OPTIONAL TOKEN DEBUG
    if debug_tokens {
        println!("\n========== TOKEN DEBUG ==========");

        for token in &lexical.tokens {
            println!(
                "TOKEN: {:?} | LINE: {} | COLUMN: {}",
                token.token,
                token.location.line,
                token.location.column
            );
        }

        println!("=================================\n");
    }

    if has_errors(&lexical.diagnostics) {
        progress.failed("LEXICAL ANALYSIS");

        for diagnostic in &lexical.diagnostics {
            diagnostic.display();
        }

        return;
    }

    progress.complete("LEXICAL ANALYSIS");

    // STAGE 2: PARSER
    progress.start("SYNTAX ANALYSIS");

    let (mut program, parser_errors) =
        Parser::new(lexical.tokens).parse();

    if has_errors(&parser_errors) {
        progress.failed("SYNTAX ANALYSIS");

        for diagnostic in &parser_errors {
            diagnostic.display();
        }

        return;
    }

    progress.complete("SYNTAX ANALYSIS");

    // MODULE RESOLUTION
    let module_root = project_dir.join("modules");

    let mut module_resolver =
        module_resolver::ModuleResolver::new();

    let modules = match module_resolver.resolve(
        &absolute_source,
        &module_root,
    ) {
        Ok(modules) => modules,
        Err(error) => {
            eprintln!("Module Resolution Error: {}", error);
            return;
        }
    };

    println!(
        "[MODULE RESOLVER] Loaded {} module(s).",
        modules.len()
    );

    // MERGE MODULE AST
    for module in modules {
        println!(
            "[MODULE] Integrating: {}",
            module.name
        );

        program
            .statements
            .extend(module.program.statements);
    }

    // STAGE 3: SEMANTIC ANALYSIS
    progress.start("SEMANTIC ANALYSIS");

    let semantic_errors = semantic::analyze(&program);

    if has_errors(&semantic_errors) {
        progress.failed("SEMANTIC ANALYSIS");

        for diagnostic in &semantic_errors {
            diagnostic.display();
        }

        return;
    }

    progress.complete("SEMANTIC ANALYSIS");

    // STAGE 4: OWNERSHIP ANALYSIS
    progress.start("OWNERSHIP ANALYSIS");

    let mut ownership_analyzer =
        ownership::analyzer::OwnershipAnalyzer::new();

    ownership_analyzer.analyze_program(&program);

    let ownership_errors = ownership_analyzer.diagnostics();

    if !ownership_errors.is_empty() {
        progress.failed("OWNERSHIP ANALYSIS");

        for diagnostic in ownership_errors {
            diagnostic.display();
        }

        return;
    }

    progress.complete("OWNERSHIP ANALYSIS");

    // STAGE 5: AST TO IR
    progress.start("INTERMEDIATE REPRESENTATION");

    let ir_module = match ir_lowering::lower_program(&program) {
        Ok(module) => module,
        Err(error) => {
            progress.failed("IR LOWERING");
            eprintln!("IR Lowering Error: {}", error);
            return;
        }
    };

    progress.complete("INTERMEDIATE REPRESENTATION");

    // STAGE 6: LLVM IR GENERATION
    progress.start("LLVM IR GENERATION");

    let llvm_ir = match backend::generate_llvm_ir(&ir_module) {
        Ok(output) => output,
        Err(error) => {
            progress.failed("LLVM IR GENERATION");
            eprintln!("Backend Error: {}", error);
            return;
        }
    };

    if let Err(error) = fs::write(&llvm_path, &llvm_ir) {
        progress.failed("LLVM IR OUTPUT");
        eprintln!("Cannot write LLVM IR: {}", error);
        return;
    }

    progress.complete("LLVM IR GENERATION");

    println!(
        "[TIMING] AEROXA Frontend + LLVM Backend: {:.3} ms",
        compiler_start.elapsed().as_secs_f64() * 1000.0
    );

    // STAGE 7: NATIVE COMPILATION
    let clang_start = Instant::now();

    progress.start("NATIVE COMPILATION");

    let compilation = Command::new("clang")
        .arg(&llvm_path)
        .arg("-O2")
        .arg("-o")
        .arg(&executable_path)
        .output();

    match compilation {
        Ok(result) => {
            if !result.status.success() {
                progress.failed("NATIVE COMPILATION");

                eprintln!("Native compilation failed.");
                eprintln!("{}", String::from_utf8_lossy(&result.stderr));

                return;
            }
        }
        Err(error) => {
            progress.failed("NATIVE COMPILATION");
            eprintln!("Cannot execute Clang: {}", error);
            return;
        }
    }

    println!(
        "[TIMING] Clang Native Compilation: {:.3} ms",
        clang_start.elapsed().as_secs_f64() * 1000.0
    );

    progress.complete("NATIVE COMPILATION");

    // SAVE CACHE
    if let Err(error) = fs::write(&cache_path, &fingerprint) {
        eprintln!("Warning: Cannot save build cache: {}", error);
    }

    // COMPILE-ONLY MODE
    if compile_only {
        println!();
        println!("\x1b[96m  >>> COMPILATION FINISHED <<<\x1b[0m");
        println!("\x1b[92m  EXECUTION SKIPPED\x1b[0m");
        println!();

        return;
    }

    // EXECUTE PROGRAM
    println!();
    println!("\x1b[96m  >>> LAUNCHING AEROXA PROGRAM <<<\x1b[0m");
    println!();

    run_executable(&executable_path);
}

// EXECUTE NATIVE PROGRAM
fn run_executable(executable_path: &Path) {
    let execution = Command::new(executable_path).output();

    match execution {
        Ok(result) => {
            print!("{}", String::from_utf8_lossy(&result.stdout));

            if !result.status.success() {
                eprintln!("Program execution failed.");
                eprintln!("{}", String::from_utf8_lossy(&result.stderr));
            }
        }
        Err(error) => {
            eprintln!("Cannot execute AEROXA program: {}", error);
        }
    }
}