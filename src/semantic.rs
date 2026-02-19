//! Semantic analysis: name resolution, variable slots, function/class metadata.
//! Phase 1 of COMPILER_PLAN. Consumes AST, produces SymbolTable for HIR lowering.

use std::collections::HashMap;

use crate::ast::{Expr, Literal, Stmt, TypeAnnotation};
use crate::error::{codes, type_error_ex, ErrorMeta, SoplangError};

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
    /// Variable name -> VarInfo for this function's scope (params + locals).
    pub scope_vars:  HashMap<String, VarInfo>,
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
                return Err(type_error_ex(
                    format!("Magaca '{}' waa la qoray horay", name),
                    *line,
                    *col,
                    ErrorMeta::default()
                        .with_code(codes::E020_REDECLARED)
                        .with_hint("Magacan ayaa horay loogu isticmaalay. Isticmaal magac kale oo gaar ah."),
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
            // Phase 2: basic static type check for initializer when we can infer a type.
            if *type_ann != TypeAnnotation::Dynamic {
                if let Some(expr_ty) = infer_expr_type(value, sym) {
                    if !is_type_compatible(*type_ann, expr_ty) {
                        return Err(type_mismatch_error(
                            name,
                            *type_ann,
                            expr_ty,
                            *line,
                            *col,
                        ));
                    }
                }
            }
            analyze_expr(value, sym)?;
        }
        Stmt::Assign { target, value, line, col } => {
            if let Expr::Identifier(name) = target {
                let var = resolve_name(sym, &name, None).ok_or_else(|| {
                    type_error_ex(
                        format!("Magaca '{}' ma aqoonsan", name),
                        *line,
                        *col,
                        ErrorMeta::default()
                            .with_code(codes::E021_UNDECLARED)
                            .with_hint("Doorsame cusub ku qeex 'door' ama 'madoor' ka hor intaadan isticmaalin."),
                    )
                })?;
                if var.is_const {
                    return Err(type_error_ex(
                        format!("Ma bedeli kartid qiimaha doorsamaha madoor '{}'", name),
                        *line,
                        *col,
                        ErrorMeta::default()
                            .with_code(codes::E020_REDECLARED)
                            .with_hint("Doorsamaha madoor ma beddeli karaan qiimahooda. Isticmaal 'door' haddii aad rabto mid beddeli kara."),
                    ));
                }
                if var.type_ann != TypeAnnotation::Dynamic {
                    if let Some(expr_ty) = infer_expr_type(value, sym) {
                        if !is_type_compatible(var.type_ann, expr_ty) {
                            return Err(type_mismatch_error(
                                name,
                                var.type_ann,
                                expr_ty,
                                *line,
                                *col,
                            ));
                        }
                    }
                }
            } else {
                // Index or Property: resolve subexpressions only
                analyze_assign_target(target, sym)?;
            }
            analyze_expr(value, sym)?;
        }
        Stmt::FuncDef { name, params, body } => {
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
                name:       name.clone(),
                param_slots,
                local_count,
                captures:   Vec::new(),
                scope_vars: scope.vars,
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
        Stmt::For { var, start, end, step, body } => {
            // Loop variable is in scope for the body
            let scope = sym.scopes.last_mut().unwrap();
            if !scope.vars.contains_key(var) {
                let slot = scope.vars.len();
                scope.vars.insert(
                    var.clone(),
                    VarInfo {
                        slot,
                        type_ann: TypeAnnotation::Dynamic,
                        is_const: false,
                        is_captured: false,
                    },
                );
            }
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

// ---------------------------------------------------------------------------
// Simple type inference for expressions (used for VarDecl/Assign checks)
// ---------------------------------------------------------------------------

fn infer_expr_type(expr: &Expr, sym: &SymbolTable) -> Option<TypeAnnotation> {
    match expr {
        Expr::Literal(l) => Some(match l {
            Literal::Int(_) => TypeAnnotation::Abn,
            Literal::Float(_) => TypeAnnotation::Jajab,
            Literal::Str(_) => TypeAnnotation::Qoraal,
            Literal::Bool(_) => TypeAnnotation::Bool,
            Literal::Null => TypeAnnotation::Dynamic,
        }),
        Expr::Identifier(name) => resolve_name(sym, name, None).map(|v| v.type_ann),
        Expr::List(_) => Some(TypeAnnotation::Teed),
        Expr::Object(_) => Some(TypeAnnotation::Walax),
        Expr::UnaryOp { expr: inner, .. } => infer_expr_type(inner, sym),
        Expr::BinaryOp { op, left, right } => {
            let lt = infer_expr_type(left, sym)?;
            let rt = infer_expr_type(right, sym)?;
            match op.as_str() {
                "+" | "-" | "*" | "%" => {
                    // Numeric ops: if any side is Jajab → Jajab, else Abn
                    if matches!(lt, TypeAnnotation::Jajab) || matches!(rt, TypeAnnotation::Jajab) {
                        Some(TypeAnnotation::Jajab)
                    } else if matches!(lt, TypeAnnotation::Abn) && matches!(rt, TypeAnnotation::Abn) {
                        Some(TypeAnnotation::Abn)
                    } else {
                        None
                    }
                }
                "/" => {
                    // Soplang division returns decimal even for integer operands.
                    if matches!(lt, TypeAnnotation::Abn | TypeAnnotation::Jajab)
                        && matches!(rt, TypeAnnotation::Abn | TypeAnnotation::Jajab)
                    {
                        Some(TypeAnnotation::Jajab)
                    } else {
                        None
                    }
                }
                "==" | "!=" | "<" | "<=" | ">" | ">=" | "&&" | "||" => Some(TypeAnnotation::Bool),
                _ => None,
            }
        }
        Expr::Call { .. } | Expr::MethodCall { .. } | Expr::Index { .. } | Expr::Property { .. } => {
            // For now, calls/index/property are treated as dynamic (we can't easily know statically)
            None
        }
    }
}

fn is_type_compatible(target: TypeAnnotation, value: TypeAnnotation) -> bool {
    use TypeAnnotation::*;
    match target {
        Dynamic => true,
        // Interpreter allows assigning integral decimals into abn at runtime.
        // Keep semantic check permissive here to avoid false negatives.
        Abn => matches!(value, Abn | Jajab),
        Jajab => matches!(value, Abn | Jajab),
        Qoraal => matches!(value, Qoraal),
        Bool => matches!(value, Bool),
        Teed => matches!(value, Teed),
        Walax => matches!(value, Walax),
    }
}

fn type_mismatch_error(
    name: &str,
    expected: TypeAnnotation,
    found: TypeAnnotation,
    line: usize,
    col: usize,
) -> SoplangError {
    let msg = format!(
        "'{}' waa {} laakin qiimaheeda '{}' ma ahan {}",
        name,
        ty_str(expected),
        ty_str(found),
        ty_str(expected)
    );
    type_error_ex(
        msg,
        line,
        col,
        ErrorMeta::default()
            .with_code(codes::E022_TYPE_MISMATCH)
            .with_hint("Hubi in nooca qiimaha uu la mid yahay nooca lagu qeexay doorsamaha."),
    )
}

fn ty_str(a: TypeAnnotation) -> &'static str {
    match a {
        TypeAnnotation::Abn => "abn",
        TypeAnnotation::Jajab => "jajab",
        TypeAnnotation::Qoraal => "qoraal",
        TypeAnnotation::Bool => "bool",
        TypeAnnotation::Teed => "teed",
        TypeAnnotation::Walax => "walax",
        TypeAnnotation::Dynamic => "dynamic",
    }
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

/// Resolve a name to its VarInfo. If func_scope is given (for function body), check it first.
pub fn resolve_name(
    sym: &SymbolTable,
    name: &str,
    func_scope: Option<&HashMap<String, VarInfo>>,
) -> Option<VarInfo> {
    if let Some(scope) = func_scope {
        if let Some(info) = scope.get(name) {
            return Some(info.clone());
        }
    }
    for scope in sym.scopes.iter().rev() {
        if let Some(info) = scope.vars.get(name) {
            return Some(info.clone());
        }
    }
    None
}
