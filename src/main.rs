// || shree ganesh ||
// VORTEX: main.rs

mod ast;
mod gpu_runtime;
mod interpreter;
mod lexer;
mod parser;
mod token;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use token::Token;

fn main() {
    println!("=============================================================");
    println!("|| VORTEX LANGUAGE - GPU-accelerated programming language ||");
    println!("=============================================================\n");

    let source = r#"
        // Variable Declarations
        let x: Int = 5
        let mut y: Float = 3.14

        // Standard Conditional
        if x > 10:
            print("x is big")
        then x == 10:
            print("x is ten")
        else:
            print("x is small")

        // GPU-Inspired Conditional
        branch x > 10 => print("x is big")
        branch x == 10 => print("x is ten")
        fallback => print("x is small")

        // Standard Loop
        for i in range(0, 3):
            print(i)

        // GPU Loop
        parallel i in 0..3:
            print(i)

        // CPU Function
        fn add_cpu(a: Int, b: Int) -> Int:
            return a + b

        // GPU Function
        @gpu fn add_gpu(a: Int, b: Int):
            return a + b

        // Function Calls
        let cpu_result = add_cpu(x, 10)
        print("CPU result:")
        print(cpu_result)

        let gpu_result = add_gpu(x, 10)
        print("GPU result:")
        print(gpu_result)
    "#;

    println!("Source code:\n{}\n", source);
    println!("======================= LEXICAL ANALYSIS =======================");

    // Step 1: Lexing
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();

    loop {
        let token = lexer.next_token();
        if token == Token::EOF {
            break;
        }
        tokens.push(token);
    }

    println!("Tokens generated: {}", tokens.len());
    println!("\nToken stream:");
    for (i, tok) in tokens.iter().enumerate() {
        println!("{:3}: {:?}", i, tok);
    }

    println!("\n========================= PARSING ============================");

    // Step 2: Parsing
    let mut parser = Parser::new(tokens.clone());
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
        Err(e) => eprintln!("\nRuntime error: {}", e),
    }

    println!("\n=============================================================");
    println!("|| VORTEX execution finished                              ||");
    println!("=============================================================");
}
