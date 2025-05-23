// || shree ganesh ||
// VORTEX: main.rs

mod ast;
mod gpu_runtime;
mod interpreter;
mod lexer;
mod parser;
mod repl;
mod token;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use token::Token;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    match args.len() {
        1 => {
            // No arguments - start REPL
            if let Err(e) = repl::start_repl() {
                eprintln!("REPL error: {}", e);
                process::exit(1);
            }
        }
        2 => {
            // One argument - execute file
            let filename = &args[1];
            if let Err(e) = execute_file(filename) {
                eprintln!("Execution error: {}", e);
                process::exit(1);
            }
        }
        _ => {
            // Too many arguments
            print_usage(&args[0]);
            process::exit(1);
        }
    }
}

fn print_usage(program_name: &str) {
    println!("VORTEX Language Interpreter");
    println!();
    println!("Usage:");
    println!("  {}              Start interactive REPL", program_name);
    println!("  {} <file.vx>    Execute Vortex file", program_name);
    println!();
    println!("Examples:");
    println!("  {}              # Interactive mode", program_name);
    println!("  {} example.vx   # Run example.vx", program_name);
}

fn execute_file(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("=============================================================");
    println!("|| VORTEX LANGUAGE - GPU-accelerated programming language ||");
    println!("=============================================================\n");
    
    println!("Executing file: {}\n", filename);

    // Read the file
    let source = fs::read_to_string(filename)?;

    println!("======================= LEXICAL ANALYSIS =======================");

    // Step 1: Lexing
    let mut lexer = Lexer::new(&source);
    let mut tokens = Vec::new();

    loop {
        let token = lexer.next_token();
        if token == Token::EOF {
            break;
        }
        tokens.push(token);
    }

    println!("Tokens generated: {}", tokens.len());

    println!("\n========================= PARSING ============================");

    // Step 2: Parsing
    let mut parser = Parser::new(tokens);
    let program = parser.parse();

    println!("\nAbstract Syntax Tree (AST):");
    for (i, stmt) in program.iter().enumerate() {
        println!("Statement {}: {:?}", i, stmt);
    }

    println!("\n====================== INTERPRETATION ========================");
    
    // Step 3: Interpret
    let mut interpreter = Interpreter::new();
    match interpreter.interpret(program) {
        Ok(_) => println!("\nExecution completed successfully."),
        Err(e) => return Err(format!("Runtime error: {}", e).into()),
    }
    
    println!("\n=============================================================");
    println!("|| VORTEX execution finished                              ||");
    println!("=============================================================");
    
    Ok(())
}
