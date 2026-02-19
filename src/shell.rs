//! Interactive REPL (Phase 6).

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use soplang::{format_error_with_source, run_source};

const PROMPT: &str = "soplang> ";

pub struct Shell {
    editor: DefaultEditor,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            editor: DefaultEditor::new().expect("rustyline Editor"),
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
        if let Err(e) = run_source(source, None, false, false, true) {
            eprintln!("{}", format_error_with_source(&e, Some(source)));
        }
    }
}
