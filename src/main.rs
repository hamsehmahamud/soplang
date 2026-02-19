mod ast;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod scope;
mod token;
mod value;

use std::env;
use std::fs;
use std::path::Path;
use std::process;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: soplang <file.sop> [--ast]");
        process::exit(1);
    }
    let path = &args[1];
    let print_ast = args.iter().any(|a| a == "--ast");
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Khalad: Ma akhriyin faylka '{}': {}", path, e);
            process::exit(1);
        }
    };

    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    if print_ast {
        let mut parser = Parser::new(tokens);
        match parser.parse() {
            Ok(stmts) => {
                for s in stmts {
                    print!("{}", s);
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
    } else {
        let mut parser = Parser::new(tokens);
        let stmts = match parser.parse() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        };
        let mut interp = Interpreter::new();
        if let Err(e) = interp.run_with_path(stmts, Some(Path::new(path))) {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}
