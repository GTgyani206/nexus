use wasm_bindgen::prelude::*;
use std::panic;
use std::cell::RefCell;

thread_local! {
    static INTERPRETER: RefCell<Interpreter> = RefCell::new(Interpreter::new());
}

pub mod vortex_core;
use vortex_core::lexer::Lexer;
use vortex_core::parser::Parser;
use vortex_core::interpreter::Interpreter;
use vortex_core::token::Token;

#[wasm_bindgen]
pub fn run_vortex(input: &str) -> String {
    // Set panic hook for better error messages in WASM
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    
    // Lexing
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        if token == Token::EOF { break; }
        tokens.push(token);
    }

    // Parsing
    let mut parser = Parser::new(tokens);
    let program = parser.parse();

    // Interpret using persistent interpreter
    INTERPRETER.with(|cell| {
        let mut interpreter = cell.borrow_mut();
        match interpreter.interpret(program) {
            Ok(Some(val)) => format!("{}\n", val),
            Ok(None) => "✅ Execution completed successfully.\n".to_string(),
            Err(e) => format!("❌ Runtime error: {}\n", e),
        }
    })
}
