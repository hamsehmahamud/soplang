//! Interactive REPL (Phase 6).

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use soplang::{format_error_with_source, Interpreter, Lexer, Parser};

const PROMPT: &str = "soplang> ";

pub struct Shell {
    interpreter: Interpreter,
    editor:      DefaultEditor,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
            editor:      DefaultEditor::new().expect("rustyline Editor"),
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.editor.readline(PROMPT) {
                Ok(line) => {
                    let line = line.trim();
                    if !line.is_empty() {
                        let _ = self.editor.add_history_entry(line);
                        self.execute(line);
                    }
                }
                Err(ReadlineError::Interrupted | ReadlineError::Eof) => break,
                Err(e) => eprintln!("Khalad: {}", e),
            }
        }
    }

    fn execute(&mut self, source: &str) {
        let tokens = match Lexer::new(source).tokenize() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{}", format_error_with_source(&e, Some(source)));
                return;
            }
        };
        let stmts = match Parser::new(tokens).parse() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}", format_error_with_source(&e, Some(source)));
                return;
            }
        };
        if let Err(e) = self.interpreter.run(stmts) {
            eprintln!("{}", format_error_with_source(&e, Some(source)));
        }
    }
}
