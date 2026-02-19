//! Tree-walking interpreter. Phase 4: functions, classes, import, try/catch.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use crate::ast::{Expr, Literal, Param, Stmt, TypeAnnotation};
use crate::error::{runtime_error, type_error, SoplangError};
use crate::scope::Env;
use crate::value::{Value, FunctionId};

#[derive(Debug)]
pub enum Signal {
    None,
    Break,
    Continue,
    #[allow(dead_code)] // Phase 4: return value used by execute_function_call
    Return(Value),
}

/// User-defined function (captured env for closure).
#[allow(dead_code)] // name for debugging / error messages
pub struct FunctionDef {
    pub name:   String,
    pub params: Vec<String>,
    pub body:   Vec<Stmt>,
    pub env:    Rc<RefCell<Env>>,
}

/// Class definition: default fields and method (param_names, body).
#[allow(dead_code)] // name, parent for inheritance / error messages
pub struct ClassDef {
    pub name:    String,
    pub parent:  Option<String>,
    pub methods: HashMap<String, (Vec<String>, Vec<Stmt>)>,
    pub fields:  HashMap<String, Value>,
}

pub struct Interpreter {
    pub globals:  Rc<RefCell<Env>>,
    pub functions: Vec<FunctionDef>,
    pub classes:   HashMap<String, ClassDef>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            globals:   Rc::new(RefCell::new(Env::new())),
            functions: Vec::new(),
            classes:   HashMap::new(),
        }
    }

    #[allow(dead_code)] // public API when no file path (e.g. REPL, tests)
    pub fn run(&mut self, stmts: Vec<Stmt>) -> Result<(), SoplangError> {
        self.run_with_path(stmts, None)
    }

    pub fn run_with_path(&mut self, stmts: Vec<Stmt>, current_file: Option<&Path>) -> Result<(), SoplangError> {
        for stmt in &stmts {
            let sig = self.exec_stmt(stmt, Rc::clone(&self.globals), current_file)?;
            match sig {
                Signal::Break => return Err(runtime_error("Jooji waa in ay ku jiraan xalqad", 0, 0)),
                Signal::Continue => return Err(runtime_error("soco waa in ay ku jiraan xalqad", 0, 0)),
                Signal::Return(_) => return Err(runtime_error("celi waa in ay ku jirto hawl", 0, 0)),
                Signal::None => {}
            }
        }
        Ok(())
    }

    fn exec_stmt(
        &mut self,
        stmt: &Stmt,
        env: Rc<RefCell<Env>>,
        current_file: Option<&Path>,
    ) -> Result<Signal, SoplangError> {
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
                    return self.exec_block(then_body, env, current_file);
                }
                for (c, body) in elseifs {
                    if self.eval_expr(c, Rc::clone(&env))?.is_truthy() {
                        return self.exec_block(body, env, current_file);
                    }
                }
                if let Some(eb) = else_body {
                    return self.exec_block(eb, env, current_file);
                }
                Ok(Signal::None)
            }
            Stmt::Switch { expr, cases, default } => {
                let v = self.eval_expr(expr, Rc::clone(&env))?;
                for (case_val, body) in cases {
                    let c = self.eval_expr(case_val, Rc::clone(&env))?;
                    if values_eq(&v, &c) {
                        return self.exec_block(body, env, current_file);
                    }
                }
                if let Some(d) = default {
                    return self.exec_block(d, env, current_file);
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
                    let sig = self.exec_block(body, Rc::clone(&env), current_file)?;
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
                    let sig = self.exec_block(body, Rc::clone(&env), current_file)?;
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
            Stmt::Block(stmts) => self.exec_block(stmts, env, current_file),
            Stmt::Expr(e) => {
                self.eval_expr(e, env)?;
                Ok(Signal::None)
            }
            Stmt::FuncDef { name, params, body } => self.exec_func_def(name, params, body, env),
            Stmt::ClassDef { name, parent, body } => {
                self.exec_class_def(name, parent.clone(), body, env, current_file)
            }
            Stmt::TryCatch { try_body, err_var, catch_body } => {
                self.exec_try_catch(try_body, err_var, catch_body, env, current_file)
            }
            Stmt::Import(path) => self.exec_import(path, current_file, env),
        }
    }

    fn exec_block(
        &mut self,
        body: &[Stmt],
        env: Rc<RefCell<Env>>,
        current_file: Option<&Path>,
    ) -> Result<Signal, SoplangError> {
        for stmt in body {
            let sig = self.exec_stmt(stmt, Rc::clone(&env), current_file)?;
            if !matches!(sig, Signal::None) {
                return Ok(sig);
            }
        }
        Ok(Signal::None)
    }

    fn exec_func_def(
        &mut self,
        name: &str,
        params: &[Param],
        body: &[Stmt],
        env: Rc<RefCell<Env>>,
    ) -> Result<Signal, SoplangError> {
        let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
        let def = FunctionDef {
            name:   name.to_string(),
            params: param_names.clone(),
            body:   body.to_vec(),
            env:    Rc::clone(&env),
        };
        let id = self.functions.len();
        self.functions.push(def);
        env.borrow_mut().define(name, Value::Function(FunctionId(id)), TypeAnnotation::Dynamic, false);
        Ok(Signal::None)
    }

    fn exec_class_def(
        &mut self,
        name: &str,
        parent: Option<String>,
        body: &[Stmt],
        env: Rc<RefCell<Env>>,
        current_file: Option<&Path>,
    ) -> Result<Signal, SoplangError> {
        if let Some(ref p) = parent {
            if !self.classes.contains_key(p) {
                return Err(runtime_error(format!("Fasalka waalidka '{}' ma jiro", p), 0, 0));
            }
        }
        let mut methods = HashMap::new();
        let mut fields = HashMap::new();
        for stmt in body {
            match stmt {
                Stmt::FuncDef { name: mname, params, body: b } => {
                    let pnames: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                    methods.insert(mname.clone(), (pnames, b.clone()));
                }
                Stmt::VarDecl { name: fname, value, .. } => {
                    let val = self.eval_expr(value, Rc::clone(&env))?;
                    fields.insert(fname.clone(), val);
                }
                _ => {
                    let _ = self.exec_stmt(stmt, Rc::clone(&env), current_file);
                }
            }
        }
        self.classes.insert(
            name.to_string(),
            ClassDef {
                name:   name.to_string(),
                parent,
                methods,
                fields,
            },
        );
        Ok(Signal::None)
    }

    fn exec_try_catch(
        &mut self,
        try_body: &[Stmt],
        err_var: &str,
        catch_body: &[Stmt],
        env: Rc<RefCell<Env>>,
        current_file: Option<&Path>,
    ) -> Result<Signal, SoplangError> {
        match self.exec_block(try_body, Rc::clone(&env), current_file) {
            Ok(sig) => Ok(sig),
            Err(e) => {
                env.borrow_mut().define(
                    err_var,
                    Value::Str(e.to_string()),
                    TypeAnnotation::Dynamic,
                    false,
                );
                self.exec_block(catch_body, env, current_file)
            }
        }
    }

    fn exec_import(
        &mut self,
        filename: &str,
        current_file: Option<&Path>,
        env: Rc<RefCell<Env>>,
    ) -> Result<Signal, SoplangError> {
        let path = match current_file {
            Some(p) => p.parent().unwrap_or_else(|| p.as_ref()).join(filename),
            None => filename.into(),
        };
        let source = std::fs::read_to_string(&path).map_err(|_| {
            SoplangError::Import {
                msg:  format!("Faylka '{}' ma helin", filename),
                line: 0,
                col:  0,
            }
        })?;
        let tokens = crate::lexer::Lexer::new(&source).tokenize().map_err(|e| {
            SoplangError::Import {
                msg:  e.to_string(),
                line: 0,
                col:  0,
            }
        })?;
        let stmts = crate::parser::Parser::new(tokens).parse().map_err(|e| {
            SoplangError::Import {
                msg:  e.to_string(),
                line: 0,
                col:  0,
            }
        })?;
        self.exec_block(&stmts, env, Some(path.as_path()))
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
                if name == "cusub" {
                    let class_name = match evaled.first() {
                        Some(Value::Str(s)) => s.clone(),
                        _ => return Err(runtime_error("cusub: argumanka koowaad waa in uu noqdo magac fasalka", 0, 0)),
                    };
                    let fields = self
                        .classes
                        .get(&class_name)
                        .ok_or_else(|| runtime_error(format!("Fasalka '{}' ma jiro", class_name), 0, 0))
                        .map(|c| {
                            let mut f = c.fields.clone();
                            f.insert("__class__".to_string(), Value::Str(class_name.clone()));
                            f
                        })?;
                    let instance = Value::Object(Rc::new(RefCell::new(fields)));
                    let dhaw = self.classes.get(&class_name).and_then(|c| c.methods.get("dhaw").cloned());
                    if let Some((param_names, body)) = dhaw {
                        let call_env = Rc::new(RefCell::new(Env::new_child(Rc::clone(&env))));
                        call_env.borrow_mut().define("nafta", instance.clone(), TypeAnnotation::Dynamic, false);
                        for (i, p) in param_names.iter().skip(1).enumerate() {
                            let arg = evaled.get(i + 1).cloned().unwrap_or(Value::Null);
                            call_env.borrow_mut().define(p, arg, TypeAnnotation::Dynamic, false);
                        }
                        let _ = self.exec_block(&body, call_env, None)?;
                    }
                    Ok(instance)
                } else {
                    let callee = env.borrow().get(name).ok_or_else(|| {
                        runtime_error(format!("Hawl aan la qeexin: '{}'", name), 0, 0)
                    })?;
                    if let Value::Function(id) = callee {
                        self.call_user_function(id.0, &evaled, env)
                    } else {
                        Err(runtime_error(format!("'{}' ma aha hawl", name), 0, 0))
                    }
                }
            }
            Expr::MethodCall { obj, method, args } => {
                let receiver = self.eval_expr(obj, Rc::clone(&env))?;
                let args_val: Vec<Value> = args
                    .iter()
                    .map(|a| self.eval_expr(a, Rc::clone(&env)))
                    .collect::<Result<Vec<_>, _>>()?;
                self.eval_method_call(receiver, method, &args_val, env)
            }
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

    fn call_user_function(
        &mut self,
        id: usize,
        args: &[Value],
        _env: Rc<RefCell<Env>>,
    ) -> Result<Value, SoplangError> {
        let body = self
            .functions
            .get(id)
            .ok_or_else(|| runtime_error("Hawl aan la aqoon", 0, 0))
            .map(|def| (def.body.clone(), def.params.clone(), Rc::clone(&def.env)))?;
        let (body, params, def_env) = body;
        let call_env = Rc::new(RefCell::new(Env::new_child(def_env)));
        for (i, p) in params.iter().enumerate() {
            let val = args.get(i).cloned().unwrap_or(Value::Null);
            call_env.borrow_mut().define(p, val, TypeAnnotation::Dynamic, false);
        }
        match self.exec_block(&body, call_env, None) {
            Ok(Signal::Return(v)) => Ok(v),
            Ok(_) => Ok(Value::Null),
            Err(e) => Err(e),
        }
    }

    fn eval_method_call(
        &mut self,
        receiver: Value,
        method: &str,
        args: &[Value],
        env: Rc<RefCell<Env>>,
    ) -> Result<Value, SoplangError> {
        let class_name = match &receiver {
            Value::Object(m) => m
                .borrow()
                .get("__class__")
                .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
                .ok_or_else(|| runtime_error("Habka waxaa loo yeedhi karaa walax kaliya", 0, 0))?,
            _ => return Err(runtime_error("Habka waxaa loo yeedhi karaa walax kaliya", 0, 0)),
        };
        let (param_names, body) = self
            .classes
            .get(&class_name)
            .and_then(|c| c.methods.get(method).cloned())
            .ok_or_else(|| {
                runtime_error(
                    format!("Fasalka '{}' ma jiro ama habka '{}' ma jiro", class_name, method),
                    0,
                    0,
                )
            })?;
        let call_env = Rc::new(RefCell::new(Env::new_child(env)));
        call_env.borrow_mut().define("nafta", receiver, TypeAnnotation::Dynamic, false);
        for (i, p) in param_names.iter().filter(|s| *s != "nafta").enumerate() {
            let val = args.get(i).cloned().unwrap_or(Value::Null);
            call_env.borrow_mut().define(p, val, TypeAnnotation::Dynamic, false);
        }
        match self.exec_block(&body, call_env, None) {
            Ok(Signal::Return(v)) => Ok(v),
            Ok(_) => Ok(Value::Null),
            Err(e) => Err(e),
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
