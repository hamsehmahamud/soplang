//! Typed AST for Soplang. Replaces Python's generic ASTNode with Expr/Stmt enums.
//! Matches psrc/core/ast.py and IMPLEMENTATION_PLAN Phase 2.

use std::fmt;

/// Static type annotation (abn, jajab, qoraal, bool, teed, walax) or dynamic (door/madoor).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeAnnotation {
    Abn,
    Jajab,
    Qoraal,
    Bool,
    Teed,
    Walax,
    Dynamic,
}

/// Literal values at parse time.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Null,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Int(n) => write!(f, "{}", n),
            Literal::Float(x) => write!(f, "{}", x),
            Literal::Str(s) => write!(f, "{:?}", s),
            Literal::Bool(b) => write!(f, "{}", if *b { "run" } else { "been" }),
            Literal::Null => write!(f, "null"),
        }
    }
}

/// Expressions (produce values).
#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    BinaryOp {
        op:   String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op:   String,
        expr: Box<Expr>,
    },
    /// Function call: name(args)
    Call {
        name: String,
        args: Vec<Expr>,
    },
    /// obj.method(args)
    MethodCall {
        obj:    Box<Expr>,
        method: String,
        args:   Vec<Expr>,
    },
    /// arr[idx]
    Index {
        obj: Box<Expr>,
        idx: Box<Expr>,
    },
    /// obj.prop
    Property {
        obj:  Box<Expr>,
        prop: String,
    },
    List(Vec<Expr>),
    Object(Vec<(String, Expr)>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(l) => write!(f, "{}", l),
            Expr::Identifier(s) => write!(f, "{}", s),
            Expr::BinaryOp { op, left, right } => {
                write!(f, "({} {} {})", left, op, right)
            }
            Expr::UnaryOp { op, expr } => write!(f, "({} {})", op, expr),
            Expr::Call { name, args } => {
                write!(f, "{}(", name)?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", a)?;
                }
                write!(f, ")")
            }
            Expr::MethodCall { obj, method, args } => {
                write!(f, "({}).{}(", obj, method)?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", a)?;
                }
                write!(f, ")")
            }
            Expr::Index { obj, idx } => write!(f, "{}[{}]", obj, idx),
            Expr::Property { obj, prop } => write!(f, "{}.{}", obj, prop),
            Expr::List(es) => {
                write!(f, "[")?;
                for (i, e) in es.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, "]")
            }
            Expr::Object(pairs) => {
                write!(f, "{{")?;
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
        }
    }
}

/// Function parameter (name only for now).
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
}

/// Statements (side effects; optional line for errors).
#[derive(Debug, Clone)]
pub enum Stmt {
    VarDecl {
        name:     String,
        type_ann: TypeAnnotation,
        is_const: bool,
        value:    Expr,
        #[allow(dead_code)] // Phase 3 error reporting
        line:     usize,
        #[allow(dead_code)]
        col:      usize,
    },
    Assign {
        target: Expr,
        value:  Expr,
        #[allow(dead_code)] // Phase 3 error reporting
        line:   usize,
        #[allow(dead_code)]
        col:    usize,
    },
    FuncDef {
        name:   String,
        params: Vec<Param>,
        body:   Vec<Stmt>,
    },
    ClassDef {
        name:   String,
        parent: Option<String>,
        body:   Vec<Stmt>,
    },
    If {
        cond:      Expr,
        then_body: Vec<Stmt>,
        elseifs:   Vec<(Expr, Vec<Stmt>)>,
        else_body: Option<Vec<Stmt>>,
    },
    Switch {
        expr:   Expr,
        cases:  Vec<(Expr, Vec<Stmt>)>,
        default: Option<Vec<Stmt>>,
    },
    For {
        var:   String,
        start: Expr,
        end:   Expr,
        step:  Option<Expr>,
        body:  Vec<Stmt>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    Break,
    Continue,
    TryCatch {
        try_body:   Vec<Stmt>,
        err_var:   String,
        catch_body: Vec<Stmt>,
    },
    Import(String),
    Block(Vec<Stmt>),
    Expr(Expr),
}

impl Stmt {
    /// Pretty-print with indentation (for debug AST output).
    pub fn fmt_with_depth(&self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        const INDENT: &str = "  ";
        let pad = INDENT.repeat(depth);
        let pad_inner = INDENT.repeat(depth + 1);
        match self {
            Stmt::VarDecl { name, type_ann, is_const, value, .. } => {
                let ty = match type_ann {
                    TypeAnnotation::Dynamic => if *is_const { "madoor" } else { "door" }.to_string(),
                    TypeAnnotation::Abn => "abn".to_string(),
                    TypeAnnotation::Jajab => "jajab".to_string(),
                    TypeAnnotation::Qoraal => "qoraal".to_string(),
                    TypeAnnotation::Bool => "bool".to_string(),
                    TypeAnnotation::Teed => "teed".to_string(),
                    TypeAnnotation::Walax => "walax".to_string(),
                };
                writeln!(f, "{}{} {} = {};", pad, ty, name, value)
            }
            Stmt::Assign { target, value, .. } => writeln!(f, "{}{} = {};", pad, target, value),
            Stmt::FuncDef { name, params, body } => {
                write!(f, "{}hawl {}(", pad, name)?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p.name)?;
                }
                writeln!(f, ") {{")?;
                for s in body {
                    s.fmt_with_depth(f, depth + 1)?;
                }
                writeln!(f, "{}}}", pad)
            }
            Stmt::ClassDef { name, parent, body } => {
                if let Some(p) = parent {
                    writeln!(f, "{}fasalka {} ka_dhaxal {} {{", pad, name, p)?;
                } else {
                    writeln!(f, "{}fasalka {} {{", pad, name)?;
                }
                for s in body {
                    s.fmt_with_depth(f, depth + 1)?;
                }
                writeln!(f, "{}}}", pad)
            }
            Stmt::If { cond, then_body, elseifs, else_body } => {
                writeln!(f, "{}haddii ({}) {{", pad, cond)?;
                for s in then_body {
                    s.fmt_with_depth(f, depth + 1)?;
                }
                for (c, b) in elseifs {
                    writeln!(f, "{}haddii_kale ({}) {{", pad, c)?;
                    for s in b {
                        s.fmt_with_depth(f, depth + 2)?;
                    }
                    writeln!(f, "{}}}", pad)?;
                }
                if let Some(eb) = else_body {
                    writeln!(f, "{}ugudambeyn {{", pad)?;
                    for s in eb {
                        s.fmt_with_depth(f, depth + 1)?;
                    }
                    writeln!(f, "{}}}", pad)?;
                }
                Ok(())
            }
            Stmt::Switch { expr, cases, default } => {
                writeln!(f, "{}dooro ({}) {{", pad, expr)?;
                for (v, stmts) in cases {
                    writeln!(f, "{}xaalad {} {{", pad_inner, v)?;
                    for s in stmts {
                        s.fmt_with_depth(f, depth + 2)?;
                    }
                    writeln!(f, "{}}}", pad_inner)?;
                }
                if let Some(d) = default {
                    writeln!(f, "{}ugudambeyn {{", pad_inner)?;
                    for s in d {
                        s.fmt_with_depth(f, depth + 2)?;
                    }
                    writeln!(f, "{}}}", pad_inner)?;
                }
                writeln!(f, "{}}}", pad)
            }
            Stmt::For { var, start, end, step, body } => {
                write!(f, "{}kuceli ({} {} ilaa {}", pad, var, start, end)?;
                if let Some(s) = step {
                    write!(f, " :: {}", s)?;
                }
                writeln!(f, ") {{")?;
                for s in body {
                    s.fmt_with_depth(f, depth + 1)?;
                }
                writeln!(f, "{}}}", pad)
            }
            Stmt::While { cond, body } => {
                writeln!(f, "{}intay ({}) {{", pad, cond)?;
                for s in body {
                    s.fmt_with_depth(f, depth + 1)?;
                }
                writeln!(f, "{}}}", pad)
            }
            Stmt::Return(Some(e)) => writeln!(f, "{}celi {};", pad, e),
            Stmt::Return(None) => writeln!(f, "{}celi;", pad),
            Stmt::Break => writeln!(f, "{}jooji;", pad),
            Stmt::Continue => writeln!(f, "{}soco;", pad),
            Stmt::TryCatch { try_body, err_var, catch_body } => {
                writeln!(f, "{}isku_day {{", pad)?;
                for s in try_body {
                    s.fmt_with_depth(f, depth + 1)?;
                }
                writeln!(f, "{}}} qabo ({}) {{", pad, err_var)?;
                for s in catch_body {
                    s.fmt_with_depth(f, depth + 1)?;
                }
                writeln!(f, "{}}}", pad)
            }
            Stmt::Import(path) => writeln!(f, "{}ka_keen {:?};", pad, path),
            Stmt::Block(stmts) => {
                writeln!(f, "{} {{", pad)?;
                for s in stmts {
                    s.fmt_with_depth(f, depth + 1)?;
                }
                writeln!(f, "{}}}", pad)
            }
            Stmt::Expr(e) => writeln!(f, "{}{};", pad, e),
        }
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_depth(f, 0)
    }
}
