mod bool;
mod field;
mod float;
mod function;
mod int;
mod int_list;
mod regex;
mod serialization;
mod str;
mod str_list;
mod val_type;

pub use bool::*;
pub use field::*;
pub use float::*;
pub use function::FnCall;
pub use int::*;
pub use int_list::*;
pub use regex::RadeRegex;
use serde::{Deserialize, Serialize};
use serde_yaml_bw::Value as YamlValue;
pub use str::*;
pub use str_list::*;
use val_type::ValType;

use crate::{Event, FatString, InsensitiveFlag, OperandContainer, RadeResult, ResultMap};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub enum Comparator {
    Eq,
    Neq,
    Gt,
    Lt,
    Ge,
    Le,
    Match,
    Nmatch,
}

impl Comparator {
    pub fn negate(&self) -> Self {
        match self {
            Comparator::Eq => Comparator::Neq,
            Comparator::Neq => Comparator::Eq,
            Comparator::Gt => Comparator::Le,
            Comparator::Lt => Comparator::Ge,
            Comparator::Ge => Comparator::Lt,
            Comparator::Le => Comparator::Gt,
            Comparator::Match => Comparator::Nmatch,
            Comparator::Nmatch => Comparator::Match,
        }
    }

    /// Swap the comparator for operand swap (a op b → b swap(op) a)
    pub fn swap(&self) -> Self {
        match self {
            Comparator::Eq => Comparator::Eq,
            Comparator::Neq => Comparator::Neq,
            Comparator::Gt => Comparator::Lt,
            Comparator::Lt => Comparator::Gt,
            Comparator::Ge => Comparator::Le,
            Comparator::Le => Comparator::Ge,
            Comparator::Match => Comparator::Match,
            Comparator::Nmatch => Comparator::Nmatch,
        }
    }
}
pub trait Eq {
    fn equal<'a>(
        &'a self,
        _elem: &Val,
        _event: &'a Event,
        _comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        Err("Not implemented".into())
    }

    fn neq<'a>(
        &'a self,
        _elem: &Val,
        _event: &'a Event,
        _comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        Err("Not implemented".into())
    }
}

pub trait Match {
    fn match_<'a>(
        &'a self,
        _elem: &Val,
        _event: &'a Event,
        _comp_flag: &Comparator,
    ) -> RadeResult<bool> {
        Err("Not implemented".into())
    }
}

pub trait Contains {
    fn contains<'a>(
        &'a self,
        _elem: &Val,
        _event: &'a Event,
        _comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        Err("Not implemented".into())
    }
}

pub trait Cast {
    fn as_i64<'a>(&'a self, _event: &'a Event) -> RadeResult<i64> {
        Err("Not implemented".into())
    }
    fn as_i64_list<'a>(&'a self, _event: &'a Event) -> RadeResult<&'a Vec<i64>> {
        Err("Not implemented".into())
    }
    fn as_f64<'a>(&'a self, _event: &'a Event) -> RadeResult<f64> {
        Err("Not implemented".into())
    }
    fn as_bool<'a>(
        &'a self,
        _event: &'a Event,
        _cache: Option<&mut ResultMap>,
    ) -> RadeResult<bool> {
        Err("Not implemented".into())
    }

    fn as_str<'a>(
        &'a self,
        _event: &'a Event,
        _comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<&'a str> {
        Err("Not implemented".into())
    }
    fn as_str_list<'a>(
        &'a self,
        _event: &'a Event,
        _comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<Vec<&'a str>> {
        Err("Not implemented".into())
    }
}

pub trait Compare {
    fn cmp<'a>(
        &'a self,
        _elem: &Val,
        _event: &'a Event,
        _comparator: &Comparator,
        _flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        Err(format!("Not a number").into())
    }
}

#[derive(Debug, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub enum Val {
    Bool(Bool),
    Int(Int),
    IntList(IntList),
    Float(Float),
    Str(Str),
    StrList(StrList),
    Regex(RadeRegex),
    Field(Field),
    FieldIndex(Field, i64),
    Expression(Box<OperandContainer>),
    FnCall(FnCall), //MethodCall(FnName, Vec<Val>)
}

impl Val {
    pub fn fn_arg(&self, event: &Event, cache: &mut ResultMap) -> RadeResult<Val> {
        // For now, we only evaluate expressions and function calls. In the future, we
        // may want to evaluate fields as well.
        match self {
            Val::Field(Field(field_name)) => event.get_field(field_name).map(|v| v.clone()),
            Val::FieldIndex(field, index) => field.get_val(event, *index),
            Val::FnCall(fn_call) => fn_call.call(event, cache),
            Val::Expression(_) => Err("Expression cannot be function argument".into()),
            Val::Regex(_) => Err("Regex cannot be function argument".into()),
            _ => Ok(self.clone()),
        }
    }

    pub fn validate_bool(self) -> RadeResult<Val> {
        match self {
            Val::Bool(_) => Ok(self),
            Val::Field(_) => Ok(self),
            Val::Expression(_) => Ok(self),
            Val::FnCall(ref fn_call) => {
                if fn_call.is_bool()? {
                    Ok(self)
                } else {
                    Err(format!("Function {} does not return a boolean", fn_call.name()).into())
                }
            },
            _ => Err(format!("Not a boolean: {:?}", self).into()),
        }
    }

    /// Convert Val to bool (for function arguments, without Event context)
    pub fn to_bool(&self) -> RadeResult<bool> {
        match self {
            Val::Bool(b) => Ok(b.0),
            _ => Err(format!("Cannot convert {:?} to bool", self).into()),
        }
    }

    /// Convert Val to i64 (for function arguments, without Event context)
    pub fn to_i64(&self) -> RadeResult<i64> {
        match self {
            Val::Int(i) => Ok(i.0),
            _ => Err(format!("Cannot convert {:?} to i64", self).into()),
        }
    }

    /// Convert Val to i64 (for function arguments, without Event context)
    pub fn to_f64(&self) -> RadeResult<f64> {
        match self {
            Val::Float(f) => Ok(f.0),
            _ => Err(format!("Cannot convert {:?} to f64", self).into()),
        }
    }

    /// Convert Val to String (for function arguments, without Event context)
    pub fn to_string(&self) -> RadeResult<String> {
        match self {
            Val::Str(s) => Ok(s.0.plain().to_string()),
            _ => Err(format!("Cannot convert {:?} to String", self).into()),
        }
    }

    /// Convert Val to String (for function arguments, without Event context)
    pub fn to_int_list(&self) -> RadeResult<Vec<String>> {
        match self {
            Val::IntList(list) => Ok(list.0.iter().map(|i| i.to_string()).collect()),
            _ => Err(format!("Cannot convert {:?} to Vec<String>", self).into()),
        }
    }

    /// Convert Val to String (for function arguments, without Event context)
    pub fn to_str_list(&self) -> RadeResult<Vec<String>> {
        match self {
            Val::StrList(list) => Ok(list.0.iter().map(|s| s.plain().to_string()).collect()),
            _ => Err(format!("Cannot convert {:?} to Vec<String>", self).into()),
        }
    }
}

impl Eq for Val {
    fn equal<'a>(
        &'a self,
        elem: &Val,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        match self {
            Val::Int(i) => i.equal(elem, event, comp_flag),
            Val::Float(f) => f.equal(elem, event, comp_flag),
            Val::IntList(_) => Err("Eq for intlist not implemented".into()),
            Val::Str(s) => s.equal(elem, event, comp_flag),
            Val::StrList(_) => Err("Eq for strlist not implemented".into()),
            Val::Field(field) => field.equal(elem, event, comp_flag),
            Val::Bool(b) => b.equal(elem, event, comp_flag),
            Val::Regex(_) => Err("Eq for regex not implemented".into()),
            Val::FieldIndex(field, index) => {
                field.get_val(event, *index)?.equal(elem, event, comp_flag)
            },
            Val::Expression(_) => Err("Eq for expression not implemented".into()),
            Val::FnCall(_) => Err("Eq for function call not implemented".into()),
        }
    }

    fn neq<'a>(
        &'a self,
        elem: &Val,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        match self {
            Val::Bool(b) => b.neq(elem, event, comp_flag),
            Val::Int(int) => int.neq(elem, event, comp_flag),
            Val::Float(float) => float.neq(elem, event, comp_flag),
            Val::IntList(_) => Err("Neq for intlist not implemented".into()),
            Val::Str(str) => str.neq(elem, event, comp_flag),
            Val::StrList(_) => Err("Neq for strlist not implemented".into()),
            Val::Regex(_) => Err("Neq for regex not implemented".into()),
            Val::Field(field) => field.neq(elem, event, comp_flag),
            Val::FieldIndex(field, index) => {
                field.get_val(event, *index)?.neq(elem, event, comp_flag)
            },
            Val::Expression(_) => Err("Neq for expression not implemented".into()),
            Val::FnCall(_) => Err("Neq for function call not implemented".into()),
        }
    }
}

impl Match for Val {
    fn match_<'a>(
        &'a self,
        elem: &Val,
        event: &'a Event,
        comparator: &Comparator,
    ) -> RadeResult<bool> {
        match self {
            Val::Regex(i) => i.match_(elem, event, comparator),
            _ => Err("Match implemented only for Regex".into()),
        }
    }
}

impl Cast for Val {
    fn as_i64<'a>(&'a self, event: &'a Event) -> RadeResult<i64> {
        match self {
            Val::Int(int) => int.as_i64(event),
            Val::Float(float) => float.as_i64(event),
            Val::Field(field) => field.as_i64(event),
            Val::FieldIndex(f, index) => Ok(f.get_int(event, *index)?),
            _ => Err(format!("Cannot convert {:?} to i64", self).into()),
        }
    }

    fn as_f64<'a>(&'a self, event: &'a Event) -> RadeResult<f64> {
        match self {
            Val::Int(int) => int.as_f64(event),
            Val::Float(float) => float.as_f64(event),
            Val::Field(field) => field.as_f64(event),
            Val::FieldIndex(f, index) => Ok(f.get_int(event, *index)? as f64),
            _ => Err(format!("Cannot convert {:?} to f64", self).into()),
        }
    }

    fn as_str<'a>(
        &'a self,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<&'a str> {
        match self {
            Val::Str(str) => str.as_str(event, comp_flag),
            Val::StrList(str_list) => str_list.as_str(event, comp_flag),
            Val::Field(f) => f.as_str(event, comp_flag),
            Val::FieldIndex(f, index) => Ok(f.get_str(event, *index)?.choose(comp_flag)),
            _ => Err(format!("Cannot convert {:?} to &str", self).into()),
        }
    }

    fn as_i64_list<'a>(&'a self, event: &'a Event) -> RadeResult<&'a Vec<i64>> {
        match self {
            Val::Int(i) => i.as_i64_list(event),
            Val::IntList(i) => i.as_i64_list(event),
            _ => Err(format!("Cannot convert {:?} to Vec<i64>", self).into()),
        }
    }

    fn as_str_list<'a>(
        &'a self,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<Vec<&'a str>> {
        match self {
            Val::Str(s) => s.as_str_list(event, comp_flag),
            Val::StrList(s) => s.as_str_list(event, comp_flag),
            _ => Err(format!("Cannot convert {:?} to Vec<&str>", self).into()),
        }
    }

    fn as_bool<'a>(&'a self, event: &'a Event, cache: Option<&mut ResultMap>) -> RadeResult<bool> {
        match self {
            Val::Bool(b) => b.as_bool(event, None),
            Val::Field(f) => f.as_bool(event, None),
            Val::Expression(expr) => {
                Ok(expr.evaluate(event, cache.unwrap_or(&mut ResultMap::new())))
            },
            Val::FnCall(fn_call) => {
                // Evaluate function call arguments
                fn_call
                    .call(event, cache.unwrap_or(&mut ResultMap::new()))?
                    .to_bool()
            },
            _ => Err("Type mismatch. Expected boolean.".into()),
        }
    }
}

impl Compare for Val {
    fn cmp<'a>(
        &'a self,
        elem: &Val,
        event: &'a Event,
        comparator: &Comparator,
        flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        println!("Comparing {:?} with {:?}", self, elem);
        match self {
            Val::Bool(b) => b.cmp(elem, event, comparator, flag),
            Val::Int(i) => i.cmp(elem, event, comparator, flag),
            Val::Float(f) => f.cmp(elem, event, comparator, flag),
            Val::Str(s) => s.cmp(elem, event, comparator, flag),
            Val::Field(field) => event
                .get_field(&field.0)?
                .cmp(elem, event, comparator, flag),
            Val::FieldIndex(field, index) => field
                .get_val(event, *index)?
                .cmp(elem, event, comparator, flag),
            Val::FnCall(fn_call) => {
                // Evaluate the function call and compare its result
                let result = fn_call.call(event, &mut ResultMap::new())?;
                result.cmp(elem, event, comparator, flag)
            },
            _ => Err(format!("Cannot compare {:?} with {:?}", self, elem).into()),
        }
    }
}

// From implementations for native types (used by function wrappers)
impl From<i64> for Val {
    fn from(value: i64) -> Self {
        Val::Int(Int(value))
    }
}

impl From<f64> for Val {
    fn from(value: f64) -> Self {
        Val::Float(Float(value))
    }
}

impl From<bool> for Val {
    fn from(value: bool) -> Self {
        Val::Bool(Bool(value))
    }
}

impl From<String> for Val {
    fn from(value: String) -> Self {
        Val::Str(Str(value.into()))
    }
}

impl From<&YamlValue> for Val {
    fn from(value: &YamlValue) -> Self {
        match value {
            YamlValue::Bool(b) => Val::Bool(Bool(*b)),
            YamlValue::Number(n) => {
                if n.is_f64() {
                    Val::Float(Float(n.as_f64().unwrap()))
                } else {
                    Val::Int(Int(n.as_i64().unwrap()))
                }
            },
            YamlValue::String(s) => Val::Str(Str(FatString::from(s))),
            YamlValue::Sequence(seq) => {
                if seq.iter().all(|v| v.is_i64()) {
                    Val::IntList(IntList(
                        seq.iter().map(|v| v.as_i64().unwrap_or_default()).collect(),
                    ))
                } else if seq.iter().all(|v| v.is_string()) {
                    Val::StrList(StrList(
                        seq.iter()
                            .map(|v| FatString::from(v.as_str().unwrap_or_default()))
                            .collect(),
                    ))
                } else {
                    panic!("Unsupported YAML sequence type")
                }
            },
            _ => todo!(),
        }
    }
}

impl From<&Val> for YamlValue {
    fn from(value: &Val) -> Self {
        match value {
            Val::Int(i) => YamlValue::Number(i.0.into()),
            Val::Str(s) => YamlValue::String(s.0.plain().to_string()),
            Val::Bool(b) => YamlValue::Bool(b.0),
            Val::Float(f) => YamlValue::Number(f.0.into()),
            _ => todo!(),
        }
    }
}
