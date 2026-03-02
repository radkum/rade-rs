mod field;
mod float;
mod bool;
mod int;
mod int_list;
mod num;
mod regex;
mod serialization;
mod str;
mod str_list;

pub use bool::*;
pub use field::*;
pub use float::*;
pub use int::*;
pub use int_list::*;
pub use num::{Comparator, Num};
pub use regex::RadeRegex;
use serde_yaml_bw::Value as YamlValue;
pub use str::*;
pub use str_list::*;

use crate::{Event, FatString, InsensitiveFlag, RadeResult};

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
        _elem: &Field,
        _event: &'a Event,
        _comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        Err("Not implemented".into())
    }

    fn not_match<'a>(
        &'a self,
        _elem: &Field,
        _event: &'a Event,
        _comp_flag: &Option<InsensitiveFlag>,
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
    fn as_u64<'a>(&'a self, _event: &'a Event) -> RadeResult<u64> {
        Err("Not implemented".into())
    }
    fn as_u64_list<'a>(&'a self, _event: &'a Event) -> RadeResult<&'a Vec<u64>> {
        Err("Not implemented".into())
    }
    fn as_f64<'a>(&'a self, _event: &'a Event) -> RadeResult<f64> {
        Err("Not implemented".into())
    }
    fn as_bool<'a>(&'a self, _event: &'a Event) -> RadeResult<bool> {
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
    fn cmp<'a>(&'a self, _elem: &Num, _event: &'a Event, _coparator: &Comparator) -> RadeResult<bool> {
        Err(format!("Not a number").into())
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub enum Val {
    Int(Int),
    IntList(IntList),
    Float(Float),
    Str(Str),
    StrList(StrList),
    Regex(RadeRegex),
    Field(Field),
    Bool(Bool),
}

impl Val {
    pub fn into_num(self) -> RadeResult<Num> {
        match self {
            Val::Int(int) => Ok(Num::Int(int.clone())),
            Val::Float(float) => Ok(Num::Float(float.clone())),
            Val::Field(field) => Ok(Num::Field(field.clone())),
            _ => Err(format!("Not a number: {:?}", self).into()),
        }
    }

    pub fn validate_bool(self) -> RadeResult<Val> {
        match self {
             Val::Bool(_) => Ok(self),
             Val::Field(_) => Ok(self),
             _ => Err(format!("Not a boolean: {:?}", self).into()),
         }
    }
}
impl Contains for Val {
    fn contains<'a>(&self, elem: &Val, event: &Event, comp_flag: &Option<InsensitiveFlag>) -> RadeResult<bool> {
        match self {
            Val::Int(val) => val.contains(elem, event, comp_flag),
            Val::Float(val) => val.contains(elem, event, comp_flag),
            Val::Str(val) => val.contains(elem, event, comp_flag),
            Val::IntList(val) => val.contains(elem, event, comp_flag),
            Val::StrList(val) => val.contains(elem, event, comp_flag),
            Val::Field(field) => field.contains(elem, event, comp_flag),
            Val::Bool(_) => Err("Contains for bool not implemented".into()),
            Val::Regex(_) => Err("Contains for regex not implemented".into()),
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
        }
    }

    fn neq<'a>(
        &'a self,
        elem: &Val,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        match self {
            Val::Int(int) => int.neq(elem, event, comp_flag),
            Val::Float(float) => float.neq(elem, event, comp_flag),
            Val::IntList(_) => Err("Neq for intlist not implemented".into()),
            Val::Str(str) => str.neq(elem, event, comp_flag),
            Val::StrList(_) => Err("Neq for strlist not implemented".into()),
            Val::Field(field) => field.neq(elem, event, comp_flag),
            Val::Bool(b) => b.neq(elem, event, comp_flag),
            Val::Regex(_) => Err("Neq for regex not implemented".into()),
        }
    }
}

impl Match for Val {
    fn match_<'a>(
        &'a self,
        elem: &Field,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        match self {
            Val::Regex(i) => i.match_(elem, event, comp_flag),
            _ => Err("Match implemented only for Regex".into()),
        }
    }

    fn not_match<'a>(
        &'a self,
        elem: &Field,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        match self {
            Val::Regex(i) => i.not_match(elem, event, comp_flag),
            _ => Err("Not match implemented only for Regex".into()),
        }
    }
}

impl Cast for Val {
    fn as_u64<'a>(&'a self, event: &'a Event) -> RadeResult<u64> {
        match self {
            Val::Int(int) => int.as_u64(event),
            Val::Float(float) => float.as_u64(event),
            _ => Err(format!("Cannot convert {:?} to u64", self).into()),
        }
    }

    fn as_f64<'a>(&'a self, event: &'a Event) -> RadeResult<f64> {
        match self {
            Val::Int(int) => int.as_f64(event),
            Val::Float(float) => float.as_f64(event),
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
            _ => Err(format!("Cannot convert {:?} to &str", self).into()),
        }
    }

    fn as_u64_list<'a>(&'a self, event: &'a Event) -> RadeResult<&'a Vec<u64>> {
        match self {
            Val::Int(i) => i.as_u64_list(event),
            Val::IntList(i) => i.as_u64_list(event),
            _ => Err(format!("Cannot convert {:?} to Vec<u64>", self).into()),
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

    fn as_bool<'a>(&'a self, event: &'a Event) -> RadeResult<bool> {
        match self {
            Val::Bool(b) => b.as_bool(event),
            Val::Field(f) => f.as_bool(event),
            _ => Err("Type mismatch. Expected boolean.".into()),
        }
    }
}

impl From<&YamlValue> for Val {
    fn from(value: &YamlValue) -> Self {
        match value {
            YamlValue::Number(n) => if n.is_f64() {
                Val::Float(Float(n.as_f64().unwrap()))
            } else {
                Val::Int(Int(n.as_u64().unwrap()))
            },
            YamlValue::String(s) => Val::Str(Str(FatString::from(s))),
            YamlValue::Bool(b) => Val::Bool(Bool(*b)),
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
