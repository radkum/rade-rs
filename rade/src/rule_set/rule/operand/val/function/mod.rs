use macros::register_functions;
use serde::{Deserialize, Serialize};

use super::{Event, ResultMap, Val, ValType};
use crate::RadeResult;

type FnName = String;
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct FnCall {
    name: FnName,
    args: Vec<Val>,
}

impl FnCall {
    pub fn new(name: String, args: Vec<Val>) -> Self {
        Self { name, args }
    }

    pub fn name(&self) -> &FnName {
        &self.name
    }
    pub fn is_bool(&self) -> RadeResult<bool> {
        let Some(ret_type) = FUNCTIONS.ret_type(self.name.as_str()) else {
            return Err(format!("Function {} not found", self.name()).into());
        };
        Ok(*ret_type == ValType::Bool)
    }

    pub fn call(&self, event: &Event, cache: &mut ResultMap) -> RadeResult<Val> {
        let args: RadeResult<Vec<_>> = self
            .args
            .iter()
            .map(|arg| arg.fn_arg(event, cache))
            .collect();
        let Some(function) = FUNCTIONS.function(self.name.as_str()) else {
            return Err(format!("Function {} not found", self.name()).into());
        };
        function(args?)
    }
}

#[register_functions(map = "FUNCTIONS")]
mod definitions {
    // Define functions with native Rust types.
    // The macro will generate wrappers that convert Vec<Val> to these types.

    pub fn concat(a: Vec<String>) -> String {
        a.join("")
    }

    pub fn split(s: String, delimiter: String) -> Vec<String> {
        s.split(&delimiter).map(|s| s.to_string()).collect()
    }

    pub fn length(s: String) -> i64 {
        s.len() as i64
    }

    pub fn is_empty(s: String) -> bool {
        s.is_empty()
    }

    pub fn float_sum(a: f64, b: f64) -> f64 {
        a + b
    }
}
