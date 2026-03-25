use std::env;
use std::fs;
use std::process;

use limbaj::{Interpreter, Lexer, Parser, TypeChecker};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: limbaj <command> [args]");
        eprintln!("Commands:");
        eprintln!("  build <file>   - Compile a .limbaj file");
        eprintln!("  run <file>     - Compile and execute a .limbaj file");
        process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "build" => {
            if args.len() < 3 {
                eprintln!("Usage: limbaj build <file>");
                process::exit(1);
            }
            handle_build(&args[2]);
        }
        "run" => {
            if args.len() < 3 {
                eprintln!("Usage: limbaj run <file>");
                process::exit(1);
            }
            handle_run(&args[2]);
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            process::exit(1);
        }
    }
}

fn handle_build(path: &str) -> () {
    match compile_file(path) {
        Ok(_) => {
            println!("Build successful: {}", path);
        }
        Err(e) => {
            eprintln!("Build failed: {}", e);
            process::exit(1);
        }
    }
}

fn handle_run(path: &str) -> () {
    match compile_and_run(path) {
        Ok(output) => {
            if !output.is_empty() && output != "Unit" {
                println!("{}", output);
            }
        }
        Err(e) => {
            eprintln!("Execution failed: {}", e);
            process::exit(1);
        }
    }
}

fn compile_file(path: &str) -> Result<(), String> {
    let contents = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file {}: {}", path, e))?;

    let lexer = Lexer::new(&contents);
    let mut parser = Parser::new(lexer)?;
    let program = parser.parse()?;

    let mut type_checker = TypeChecker::new();
    type_checker.check_program(&program)?;

    Ok(())
}

fn compile_and_run(path: &str) -> Result<String, String> {
    let contents = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file {}: {}", path, e))?;

    let lexer = Lexer::new(&contents);
    let mut parser = Parser::new(lexer)?;
    let program = parser.parse()?;

    let mut type_checker = TypeChecker::new();
    type_checker.check_program(&program)?;

    let mut interpreter = Interpreter::new(&program);
    let result = interpreter.run()?;

    Ok(format!("{}", result))
}
