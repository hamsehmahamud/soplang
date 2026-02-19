//! Runtime values for Soplang. Matches Python types used in psrc/runtime/interpreter.py.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    List(Rc<RefCell<Vec<Value>>>),
    Object(Rc<RefCell<HashMap<String, Value>>>),
    Null,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::List(a), Value::List(b)) => Rc::ptr_eq(a, b) || *a.borrow() == *b.borrow(),
            (Value::Object(a), Value::Object(b)) => Rc::ptr_eq(a, b) || *a.borrow() == *b.borrow(),
            _ => false,
        }
    }
}

impl Value {
    /// Somali type name for error messages (nooc). Phase 4+.
    #[allow(dead_code)]
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "abn",
            Value::Float(_) => "jajab",
            Value::Str(_) => "qoraal",
            Value::Bool(_) => "bool",
            Value::List(_) => "teed",
            Value::Object(_) => "walax",
            Value::Null => "null",
        }
    }

    /// Truthiness for conditions and short-circuit.
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(x) => *x != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::List(l) => !l.borrow().is_empty(),
            Value::Object(o) => !o.borrow().is_empty(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(x) => {
                if x.fract() == 0.0 && x.is_finite() {
                    write!(f, "{}", *x as i64)
                } else {
                    write!(f, "{}", x)
                }
            }
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", if *b { "run" } else { "been" }),
            Value::Null => write!(f, "null"),
            Value::List(lst) => {
                let v = lst.borrow();
                write!(f, "[")?;
                for (i, item) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Object(obj) => {
                let m = obj.borrow();
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
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
