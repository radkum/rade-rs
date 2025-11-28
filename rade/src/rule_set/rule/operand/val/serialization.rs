use serde::{Deserialize, Serialize};

use super::Val;

#[derive(Debug, Deserialize, Serialize)]
enum ValSerialized {
    Int(u64),
    IntList(Vec<u64>),
    Str(String),
    StrList(Vec<String>),
    Field(String),
}

impl<'de> serde::Deserialize<'de> for Val {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val_helper = ValSerialized::deserialize(deserializer)?;
        Ok(Val::from(val_helper))
    }
}

impl From<ValSerialized> for Val {
    fn from(val_ser: ValSerialized) -> Self {
        match val_ser {
            ValSerialized::Int(i) => Val::Int(i.into()),
            ValSerialized::IntList(v) => Val::IntList(v.into()),
            ValSerialized::Str(s) => Val::Str(s.into()),
            ValSerialized::StrList(v) => Val::StrList(v.into()),
            ValSerialized::Field(f) => Val::Field(f.into()),
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
            Val::Str(s) => ValSerialized::Str(s.0.plain().to_string()),
            Val::StrList(sl) => {
                ValSerialized::StrList(sl.0.iter().map(|fs| fs.plain().to_string()).collect())
            },
            Val::Field(f) => ValSerialized::Field(f.plain().to_string()),
        };
        val_ser.serialize(serializer)
    }
}
