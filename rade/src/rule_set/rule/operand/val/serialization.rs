// use serde::{Deserialize, Serialize};

// use super::{RadeRegex, Val};

// #[derive(Debug, Deserialize, Serialize)]
// enum ValSerialized {
//     Bool(bool),
//     Int(i64),
//     IntList(Vec<i64>),
//     Float(f64),
//     Str(String),
//     StrList(Vec<String>),
//     Regex(String),
//     Field(String),
//     FieldIndex(String, i64),
// }

// impl<'de> serde::Deserialize<'de> for Val {
//     fn deserialize<D>(deserializer: D) -> core::result::Result<Self,
// D::Error>     where
//         D: serde::Deserializer<'de>,
//     {
//         let val_helper = ValSerialized::deserialize(deserializer)?;
//         Ok(Val::from(val_helper).map_err(serde::de::Error::custom)?)
//     }
// }

// impl Val {
//     fn from(val_ser: ValSerialized) -> Result<Self, Box<dyn
// std::error::Error>> {         match val_ser {
//             ValSerialized::Bool(b) => Ok(Val::Bool(b.into())),
//             ValSerialized::Int(i) => Ok(Val::Int(i.into())),
//             ValSerialized::IntList(v) => Ok(Val::IntList(v.into())),
//             ValSerialized::Str(s) => Ok(Val::Str(s.into())),
//             ValSerialized::StrList(v) => Ok(Val::StrList(v.into())),
//             ValSerialized::Float(f) => Ok(Val::Float(f.into())),
//             ValSerialized::Regex(f) =>
// Ok(Val::Regex(RadeRegex::from_str(&f)?)),             ValSerialized::Field(f)
// => Ok(Val::Field(f.into())),             ValSerialized::FieldIndex(f, i) =>
// Ok(Val::FieldIndex(f.into(), i)),         }
//     }
// }
// impl serde::Serialize for Val {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let val_ser = match self {
//             Val::Bool(b) => ValSerialized::Bool(b.0),
//             Val::Int(i) => ValSerialized::Int(i.0),
//             Val::IntList(il) => ValSerialized::IntList(il.0.clone()),
//             Val::Float(f) => ValSerialized::Float(f.0 as f64),
//             Val::Str(s) => ValSerialized::Str(s.0.plain().to_string()),
//             Val::StrList(sl) => {
//                 ValSerialized::StrList(sl.0.iter().map(|fs|
// fs.plain().to_string()).collect())             },
//             Val::Regex(f) => ValSerialized::Regex(f.to_string()),
//             Val::Field(f) => ValSerialized::Field(f.plain().to_string()),
//             Val::FieldIndex(field, index) => {
//                 ValSerialized::FieldIndex(field.plain().to_string(), *index)
//             },
//             Val::Expression(expr) => *expr.op.serialize(serializer)
//         };
//         val_ser.serialize(serializer)
//     }
// }
