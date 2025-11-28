use serde::{Deserialize, Serialize};

use super::{Int, IntList, Str, StrList, Val};

#[derive(Debug, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct Field(String);
impl From<&str> for Field {
    fn from(s: &str) -> Self {
        Field(s.to_string())
    }
}

impl Field {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Deserialize, Serialize)]
enum ValSerialized {
    Int(u64),
    FieldInt(Field),
    IntList(Vec<u64>),
    Str(String),
    FieldStr(Field),
    StrList(Vec<String>),
    FieldStrList(Field),
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
            ValSerialized::Int(i) => Val::Int(Int::Lit(i)),
            ValSerialized::IntList(v) => Val::IntList(IntList(v)),
            ValSerialized::Str(s) => Val::Str(Str::Lit(s.into())),
            ValSerialized::StrList(v) => {
                Val::StrList(StrList::Lit(v.into_iter().map(|s| s.into()).collect()))
            },
            ValSerialized::FieldInt(field) => Val::Int(Int::Field(field)),
            ValSerialized::FieldStr(field) => Val::Str(Str::Field(field)),
            ValSerialized::FieldStrList(field) => Val::StrList(StrList::Field(field)),
        }
    }
}
impl serde::Serialize for Val {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let val_ser = match self {
            Val::Int(Int::Lit(i)) => ValSerialized::Int(*i),
            Val::IntList(IntList(v)) => ValSerialized::IntList(v.clone()),
            Val::Str(Str::Lit(s)) => ValSerialized::Str(s.plain.clone()),
            Val::StrList(StrList::Lit(v)) => {
                ValSerialized::StrList(v.iter().map(|fs| fs.plain.clone()).collect())
            },
            Val::Int(Int::Field(f)) => ValSerialized::FieldInt(f.clone()),
            Val::Str(Str::Field(f)) => ValSerialized::FieldStr(f.clone()),
            Val::StrList(StrList::Field(f)) => ValSerialized::FieldStrList(f.clone()),
        };
        val_ser.serialize(serializer)
    }
}