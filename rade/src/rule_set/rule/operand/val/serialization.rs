use serde::{Deserialize, Serialize};

use super::{RadeRegex, Val};

#[derive(Debug, Deserialize, Serialize)]
enum ValSerialized {
    Int(u64),
    IntList(Vec<u64>),
    Float(f64),
    Str(String),
    StrList(Vec<String>),
    Bool(bool),
    Field(String),
    Regex(String),
}

impl<'de> serde::Deserialize<'de> for Val {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val_helper = ValSerialized::deserialize(deserializer)?;
        Ok(Val::from(val_helper).map_err(serde::de::Error::custom)?)
    }
}

impl Val {
    fn from(val_ser: ValSerialized) -> Result<Self, Box<dyn std::error::Error>> {
        match val_ser {
            ValSerialized::Int(i) => Ok(Val::Int(i.into())),
            ValSerialized::IntList(v) => Ok(Val::IntList(v.into())),
            ValSerialized::Str(s) => Ok(Val::Str(s.into())),
            ValSerialized::StrList(v) => Ok(Val::StrList(v.into())),
            ValSerialized::Field(f) => Ok(Val::Field(f.into())),
            ValSerialized::Regex(f) => Ok(Val::Regex(RadeRegex::from_str(&f)?)),
            ValSerialized::Float(f) => Ok(Val::Float(f.into())),
            ValSerialized::Bool(b) => Ok(Val::Bool(b.into())),
        }
    }
}
impl serde::Serialize for Val {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let val_ser = match self {
            Val::Int(i) => ValSerialized::Int(i.0),
            Val::IntList(il) => ValSerialized::IntList(il.0.clone()),
            Val::Float(f) => ValSerialized::Float(f.0 as f64),
            Val::Str(s) => ValSerialized::Str(s.0.plain().to_string()),
            Val::StrList(sl) => {
                ValSerialized::StrList(sl.0.iter().map(|fs| fs.plain().to_string()).collect())
            },
            Val::Field(f) => ValSerialized::Field(f.plain().to_string()),
            Val::Regex(f) => ValSerialized::Regex(f.0.plain().to_string()),
            Val::Bool(b) => ValSerialized::Bool(b.0),
        };
        val_ser.serialize(serializer)
    }
}
