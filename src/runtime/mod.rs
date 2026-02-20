//! Runtime: values, stdlib builtins, and C ABI for backends.

pub mod abi;
pub mod stdlib;
pub mod value;

pub use abi::*;
pub use value::{value_to_string, NativeFn, Value};
