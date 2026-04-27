// || shree ganesh ||
// REPL (Read-Eval-Print Loop) for Vortex Language

use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::token::Token;
use colored::*;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use std::fs;
use std::path::Path;

pub struct VortexRepl {
    interpreter: Interpreter,
    editor: DefaultEditor,
    version: String,
    multi_line_buffer: String,
    in_multi_line: bool,
    bracket_depth: i32,
}

impl VortexRepl {
    pub fn new() -> Result<Self> {
        let mut editor = DefaultEditor::new()?;
        
        // Try to load history from file
        let _ = editor.load_history("vortex_history.txt");
        
        Ok(VortexRepl {
            interpreter: Interpreter::new(),
            editor,
            version: "0.1.0".to_string(),
            multi_line_buffer: String::new(),
            in_multi_line: false,
            bracket_depth: 0,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.print_banner();
        self.print_help();

        loop {
            let prompt = self.get_prompt();
            
            match self.editor.readline(&prompt) {
                Ok(line) => {
                    // Add to history if not empty and not a command
                    if !line.trim().is_empty() && !line.trim().starts_with(':') {
                        self.editor.add_history_entry(line.as_str()).ok();
                    }

                    // Handle commands
                    if line.trim().starts_with(':') {
                        if self.handle_command(&line) {
                            break; // Exit command
                        }
                        continue;
                    }

                    // Handle regular Vortex code
                    self.handle_input(&line);
                }
                Err(ReadlineError::Interrupted) => {
                    println!("{}", "^C".red());
                    if self.in_multi_line {
                        self.reset_multi_line();
                        continue;
                    }
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("{}", "Goodbye!".yellow());
                    break;
                }
                Err(err) => {
                    println!("{}: {:?}", "Error".red(), err);
                    break;
                }
            }
        }

        // Save history
        self.editor.save_history("vortex_history.txt").ok();
        Ok(())
    }

    fn print_banner(&self) {
        println!("{}", "=============================================================".cyan());
        println!("{}", "||        VORTEX LANGUAGE - Interactive REPL             ||".cyan());
        println!("{}", "=============================================================".cyan());
        println!();
        println!("{} {}", "Version:".green().bold(), self.version.white());
        
        // Check GPU availability
        if self.interpreter.gpu_runtime.is_available() {
            println!("{} {}", "GPU:".green().bold(), "Available (Simulation Mode)".green());
        } else {
            println!("{} {}", "GPU:".green().bold(), "Not Available".red());
        }
        
        println!("{} {}", "Mode:".green().bold(), "Interactive REPL".white());
        println!();
    }

    fn print_help(&self) {
        println!("{}", "Available commands:".yellow().bold());
        println!("  {}  - Show this help message", ":help".cyan());
        println!("  {}  - Exit the REPL", ":exit".cyan());
        println!("  {}  - Clear the screen", ":clear".cyan());
        println!("  {}  - Show command history", ":history".cyan());
        println!("  {}  - Load and execute a Vortex file", ":load <file>".cyan());
        println!("  {}  - Show current environment variables", ":env".cyan());
        println!("  {}  - Reset the interpreter state", ":reset".cyan());
        println!();
        println!("{}", "Multi-line input:".yellow().bold());
        println!("  - Use {} or {} to start multi-line blocks", "if:".cyan(), "fn:".cyan());
        println!("  - Press {} to cancel multi-line input", "Ctrl+C".cyan());
        println!();
        println!("{}", "Enter Vortex code or use :help for commands".green());
        println!();
    }

    fn get_prompt(&self) -> String {
        if self.in_multi_line {
            format!("{} ", "...".yellow())
        } else {
            format!("{} ", "vortex>".green().bold())
        }
    }

    fn handle_command(&mut self, command: &str) -> bool {
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        if parts.is_empty() {
            return false;
        }

        match parts[0] {
            ":exit" | ":quit" | ":q" => {
                println!("{}", "Goodbye!".yellow());
                return true;
            }
            ":help" | ":h" => {
                self.print_help();
            }
            ":clear" | ":cls" => {
                print!("\x1B[2J\x1B[1;1H"); // Clear screen
                self.print_banner();
            }
            ":history" => {
                self.show_history();
            }
            ":load" => {
                if parts.len() < 2 {
                    println!("{} Usage: :load <filename>", "Error:".red());
                } else {
                    self.load_file(parts[1]);
                }
            }
            ":env" => {
                self.show_environment();
            }
            ":reset" => {
                self.interpreter = Interpreter::new();
                println!("{}", "Interpreter state reset.".green());
            }
            _ => {
                println!("{} Unknown command: {}", "Error:".red(), parts[0]);
                println!("Type {} for available commands.", ":help".cyan());
            }
        }
        false
    }

    fn handle_input(&mut self, input: &str) {
        let line = input.trim();
        
        // Check if this starts or continues a multi-line block
        if self.should_start_multi_line(line) || self.in_multi_line {
            self.handle_multi_line_input(line);
            return;
        }

        // Single line input
        self.execute_code(line);
    }

    fn should_start_multi_line(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.ends_with(':') && (
            trimmed.starts_with("if ") ||
            trimmed.starts_with("then ") ||
            trimmed.starts_with("else:") ||
            trimmed.starts_with("for ") ||
            trimmed.starts_with("parallel ") ||
            trimmed.starts_with("fn ") ||
            trimmed.starts_with("@gpu fn ") ||
            trimmed.starts_with("branch ") ||
            trimmed.starts_with("fallback")
        )
    }

    fn handle_multi_line_input(&mut self, line: &str) {
        if !self.in_multi_line {
            self.in_multi_line = true;
            self.multi_line_buffer.clear();
            self.bracket_depth = 0;
        }

        // Add the line to buffer
        if !self.multi_line_buffer.is_empty() {
            self.multi_line_buffer.push('\n');
        }
        self.multi_line_buffer.push_str(line);

        // Simple heuristic: if line is empty or doesn't start with whitespace, 
        // and we're not in a nested block, consider the multi-line input complete
        if line.trim().is_empty() || 
           (!line.starts_with(' ') && !line.starts_with('\t') && self.bracket_depth <= 0) {
            
            // Check if we have a complete block
            if self.is_complete_block() {
                let code = self.multi_line_buffer.clone();
                self.reset_multi_line();
                self.execute_code(&code);
            }
        }
    }

    fn is_complete_block(&self) -> bool {
        // Simple check: if buffer contains keywords that typically end blocks
        let buffer = self.multi_line_buffer.trim();
        
        // Check for function definitions
        if buffer.contains("fn ") && buffer.contains("return") {
            return true;
        }
        
        // Check for control structures with bodies
        if (buffer.contains("if ") || buffer.contains("for ") || buffer.contains("parallel ")) 
           && !buffer.trim().ends_with(':') {
            return true;
        }
        
        // Check for single-line statements that end with colon but have content
        if buffer.lines().count() > 1 {
            let last_line = buffer.lines().last().unwrap_or("").trim();
            return !last_line.is_empty() && !last_line.ends_with(':');
        }
        
        false
    }

    fn reset_multi_line(&mut self) {
        self.in_multi_line = false;
        self.multi_line_buffer.clear();
        self.bracket_depth = 0;
    }

    fn execute_code(&mut self, code: &str) {
        if code.trim().is_empty() {
            return;
        }

        // Tokenize
        let mut lexer = Lexer::new(code);
        let mut tokens = Vec::new();

        loop {
            let token = lexer.next_token();
            if token == Token::EOF {
                break;
            }
            tokens.push(token);
        }

        if tokens.is_empty() {
            return;
        }

        // Parse
        let mut parser = Parser::new(tokens);
        let statements = parser.parse();

        if statements.is_empty() {
            println!("{} No valid statements found.", "Warning:".yellow());
            return;
        }

        // Execute each statement
        for stmt in statements {
            match self.interpreter.interpret(vec![stmt]) {
                Ok(_) => {
                    // Success - no output needed unless it's an expression
                }
                Err(e) => {
                    println!("{} {}", "Runtime Error:".red(), e);
                }
            }
        }
    }

    fn show_history(&self) {
        println!("{}", "Command History:".yellow().bold());
        for (i, entry) in self.editor.history().iter().enumerate() {
            println!("  {}: {}", format!("{:3}", i + 1).cyan(), entry);
        }
    }

    fn load_file(&mut self, filename: &str) {
        let path = Path::new(filename);
        
        match fs::read_to_string(path) {
            Ok(content) => {
                println!("{} Loading file: {}", "Info:".blue(), filename.cyan());
                self.execute_code(&content);
                println!("{} File executed successfully.", "Success:".green());
            }
            Err(e) => {
                println!("{} Failed to load file '{}': {}", "Error:".red(), filename, e);
            }
        }
    }

    fn show_environment(&self) {
        println!("{}", "Current Environment:".yellow().bold());
        // This would require exposing the environment from the interpreter
        // For now, just show a placeholder
        println!("  {} Environment inspection not yet implemented", "Info:".blue());
        println!("  {} Variables and functions are stored internally", "Note:".yellow());
    }
}

pub fn start_repl() -> Result<()> {
    let mut repl = VortexRepl::new()?;
    repl.run()
}