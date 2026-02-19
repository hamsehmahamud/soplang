//! Soplang library for use by the binary and integration tests.

pub mod ast;
pub mod backend;
pub mod error;
pub mod hir;
pub mod interpreter;
pub mod runtime;
pub mod lexer;
pub mod parser;
pub mod scope;
pub mod semantic;
pub mod stdlib;
pub mod token;
pub mod value;

pub use ast::{Expr, Literal, Param, Stmt, TypeAnnotation};
pub use hir::{HirFunction, HirInstr, HirModule, HirLowering};
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
    dump_hir: bool,
    jit: bool,
) -> Result<(), SoplangError> {
    let tokens = Lexer::new(source).tokenize()?;
    let stmts = Parser::new(tokens).parse()?;
    if print_ast {
        for s in &stmts {
            print!("{}", s);
        }
        return Ok(());
    }
    if dump_hir {
        let sym = analyze(&stmts)?;
        let hir = HirLowering::lower(&sym, &stmts);
        for f in &hir.functions {
            println!("\n--- {} ---", f.name);
            for instr in &f.body {
                println!("{}", instr);
            }
        }
        println!("\n--- top_level ---");
        for instr in &hir.top_level {
            println!("{}", instr);
        }
        return Ok(());
    }
    if jit {
        let sym = analyze(&stmts)?;
        let hir = HirLowering::lower(&sym, &stmts);
        let mut backend = backend::cranelift::CraneliftBackend::new()?;
        backend.compile_module(&hir)?;
        backend.run_main()?;
        return Ok(());
    }
    interp.run_with_path(stmts, path)
}
