//! Tree-walking interpreter. Matches psrc/runtime/interpreter.py; Phase 3: vars, control flow, qor.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{Expr, Literal, Stmt, TypeAnnotation};
use crate::scope::Env;
use crate::error::{runtime_error, type_error, SoplangError};
use crate::value::Value;

#[derive(Debug)]
pub enum Signal {
    None,
    Break,
    Continue,
    #[allow(dead_code)] // Phase 4: return value used by execute_function_call
    Return(Value),
}

pub struct Interpreter {
    pub globals: Rc<RefCell<Env>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            globals: Rc::new(RefCell::new(Env::new())),
        }
    }

    pub fn run(&mut self, stmts: Vec<Stmt>) -> Result<(), SoplangError> {
        for stmt in &stmts {
            let sig = self.exec_stmt(stmt, Rc::clone(&self.globals))?;
            match sig {
                Signal::Break => return Err(runtime_error("Jooji waa in ay ku jiraan xalqad", 0, 0)),
                Signal::Continue => return Err(runtime_error("soco waa in ay ku jiraan xalqad", 0, 0)),
                Signal::Return(_) => return Err(runtime_error("celi waa in ay ku jirto hawl", 0, 0)),
                Signal::None => {}
            }
        }
        Ok(())
    }

    fn exec_stmt(&mut self, stmt: &Stmt, env: Rc<RefCell<Env>>) -> Result<Signal, SoplangError> {
        match stmt {
            Stmt::VarDecl { name, type_ann, is_const, value, line, col } => {
                let val = self.eval_expr(value, Rc::clone(&env))?;
                self.validate_type(name, &val, type_ann, *line, *col)?;
                env.borrow_mut().define(name, val, *type_ann, *is_const);
                Ok(Signal::None)
            }
            Stmt::Assign { target, value, line, col } => {
                let val = self.eval_expr(value, Rc::clone(&env))?;
                self.exec_assign(target, val, &env, *line, *col)?;
                Ok(Signal::None)
            }
            Stmt::If { cond, then_body, elseifs, else_body } => {
                if self.eval_expr(cond, Rc::clone(&env))?.is_truthy() {
                    return self.exec_block(then_body, env);
                }
                for (c, body) in elseifs {
                    if self.eval_expr(c, Rc::clone(&env))?.is_truthy() {
                        return self.exec_block(body, env);
                    }
                }
                if let Some(eb) = else_body {
                    return self.exec_block(eb, env);
                }
                Ok(Signal::None)
            }
            Stmt::Switch { expr, cases, default } => {
                let v = self.eval_expr(expr, Rc::clone(&env))?;
                for (case_val, body) in cases {
                    let c = self.eval_expr(case_val, Rc::clone(&env))?;
                    if values_eq(&v, &c) {
                        return self.exec_block(body, env);
                    }
                }
                if let Some(d) = default {
                    return self.exec_block(d, env);
                }
                Ok(Signal::None)
            }
            Stmt::For { var, start, end, step, body } => {
                let start_v = self.eval_expr(start, Rc::clone(&env))?;
                let end_v = self.eval_expr(end, Rc::clone(&env))?;
                let step_v = step
                    .as_ref()
                    .map(|s| self.eval_expr(s, Rc::clone(&env)))
                    .transpose()?
                    .unwrap_or(Value::Int(1));
                let start_f = to_number(&start_v).ok_or_else(|| type_error("kuceli billowga waa inuu noqdaa tiro", 0, 0))?;
                let end_f = to_number(&end_v).ok_or_else(|| type_error("kuceli dhamaadka waa inuu noqdaa tiro", 0, 0))?;
                let step_f = to_number(&step_v).ok_or_else(|| type_error("kuceli tallaabada waa inay noqotaa tiro", 0, 0))?;
                let mut i = start_f;
                loop {
                    let done = if step_f > 0.0 { i > end_f } else if step_f < 0.0 { i < end_f } else { true };
                    if done {
                        break;
                    }
                    env.borrow_mut().define(var, Value::Float(i), TypeAnnotation::Dynamic, false);
                    let sig = self.exec_block(body, Rc::clone(&env))?;
                    match sig {
                        Signal::Break => break,
                        Signal::Continue => {}
                        Signal::Return(_) | Signal::None => {}
                    }
                    i += step_f;
                }
                Ok(Signal::None)
            }
            Stmt::While { cond, body } => {
                while self.eval_expr(cond, Rc::clone(&env))?.is_truthy() {
                    let sig = self.exec_block(body, Rc::clone(&env))?;
                    match sig {
                        Signal::Break => break,
                        Signal::Continue => continue,
                        Signal::Return(_) => return Ok(sig),
                        Signal::None => {}
                    }
                }
                Ok(Signal::None)
            }
            Stmt::Return(Some(e)) => {
                let v = self.eval_expr(e, env)?;
                Ok(Signal::Return(v))
            }
            Stmt::Return(None) => Ok(Signal::Return(Value::Null)),
            Stmt::Break => Ok(Signal::Break),
            Stmt::Continue => Ok(Signal::Continue),
            Stmt::Block(stmts) => self.exec_block(stmts, env),
            Stmt::Expr(e) => {
                self.eval_expr(e, env)?;
                Ok(Signal::None)
            }
            Stmt::FuncDef { .. } | Stmt::ClassDef { .. } | Stmt::TryCatch { .. } | Stmt::Import(_) => {
                Err(runtime_error("Phase 3: hawl/fasalka/import ma diyaar", 0, 0))
            }
        }
    }

    fn exec_block(&mut self, body: &[Stmt], env: Rc<RefCell<Env>>) -> Result<Signal, SoplangError> {
        for stmt in body {
            let sig = self.exec_stmt(stmt, Rc::clone(&env))?;
            if !matches!(sig, Signal::None) {
                return Ok(sig);
            }
        }
        Ok(Signal::None)
    }

    fn exec_assign(
        &mut self,
        target: &Expr,
        value: Value,
        env: &Rc<RefCell<Env>>,
        line: usize,
        col: usize,
    ) -> Result<(), SoplangError> {
        match target {
            Expr::Identifier(name) => {
                env.borrow_mut().assign(name, value, line, col)?;
                Ok(())
            }
            Expr::Index { obj, idx } => {
                let arr = self.eval_expr(obj, Rc::clone(env))?;
                let i = self.eval_expr(idx, Rc::clone(env))?;
                let idx_i = to_int_index(&arr, &i, line, col)?;
                if let Value::List(list) = arr {
                    let mut v = list.borrow_mut();
                    if idx_i < 0 || idx_i >= v.len() as i64 {
                        return Err(runtime_error(format!("Tirada fihris-ku waa ka baxsan xadka: {}", idx_i), line, col));
                    }
                    v[idx_i as usize] = value;
                    Ok(())
                } else {
                    Err(type_error("Ma bedeli karo qiimaha aan ahayn teed", line, col))
                }
            }
            Expr::Property { obj, prop } => {
                let o = self.eval_expr(obj, Rc::clone(env))?;
                if let Value::Object(map) = o {
                    map.borrow_mut().insert(prop.clone(), value);
                    Ok(())
                } else {
                    Err(type_error("Ma bedeli karo qiimaha aan ahayn walax", line, col))
                }
            }
            _ => Err(runtime_error("Qiimaha aan lagu qeexin karin", line, col)),
        }
    }

    fn eval_expr(&mut self, expr: &Expr, env: Rc<RefCell<Env>>) -> Result<Value, SoplangError> {
        match expr {
            Expr::Literal(l) => Ok(literal_to_value(l)),
            Expr::Identifier(name) => env
                .borrow()
                .get(name)
                .ok_or_else(|| runtime_error(format!("Doorsame aan la qeexin: '{}'", name), 0, 0)),
            Expr::BinaryOp { op, left, right } => {
                let l = self.eval_expr(left, Rc::clone(&env))?;
                let r = self.eval_expr(right, Rc::clone(&env))?;
                self.eval_binary(op, l, r, 0)
            }
            Expr::UnaryOp { op, expr: e } => {
                let v = self.eval_expr(e, env)?;
                self.eval_unary(op, v, 0)
            }
            Expr::Call { name, args } => {
                let evaled: Vec<Value> = args
                    .iter()
                    .map(|a| self.eval_expr(a, Rc::clone(&env)))
                    .collect::<Result<Vec<_>, _>>()?;
                if name == "qor" {
                    for (i, a) in evaled.iter().enumerate() {
                        if i > 0 {
                            print!(" ");
                        }
                        print!("{}", a);
                    }
                    println!();
                    return Ok(Value::Null);
                }
                Err(runtime_error(format!("Hawl aan la qeexin: '{}'", name), 0, 0))
            }
            Expr::MethodCall { .. } => Err(runtime_error("Phase 3: habka (method) ma diyaar", 0, 0)),
            Expr::Index { obj, idx } => {
                let arr = self.eval_expr(obj, Rc::clone(&env))?;
                let i = self.eval_expr(idx, env)?;
                index_value(&arr, &i, 0, 0)
            }
            Expr::Property { obj, prop } => {
                let o = self.eval_expr(obj, env)?;
                property_value(&o, prop, 0, 0)
            }
            Expr::List(exprs) => {
                let mut v = Vec::new();
                for e in exprs {
                    v.push(self.eval_expr(e, Rc::clone(&env))?);
                }
                Ok(Value::List(Rc::new(RefCell::new(v))))
            }
            Expr::Object(pairs) => {
                let mut m = HashMap::new();
                for (k, e) in pairs {
                    m.insert(k.clone(), self.eval_expr(e, Rc::clone(&env))?);
                }
                Ok(Value::Object(Rc::new(RefCell::new(m))))
            }
        }
    }

    fn eval_binary(&self, op: &str, l: Value, r: Value, _line: usize) -> Result<Value, SoplangError> {
        match op {
            "+" => {
                if matches!(&l, Value::Str(_)) || matches!(&r, Value::Str(_)) {
                    return Ok(Value::Str(format!("{}{}", value_to_str(&l), value_to_str(&r))));
                }
                match (&l, &r) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                    (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
                    (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
                    _ => Err(type_error("Ma isticmaali karo '+' oo ku shaqeeya noocyadaan", 0, 0)),
                }
            }
            "-" => binary_num(op, l, r, |a, b| a - b, |a, b| a - b),
            "*" => binary_num(op, l, r, |a, b| a * b, |a, b| a * b),
            "/" => {
                let (_, rn) = to_two_numbers(&l, &r).ok_or_else(|| type_error("Qeybinta waa in ay noqoto tiro", 0, 0))?;
                if rn == 0.0 {
                    return Err(runtime_error("Ma suurtogali karto qeybinta eber", 0, 0));
                }
                let (ln, rn) = to_two_numbers(&l, &r).unwrap();
                Ok(Value::Float(ln / rn))
            }
            "%" => {
                let (ln, rn) = to_two_numbers(&l, &r).ok_or_else(|| type_error("Modulo waa in ay noqoto tiro", 0, 0))?;
                if rn == 0.0 {
                    return Err(runtime_error("Ma suurtogali karto modulo eber", 0, 0));
                }
                Ok(Value::Float(ln % rn))
            }
            "==" => Ok(Value::Bool(values_eq(&l, &r))),
            "!=" => Ok(Value::Bool(!values_eq(&l, &r))),
            ">" | "<" | ">=" | "<=" => {
                let (a, b) = to_two_numbers(&l, &r).ok_or_else(|| type_error("Isbarbar dhig waa in ay noqdaan tiro", 0, 0))?;
                let b = match op {
                    ">" => a > b,
                    "<" => a < b,
                    ">=" => a >= b,
                    _ => a <= b,
                };
                Ok(Value::Bool(b))
            }
            "&&" => Ok(Value::Bool(l.is_truthy() && r.is_truthy())),
            "||" => Ok(Value::Bool(l.is_truthy() || r.is_truthy())),
            _ => Err(runtime_error(format!("Hawl-gal aan la aqoon: {}", op), 0, 0)),
        }
    }

    fn eval_unary(&self, op: &str, v: Value, _line: usize) -> Result<Value, SoplangError> {
        match op {
            "!" => Ok(Value::Bool(!v.is_truthy())),
            "-" => match v {
                Value::Int(n) => Ok(Value::Int(-n)),
                Value::Float(x) => Ok(Value::Float(-x)),
                _ => Err(type_error("Unary - waa in uu noqdo tiro", 0, 0)),
            },
            _ => Err(runtime_error(format!("Hawl-gal aan la aqoon: {}", op), 0, 0)),
        }
    }

    fn validate_type(
        &self,
        name: &str,
        val: &Value,
        ann: &TypeAnnotation,
        line: usize,
        col: usize,
    ) -> Result<(), SoplangError> {
        if *ann == TypeAnnotation::Dynamic {
            return Ok(());
        }
        let ok = match (ann, val) {
            (TypeAnnotation::Abn, Value::Int(_)) => true,
            (TypeAnnotation::Abn, Value::Float(x)) => x.fract() == 0.0 && x.is_finite(),
            (TypeAnnotation::Jajab, Value::Float(_)) => true,
            (TypeAnnotation::Jajab, Value::Int(_)) => true,
            (TypeAnnotation::Qoraal, Value::Str(_)) => true,
            (TypeAnnotation::Bool, Value::Bool(_)) => true,
            (TypeAnnotation::Teed, Value::List(_)) => true,
            (TypeAnnotation::Walax, Value::Object(_)) => true,
            _ => false,
        };
        if ok {
            Ok(())
        } else {
            Err(type_error(
                format!(
                    "'{}' waa {} laakin qiimaheeda '{}' ma ahan {}",
                    name,
                    type_ann_str(ann),
                    val,
                    type_ann_str(ann)
                ),
                line,
                col,
            ))
        }
    }
}

fn literal_to_value(l: &Literal) -> Value {
    match l {
        Literal::Int(n) => Value::Int(*n),
        Literal::Float(x) => Value::Float(*x),
        Literal::Str(s) => Value::Str(s.clone()),
        Literal::Bool(b) => Value::Bool(*b),
        Literal::Null => Value::Null,
    }
}

fn value_to_str(v: &Value) -> String {
    match v {
        Value::Bool(true) => "run".to_string(),
        Value::Bool(false) => "been".to_string(),
        _ => format!("{}", v),
    }
}

fn to_number(v: &Value) -> Option<f64> {
    match v {
        Value::Int(n) => Some(*n as f64),
        Value::Float(x) => Some(*x),
        _ => None,
    }
}

fn to_two_numbers(l: &Value, r: &Value) -> Option<(f64, f64)> {
    Some((to_number(l)?, to_number(r)?))
}

fn binary_num<F, G>(_op: &str, l: Value, r: Value, fi: F, ff: G) -> Result<Value, SoplangError>
where
    F: Fn(i64, i64) -> i64,
    G: Fn(f64, f64) -> f64,
{
    match (&l, &r) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(fi(*a, *b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(ff(*a, *b))),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(ff(*a as f64, *b))),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(ff(*a, *b as f64))),
        _ => Err(type_error("Ma isticmaali karo noocyadaan tiro", 0, 0)),
    }
}

fn values_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Int(x), Value::Float(y)) => (*x as f64) == *y,
        (Value::Float(x), Value::Int(y)) => *x == (*y as f64),
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Null, Value::Null) => true,
        (Value::List(a), Value::List(b)) => *a.borrow() == *b.borrow(),
        (Value::Object(a), Value::Object(b)) => *a.borrow() == *b.borrow(),
        _ => false,
    }
}

fn to_int_index(arr: &Value, idx: &Value, line: usize, col: usize) -> Result<i64, SoplangError> {
    let n = match idx {
        Value::Int(n) => *n,
        Value::Float(x) if x.fract() == 0.0 => *x as i64,
        _ => return Err(type_error("Fihriska waa inuu noqdaa abn", line, col)),
    };
    if let Value::List(v) = arr {
        let len = v.borrow().len() as i64;
        let i = if n < 0 { len + n } else { n };
        if i < 0 || i >= len {
            return Err(runtime_error(format!("Tirada fihris-ku waa ka baxsan xadka: {}", i), line, col));
        }
        Ok(i)
    } else {
        Err(type_error("Ma heli karo tirooyinka ee qiimaha aan ahayn teed", line, col))
    }
}

fn index_value(arr: &Value, idx: &Value, line: usize, col: usize) -> Result<Value, SoplangError> {
    let i = to_int_index(arr, idx, line, col)?;
    if let Value::List(v) = arr {
        Ok(v.borrow()[i as usize].clone())
    } else {
        unreachable!()
    }
}

fn property_value(obj: &Value, prop: &str, line: usize, col: usize) -> Result<Value, SoplangError> {
    if let Value::Object(m) = obj {
        m.borrow()
            .get(prop)
            .cloned()
            .ok_or_else(|| runtime_error(format!("Astaanta '{}' kuma jirto walaxga", prop), line, col))
    } else {
        Err(type_error(format!("Ma heli karo astaanta '{}' ee qiimaha aan ahayn walax", prop), line, col))
    }
}

fn type_ann_str(a: &TypeAnnotation) -> &'static str {
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
