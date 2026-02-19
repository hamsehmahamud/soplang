//! Soplang library for use by the binary and integration tests.

pub mod ast;
pub mod error;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod scope;
pub mod semantic;
pub mod stdlib;
pub mod token;
pub mod value;

pub use ast::{Expr, Literal, Param, Stmt, TypeAnnotation};
pub use semantic::{analyze, ClassMeta, FunctionMeta, Scope, SymbolTable, VarInfo};
pub use error::{format_error_with_source, SoplangError};
pub use interpreter::Interpreter;
pub use lexer::Lexer;
pub use parser::Parser;
pub use token::{Token, TokenType};
pub use value::Value;

use std::path::Path;

/// Run source code (lex, parse, execute). Used by main and tests.
pub fn run_source(
    interp: &mut Interpreter,
    source: &str,
    path: Option<&Path>,
    print_ast: bool,
) -> Result<(), SoplangError> {
    let tokens = Lexer::new(source).tokenize()?;
    let stmts = Parser::new(tokens).parse()?;
    if print_ast {
        for s in &stmts {
            print!("{}", s);
        }
        return Ok(());
    }
    interp.run_with_path(stmts, path)
}
