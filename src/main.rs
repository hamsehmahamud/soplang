mod ast;
mod error;
mod lexer;
mod parser;
mod token;

use std::env;
use std::fs;
use std::process;

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
        for t in tokens {
            println!("{}", t);
        }
    }
}
