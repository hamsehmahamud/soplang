//! Semantic analysis: name resolution, types, symbol table.

pub mod analyze;
pub mod scope;

pub use analyze::{
    analyze, analyze_with_options, AnalyzeOptions, ClassMeta, FunctionMeta, resolve_name, Scope,
    SymbolTable, VarInfo,
};
pub use scope::Env;
