mod field;
mod float;
mod int;
mod int_list;
mod regex;
mod num;
mod serialization;
mod str;
mod str_list;

pub use field::*;
pub use float::*;
pub use int::*;
pub use int_list::*;
pub use regex::RadeRegex;
pub use num::{Comparator, Num};
use serde_yaml_bw::Value as YamlValue;
pub use str::*;
pub use str_list::*;

use crate::{Event, FatString, InsensitiveFlag, Result};

pub trait Eq {
    fn equal<'a>(
        &'a self,
        _elem: &Val,
        _event: &'a Event,
        _comp_flag: &Option<InsensitiveFlag>,
    ) -> bool {
        false
    }

    fn neq<'a>(
        &'a self,
        _elem: &Val,
        _event: &'a Event,
        _comp_flag: &Option<InsensitiveFlag>,
    ) -> bool {
        false
    }
}

pub trait Match {
    fn match_<'a>(
        &'a self,
        _elem: &Field,
        _event: &'a Event,
        _comp_flag: &Option<InsensitiveFlag>,
    ) -> bool {
        false
    }

    fn not_match<'a>(
        &'a self,
        _elem: &Field,
        _event: &'a Event,
        _comp_flag: &Option<InsensitiveFlag>,
    ) -> bool {
        false
    }
}

pub trait Contains {
    fn contains<'a>(
        &'a self,
        _elem: &Val,
        _event: &'a Event,
        _comp_flag: &Option<InsensitiveFlag>,
    ) -> bool {
        false
    }
}

pub trait Cast {
    fn as_u64<'a>(&'a self, _event: &'a Event) -> Option<u64> {
        None
    }
    fn as_u64_list<'a>(&'a self, _event: &'a Event) -> Option<&'a Vec<u64>> {
        None
    }
    fn as_f64<'a>(&'a self, _event: &'a Event) -> Option<f64> {
        None
    }

    fn as_str<'a>(
        &'a self,
        _event: &'a Event,
        _comp_flag: &Option<InsensitiveFlag>,
    ) -> Option<&'a str> {
        None
    }
    fn as_str_list<'a>(
        &'a self,
        _event: &'a Event,
        _comp_flag: &Option<InsensitiveFlag>,
    ) -> Option<Vec<&'a str>> {
        None
    }
}

pub trait Compare {
    fn ncmp<'a>(&'a self, _elem: &Num, _event: &'a Event, _coparator: &Comparator) -> Result<bool> {
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
}

impl Val {
    pub fn as_num<'a>(&'a self) -> Result<Num> {
        match self {
            Val::Int(int) => Ok(Num::Int(int.clone())),
            Val::Float(float) => Ok(Num::Float(float.clone())),
            Val::Field(field) => Ok(Num::Field(field.clone())),
            _ => Err(format!("Not a number: {:?}", self).into()),
        }
    }
}
impl Contains for Val {
    fn contains<'a>(&self, elem: &Val, event: &Event, comp_flag: &Option<InsensitiveFlag>) -> bool {
        match self {
            Val::Int(val) => val.contains(elem, event, comp_flag),
            Val::Float(val) => val.contains(elem, event, comp_flag),
            Val::Str(val) => val.contains(elem, event, comp_flag),
            Val::IntList(val) => val.contains(elem, event, comp_flag),
            Val::StrList(val) => val.contains(elem, event, comp_flag),
            Val::Field(field) => field.contains(elem, event, comp_flag),
            Val::Regex(_) => false,
        }
    }
}

impl Eq for Val {
    fn equal<'a>(
        &'a self,
        elem: &Val,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> bool {
        match self {
            Val::Int(i) => i.equal(elem, event, comp_flag),
            Val::Float(f) => f.equal(elem, event, comp_flag),
            //Val::IntList(int_list) => int_list.equal(elem, event, comp_flag),
            Val::Str(s) => s.equal(elem, event, comp_flag),
            //Val::StrList(str_list) => str_list.equal(elem, event, comp_flag),
            Val::Field(field) => field.equal(elem, event, comp_flag),
            _ => false,
        }
    }

    fn neq<'a>(
        &'a self,
        elem: &Val,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> bool {
        match self {
            Val::Int(int) => int.neq(elem, event, comp_flag),
            Val::Float(float) => float.neq(elem, event, comp_flag),
            //Val::IntList(int_list) => int_list.neq(elem, event, comp_flag),
            Val::Str(str) => str.neq(elem, event, comp_flag),
            //Val::StrList(str_list) => str_list.neq(elem, event, comp_flag),
            Val::Field(field) => field.neq(elem, event, comp_flag),
            _ => false,
        }
    }
}

impl Match for Val {
    fn match_<'a>(
        &'a self,
        elem: &Field,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> bool {
        match self {
            Val::Regex(i) => i.match_(elem, event, comp_flag),
            _ => false,
        }
    }

    fn not_match<'a>(
        &'a self,
        elem: &Field,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> bool {
        match self {
            Val::Regex(i) => i.not_match(elem, event, comp_flag),
            _ => false,
        }
    }
}

impl Cast for Val {
    fn as_u64<'a>(&'a self, event: &'a Event) -> Option<u64> {
        match self {
            Val::Int(int) => int.as_u64(event),
            Val::IntList(int) => int.as_u64(event),
            Val::Float(float) => float.as_u64(event),
            _ => None,
        }
    }

    fn as_f64<'a>(&'a self, event: &'a Event) -> Option<f64> {
        match self {
            Val::Int(int) => int.as_f64(event),
            Val::Float(float) => float.as_f64(event),
            _ => None,
        }
    }

    fn as_str<'a>(
        &'a self,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> Option<&'a str> {
        match self {
            Val::Str(str) => str.as_str(event, comp_flag),
            Val::StrList(str_list) => str_list.as_str(event, comp_flag),
            Val::Field(f) => f.as_str(event, comp_flag),
            _ => None,
        }
    }

    fn as_u64_list<'a>(&'a self, event: &'a Event) -> Option<&'a Vec<u64>> {
        match self {
            Val::Int(i) => i.as_u64_list(event),
            Val::IntList(i) => i.as_u64_list(event),
            _ => None,
        }
    }

    fn as_str_list<'a>(
        &'a self,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> Option<Vec<&'a str>> {
        match self {
            Val::Str(s) => s.as_str_list(event, comp_flag),
            Val::StrList(s) => s.as_str_list(event, comp_flag),
            _ => None,
        }
    }
}

impl From<&YamlValue> for Val {
    fn from(value: &YamlValue) -> Self {
        match value {
            YamlValue::Number(n) => Val::Int(Int(n.as_u64().unwrap())),
            YamlValue::String(s) => Val::Str(Str(FatString::from(s))),
            _ => todo!(),
        }
    }
}

impl From<&Val> for YamlValue {
    fn from(value: &Val) -> Self {
        match value {
            Val::Int(i) => YamlValue::Number(i.0.into()),
            Val::Str(s) => YamlValue::String(s.0.plain().to_string()),
            _ => todo!(),
        }
    }
}
