use serde::{Deserialize, Serialize};

use super::{Bool, Event, Float, Int, IntList, ResultMap, Str, StrList, Val};
use crate::{FatString, RadeResult, prelude::*};

type FnName = String;

/// Method call: receiver.method(args)
/// Calls Rust's native string methods directly.
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct MethodCall {
    receiver: Box<Val>,
    method: FnName,
    args: Vec<Val>,
}

impl MethodCall {
    pub fn new(receiver: Val, method: String, args: Vec<Val>) -> Self {
        Self {
            receiver: Box::new(receiver),
            method,
            args,
        }
    }

    pub fn method(&self) -> &FnName {
        &self.method
    }

    /// Returns true if this method returns a boolean value
    pub fn is_bool(&self) -> RadeResult<bool> {
        match self.method.as_str() {
            // Boolean-returning methods
            "is_empty" | "contains" | "starts_with" | "ends_with" => Ok(true),
            // Non-boolean methods
            "len" | "to_uppercase" | "to_lowercase" | "trim" | "trim_start" | "trim_end"
            | "replace" | "to_string" | "first" | "last" | "get" | "join" | "sum" | "min"
            | "max" | "reverse" => Ok(false),
            _ => Err(format!("Unknown method: {}", self.method).into()),
        }
    }

    pub fn call(&self, event: &Event, cache: &mut ResultMap) -> RadeResult<Val> {
        // Evaluate the receiver first
        let receiver_val = self.receiver.fn_arg(event, cache)?;

        // Evaluate the arguments
        let mut args = Vec::with_capacity(self.args.len());
        for arg in &self.args {
            args.push(arg.fn_arg(event, cache)?);
        }

        // Dispatch to native Rust methods based on receiver type
        match receiver_val {
            Val::Str(s) => self.call_str_method(s.0.plain(), &args),
            Val::Int(Int(i)) => self.call_int_method(i, &args),
            Val::Float(Float(f)) => self.call_float_method(f, &args),
            Val::StrList(list) => self.call_str_list_method(&list, &args),
            Val::IntList(list) => self.call_int_list_method(&list, &args),
            _ => Err(format!(
                "Method '{}' not supported on type {:?}",
                self.method, receiver_val
            )
            .into()),
        }
    }

    fn call_str_method(&self, s: &str, args: &[Val]) -> RadeResult<Val> {
        match self.method.as_str() {
            // No-argument methods
            "len" => Ok(Val::Int(Int(s.len() as i64))),
            "is_empty" => Ok(Val::Bool(Bool(s.is_empty()))),
            "to_uppercase" => Ok(Val::Str(Str::from(s.to_uppercase()))),
            "to_lowercase" => Ok(Val::Str(Str::from(s.to_lowercase()))),
            "trim" => Ok(Val::Str(Str::from(s.trim()))),
            "trim_start" => Ok(Val::Str(Str::from(s.trim_start()))),
            "trim_end" => Ok(Val::Str(Str::from(s.trim_end()))),
            "to_string" => Ok(Val::Str(Str::from(s))),

            // Single-argument methods
            "contains" => {
                let substr = self.get_str_arg(args, 0, "contains")?;
                Ok(Val::Bool(Bool(s.contains(substr))))
            },
            "starts_with" => {
                let prefix = self.get_str_arg(args, 0, "starts_with")?;
                Ok(Val::Bool(Bool(s.starts_with(prefix))))
            },
            "ends_with" => {
                let suffix = self.get_str_arg(args, 0, "ends_with")?;
                Ok(Val::Bool(Bool(s.ends_with(suffix))))
            },

            // Two-argument methods
            "replace" => {
                let from = self.get_str_arg(args, 0, "replace")?;
                let to = self.get_str_arg(args, 1, "replace")?;
                Ok(Val::Str(Str::from(s.replace(from, to))))
            },

            _ => Err(format!("Unknown string method: {}", self.method).into()),
        }
    }

    fn call_int_method(&self, i: i64, _args: &[Val]) -> RadeResult<Val> {
        match self.method.as_str() {
            "abs" => Ok(Val::Int(Int(i.abs()))),
            "to_string" => Ok(Val::Str(Str::from(i.to_string()))),
            _ => Err(format!("Unknown integer method: {}", self.method).into()),
        }
    }

    fn call_float_method(&self, f: f64, _args: &[Val]) -> RadeResult<Val> {
        match self.method.as_str() {
            "abs" => Ok(Val::Float(Float(f.abs()))),
            "floor" => Ok(Val::Float(Float(f.floor()))),
            "ceil" => Ok(Val::Float(Float(f.ceil()))),
            "round" => Ok(Val::Float(Float(f.round()))),
            "to_string" => Ok(Val::Str(Str::from(f.to_string()))),
            _ => Err(format!("Unknown float method: {}", self.method).into()),
        }
    }

    fn call_str_list_method(&self, list: &StrList, args: &[Val]) -> RadeResult<Val> {
        match self.method.as_str() {
            // No-argument methods
            "len" => Ok(Val::Int(Int(list.0.len() as i64))),
            "is_empty" => Ok(Val::Bool(Bool(list.0.is_empty()))),
            "first" => list
                .0
                .first()
                .map(|s| Val::Str(Str::from(s.plain().to_string())))
                .ok_or_else(|| "List is empty".into()),
            "last" => list
                .0
                .last()
                .map(|s| Val::Str(Str::from(s.plain().to_string())))
                .ok_or_else(|| "List is empty".into()),
            "reverse" => {
                let reversed: Vec<FatString> = list.0.iter().rev().cloned().collect();
                Ok(Val::StrList(StrList(reversed)))
            },

            // Single-argument methods
            "get" => {
                let index = self.get_int_arg(args, 0, "get")?;
                let item = list.get(index)?;
                Ok(Val::Str(Str::from(item.plain().to_string())))
            },
            "contains" => {
                let needle = self.get_str_arg(args, 0, "contains")?;
                let found = list.0.iter().any(|s| s.plain() == needle);
                Ok(Val::Bool(Bool(found)))
            },
            "join" => {
                let separator = self.get_str_arg(args, 0, "join")?;
                let joined: String = list
                    .0
                    .iter()
                    .map(|s| s.plain())
                    .collect::<Vec<_>>()
                    .join(separator);
                Ok(Val::Str(Str::from(joined)))
            },

            _ => Err(format!("Unknown string list method: {}", self.method).into()),
        }
    }

    fn call_int_list_method(&self, list: &IntList, args: &[Val]) -> RadeResult<Val> {
        match self.method.as_str() {
            // No-argument methods
            "len" => Ok(Val::Int(Int(list.0.len() as i64))),
            "is_empty" => Ok(Val::Bool(Bool(list.0.is_empty()))),
            "first" => list
                .0
                .first()
                .map(|&i| Val::Int(Int(i)))
                .ok_or_else(|| "List is empty".into()),
            "last" => list
                .0
                .last()
                .map(|&i| Val::Int(Int(i)))
                .ok_or_else(|| "List is empty".into()),
            "sum" => {
                let sum: i64 = list.0.iter().sum();
                Ok(Val::Int(Int(sum)))
            },
            "min" => list
                .0
                .iter()
                .min()
                .map(|&i| Val::Int(Int(i)))
                .ok_or_else(|| "List is empty".into()),
            "max" => list
                .0
                .iter()
                .max()
                .map(|&i| Val::Int(Int(i)))
                .ok_or_else(|| "List is empty".into()),
            "reverse" => {
                let reversed: Vec<i64> = list.0.iter().rev().copied().collect();
                Ok(Val::IntList(IntList(reversed)))
            },

            // Single-argument methods
            "get" => {
                let index = self.get_int_arg(args, 0, "get")?;
                let item = list.get(index)?;
                Ok(Val::Int(Int(*item)))
            },
            "contains" => {
                let needle = self.get_int_arg(args, 0, "contains")?;
                let found = list.0.contains(&needle);
                Ok(Val::Bool(Bool(found)))
            },

            _ => Err(format!("Unknown integer list method: {}", self.method).into()),
        }
    }

    fn get_str_arg<'a>(&self, args: &'a [Val], index: usize, method: &str) -> RadeResult<&'a str> {
        match args.get(index) {
            Some(Val::Str(s)) => Ok(s.0.plain()),
            Some(other) => Err(format!(
                "Method '{}' argument {} expected string, got {:?}",
                method, index, other
            )
            .into()),
            None => Err(format!("Method '{}' missing argument at index {}", method, index).into()),
        }
    }

    fn get_int_arg(&self, args: &[Val], index: usize, method: &str) -> RadeResult<i64> {
        match args.get(index) {
            Some(Val::Int(Int(i))) => Ok(*i),
            Some(other) => Err(format!(
                "Method '{}' argument {} expected integer, got {:?}",
                method, index, other
            )
            .into()),
            None => Err(format!("Method '{}' missing argument at index {}", method, index).into()),
        }
    }
}
