mod error;
mod lexer;
mod token;

use std::env;
use std::fs;
use std::process;

use lexer::Lexer;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: soplang <file.sop>");
        process::exit(1);
    }
    let path = &args[1];
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Khalad: Ma akhriyin faylka '{}': {}", path, e);
            process::exit(1);
        }
    };
    let mut lexer = Lexer::new(&source);
    match lexer.tokenize() {
        Ok(tokens) => {
            for t in tokens {
                println!("{:?}", t);
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}
