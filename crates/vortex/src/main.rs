mod ast;
mod codegen_ic;
mod gpu_runtime;
mod interpreter;
mod lexer;
mod parser;
mod repl;
mod token;

use std::fs;
use std::path::PathBuf;
use std::process;

use clap::{Parser as ClapParser, Subcommand};

use codegen_ic::Codegen;
use interpreter::{Interpreter, Value};
use lexer::Lexer;
use parser::Parser;
use token::Token;

#[derive(ClapParser, Debug)]
#[command(name = "nexus", version, about = "Nexus language CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    Run {
        file: PathBuf,
        #[arg(long, help = "Execute through the IC pipeline (VORTEX -> VICE)")]
        ic: bool,
        #[arg(long, help = "Print reduction statistics for the IC backend")]
        stats: bool,
    },
    Net {
        file: PathBuf,
    },
    Check {
        file: PathBuf,
    },
    Repl,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Command::Run { file, ic, stats }) => run_file(&file, ic, stats),
        Some(Command::Net { file }) => dump_net(&file),
        Some(Command::Check { file }) => check_file(&file),
        Some(Command::Repl) | None => repl::start_repl().map_err(|e| e.to_string()),
    };

    if let Err(err) = result {
        eprintln!("Error: {err}");
        process::exit(1);
    }
}

fn run_file(path: &PathBuf, ic: bool, stats: bool) -> Result<(), String> {
    let source = fs::read_to_string(path).map_err(|e| format!("cannot read file: {e}"))?;
    let program = parse_source(&source);

    if ic {
        let mut codegen = Codegen::new();
        codegen.compile_program(&program);
        let output = vice::reduce_ir_with_stats(codegen.net);

        println!("{}", output.value);
        if stats {
            println!(
                "reductions: {}, time_ms: {}",
                output.stats.reductions, output.stats.elapsed_ms
            );
        }
        return Ok(());
    }

    let mut interpreter = Interpreter::new();
    let value = interpreter.interpret_with_result(&program)?;
    if let Some(value) = value {
        if !matches!(value, Value::Nil) {
            println!("{value}");
        }
    }

    Ok(())
}

fn dump_net(path: &PathBuf) -> Result<(), String> {
    let source = fs::read_to_string(path).map_err(|e| format!("cannot read file: {e}"))?;
    let program = parse_source(&source);
    let mut codegen = Codegen::new();
    codegen.compile_program(&program);
    println!("{:#?}", codegen.net);
    Ok(())
}

fn check_file(path: &PathBuf) -> Result<(), String> {
    let source = fs::read_to_string(path).map_err(|e| format!("cannot read file: {e}"))?;
    let program = parse_source(&source);
    println!("ok: parsed {} statement(s)", program.len());
    Ok(())
}

fn parse_source(source: &str) -> Vec<ast::Stmt> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        if token == Token::EOF {
            break;
        }
        tokens.push(token);
    }

    let mut parser = Parser::new(tokens);
    parser.parse()
}
