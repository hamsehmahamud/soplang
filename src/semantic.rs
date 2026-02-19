//! Semantic analysis: name resolution, variable slots, function/class metadata.
//! Phase 1 of COMPILER_PLAN. Consumes AST, produces SymbolTable for HIR lowering.

use std::collections::HashMap;

use crate::ast::{Expr, Stmt, TypeAnnotation};
use crate::error::{type_error, SoplangError};

/// Symbol table built by semantic analysis. Used by HIR lowering.
#[derive(Debug, Default)]
pub struct SymbolTable {
    /// Stack of scopes (inner to outer). Scope 0 = global.
    pub scopes:    Vec<Scope>,
    /// Per-function metadata (params, local count, captures).
    pub functions: Vec<FunctionMeta>,
    /// Class name -> metadata (methods, parent).
    pub classes:   HashMap<String, ClassMeta>,
}

#[derive(Debug, Default, Clone)]
pub struct Scope {
    pub vars: HashMap<String, VarInfo>,
}

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub slot:        usize,
    pub type_ann:    TypeAnnotation,
    pub is_const:    bool,
    pub is_captured: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionMeta {
    pub name:        String,
    pub param_slots: Vec<usize>,
    pub local_count: usize,
    pub captures:    Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ClassMeta {
    pub name:   String,
    pub parent: Option<String>,
    /// Method names (order preserved for stable slot/index).
    pub methods: Vec<String>,
}

/// Run semantic analysis on the AST. Returns a symbol table or an error.
pub fn analyze(stmts: &[Stmt]) -> Result<SymbolTable, SoplangError> {
    let mut sym = SymbolTable::default();
    sym.scopes.push(Scope::default()); // global scope
    analyze_block(stmts, &mut sym)?;
    Ok(sym)
}

fn analyze_block(stmts: &[Stmt], sym: &mut SymbolTable) -> Result<(), SoplangError> {
    for s in stmts {
        analyze_stmt(s, sym)?;
    }
    Ok(())
}

fn analyze_stmt(stmt: &Stmt, sym: &mut SymbolTable) -> Result<(), SoplangError> {
    match stmt {
        Stmt::VarDecl { name, type_ann, is_const, value, line, col } => {
            let scope = sym.scopes.last_mut().unwrap();
            if scope.vars.contains_key(name) {
                return Err(type_error(
                    format!("Magaca '{}' waa la qoray horay", name),
                    *line,
                    *col,
                ));
            }
            let slot = scope.vars.len();
            scope.vars.insert(
                name.clone(),
                VarInfo {
                    slot,
                    type_ann: *type_ann,
                    is_const: *is_const,
                    is_captured: false,
                },
            );
            // Optionally validate initializer type (e.g. abn x = "hi" -> error). For now we skip.
            analyze_expr(value, sym)?;
        }
        Stmt::Assign { target, value, line, col } => {
            if let Expr::Identifier(name) = target {
                resolve_name(sym, &name).ok_or_else(|| {
                    type_error(
                        format!("Magaca '{}' ma aqoonsan", name),
                        *line,
                        *col,
                    )
                })?;
            } else {
                // Index or Property: resolve subexpressions only
                analyze_assign_target(target, sym)?;
            }
            analyze_expr(value, sym)?;
        }
        Stmt::FuncDef { name, params, body } => {
            // New scope for function body
            let mut func_scope = Scope::default();
            let param_slots: Vec<usize> = (0..params.len()).collect();
            for (i, p) in params.iter().enumerate() {
                func_scope.vars.insert(
                    p.name.clone(),
                    VarInfo {
                        slot: i,
                        type_ann: TypeAnnotation::Dynamic,
                        is_const: false,
                        is_captured: false,
                    },
                );
            }
            sym.scopes.push(func_scope);
            analyze_block(body, sym)?;
            let scope = sym.scopes.pop().unwrap();
            let local_count = scope.vars.len();
            sym.functions.push(FunctionMeta {
                name:   name.clone(),
                param_slots,
                local_count,
                captures: Vec::new(), // TODO: closure analysis
            });
        }
        Stmt::ClassDef { name, parent, body } => {
            let mut methods = Vec::new();
            for s in body {
                if let Stmt::FuncDef { name: mname, .. } = s {
                    methods.push(mname.clone());
                }
            }
            sym.classes.insert(
                name.clone(),
                ClassMeta {
                    name:   name.clone(),
                    parent: parent.clone(),
                    methods,
                },
            );
            // Recurse into class body (method bodies get their own scope via FuncDef)
            analyze_block(body, sym)?;
        }
        Stmt::If { cond, then_body, elseifs, else_body } => {
            analyze_expr(cond, sym)?;
            analyze_block(then_body, sym)?;
            for (c, b) in elseifs {
                analyze_expr(c, sym)?;
                analyze_block(b, sym)?;
            }
            if let Some(eb) = else_body {
                analyze_block(eb, sym)?;
            }
        }
        Stmt::Switch { expr, cases, default } => {
            analyze_expr(expr, sym)?;
            for (v, body) in cases {
                analyze_expr(v, sym)?;
                analyze_block(body, sym)?;
            }
            if let Some(d) = default {
                analyze_block(d, sym)?;
            }
        }
        Stmt::For { start, end, step, body, .. } => {
            analyze_expr(start, sym)?;
            analyze_expr(end, sym)?;
            if let Some(s) = step {
                analyze_expr(s, sym)?;
            }
            analyze_block(body, sym)?;
        }
        Stmt::While { cond, body } => {
            analyze_expr(cond, sym)?;
            analyze_block(body, sym)?;
        }
        Stmt::Return(Some(e)) => analyze_expr(e, sym)?,
        Stmt::Return(None) => {}
        Stmt::Break | Stmt::Continue => {}
        Stmt::TryCatch { try_body, catch_body, .. } => {
            analyze_block(try_body, sym)?;
            analyze_block(catch_body, sym)?;
        }
        Stmt::Import(_) => {}
        Stmt::Block(stmts) => analyze_block(stmts, sym)?,
        Stmt::Expr(e) => analyze_expr(e, sym)?,
    }
    Ok(())
}

fn analyze_expr(expr: &Expr, sym: &mut SymbolTable) -> Result<(), SoplangError> {
    match expr {
        Expr::Literal(_) => {}
        Expr::Identifier(_) => {}
        Expr::BinaryOp { left, right, .. } => {
            analyze_expr(left, sym)?;
            analyze_expr(right, sym)?;
        }
        Expr::UnaryOp { expr: e, .. } => analyze_expr(e, sym)?,
        Expr::Call { args, .. } => {
            for a in args {
                analyze_expr(a, sym)?;
            }
        }
        Expr::MethodCall { obj, args, .. } => {
            analyze_expr(obj, sym)?;
            for a in args {
                analyze_expr(a, sym)?;
            }
        }
        Expr::Index { obj, idx } => {
            analyze_expr(obj, sym)?;
            analyze_expr(idx, sym)?;
        }
        Expr::Property { obj, .. } => analyze_expr(obj, sym)?,
        Expr::List(exprs) => {
            for e in exprs {
                analyze_expr(e, sym)?;
            }
        }
        Expr::Object(pairs) => {
            for (_, e) in pairs {
                analyze_expr(e, sym)?;
            }
        }
    }
    Ok(())
}

fn analyze_assign_target(target: &Expr, sym: &mut SymbolTable) -> Result<(), SoplangError> {
    match target {
        Expr::Index { obj, idx } => {
            analyze_expr(obj, sym)?;
            analyze_expr(idx, sym)?;
        }
        Expr::Property { obj, .. } => analyze_expr(obj, sym)?,
        _ => {}
    }
    Ok(())
}

/// Resolve a name to its VarInfo (from current or outer scope). Returns None if not found.
pub fn resolve_name(sym: &SymbolTable, name: &str) -> Option<VarInfo> {
    for scope in sym.scopes.iter().rev() {
        if let Some(info) = scope.vars.get(name) {
            return Some(info.clone());
        }
    }
    None
}
