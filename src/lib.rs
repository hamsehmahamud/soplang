//! Soplang library for use by the binary and integration tests.

pub mod ast;
pub mod backend;
pub mod error;
pub mod hir;
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
pub use semantic::{
    analyze, analyze_with_options, AnalyzeOptions, ClassMeta, FunctionMeta, Scope, SymbolTable,
    VarInfo,
};
pub use error::{format_error_with_source, SoplangError};
pub use lexer::Lexer;
pub use parser::Parser;
pub use token::{Token, TokenType};
pub use value::Value;

use std::path::Path;

/// Run source code through the compiled pipeline (semantic → HIR → Cranelift JIT).
/// Used by the CLI and tests. Interpreter is no longer used in Phase 6.
pub fn run_source(
    source: &str,
    _path: Option<&Path>,
    print_ast: bool,
    dump_hir: bool,
    strict: bool,
) -> Result<(), SoplangError> {
    let tokens = Lexer::new(source).tokenize()?;
    let stmts = Parser::new(tokens).parse()?;
    if print_ast {
        for s in &stmts {
            print!("{}", s);
        }
        return Ok(());
    }
    let sym = semantic::analyze_with_options(&stmts, semantic::AnalyzeOptions { strict })?;
    let hir = HirLowering::lower(&sym, &stmts);
    if dump_hir {
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
    // Phase 6: always run via Cranelift JIT.
    let mut backend = backend::cranelift::CraneliftBackend::new()?;
    backend.compile_module(&hir)?;
    backend.run_main()
}

/// Phase 5: build a standalone executable (AOT path).
pub fn build_source(
    source: &str,
    out_path: &Path,
    opt_level: u8,
    strict: bool,
) -> Result<(), SoplangError> {
    let tokens = Lexer::new(source).tokenize()?;
    let stmts = Parser::new(tokens).parse()?;
    let sym = semantic::analyze_with_options(&stmts, semantic::AnalyzeOptions { strict })?;
    let _hir = HirLowering::lower(&sym, &stmts);
    let backend = backend::llvm::LlvmBackend::new();
    backend.build_executable(source, out_path, opt_level, strict)
}
