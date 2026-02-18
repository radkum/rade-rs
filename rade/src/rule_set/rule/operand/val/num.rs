use serde::{Deserialize, Serialize};

use super::{Cast, Compare, Field, float::Float, int::Int};
use crate::{Event, Result};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub enum Comparator {
    Gt,
    Lt,
    Ge,
    Le,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub enum Num {
    Int(Int),
    Float(Float),
    Field(Field),
}

impl Compare for Num {
    fn ncmp(&self, elem: &Num, event: &Event, coparator: &Comparator) -> Result<bool> {
        match self {
            Num::Int(i) => i.ncmp(elem, event, coparator),
            Num::Float(f) => f.ncmp(elem, event, coparator),
            Num::Field(field) => {
                if let Some(val) = event.get_field(&field.0) {
                    let num = val.as_num()?;
                    num.ncmp(elem, event, coparator)
                } else {
                    Ok(false)
                }
            },
        }
    }
}

impl Cast for Num {
    fn as_u64<'a>(&'a self, event: &'a Event) -> Option<u64> {
        match self {
            Num::Int(i) => i.as_u64(event),
            Num::Float(f) => f.as_u64(event),
            Num::Field(field) => {
                if let Some(val) = event.get_field(&field.0) {
                    val.as_u64(event)
                } else {
                    None
                }
            },
        }
    }

    fn as_f64<'a>(&'a self, event: &'a Event) -> Option<f64> {
        match self {
            Num::Int(i) => i.as_f64(event),
            Num::Float(f) => f.as_f64(event),
            Num::Field(field) => {
                if let Some(val) = event.get_field(&field.0) {
                    val.as_f64(event)
                } else {
                    None
                }
            },
        }
    }
}
