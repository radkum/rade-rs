use serde::{Deserialize, Serialize};

use super::{Cast, Compare, Field, float::Float, int::Int};
use crate::{Event, RadeResult};

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
    fn cmp(&self, elem: &Num, event: &Event, coparator: &Comparator) -> RadeResult<bool> {
        match self {
            Num::Int(i) => i.cmp(elem, event, coparator),
            Num::Float(f) => f.cmp(elem, event, coparator),
            Num::Field(field) => {
                let val = event.get_field(&field.0)?.clone().into_num()?;
                val.cmp(elem, event, coparator)
            },
        }
    }
}

impl Cast for Num {
    fn as_u64<'a>(&'a self, event: &'a Event) -> RadeResult<u64> {
        match self {
            Num::Int(i) => i.as_u64(event),
            Num::Float(f) => f.as_u64(event),
            Num::Field(field) => event.get_field(&field.0)?.as_u64(event),
        }
    }

    fn as_f64<'a>(&'a self, event: &'a Event) -> RadeResult<f64> {
        match self {
            Num::Int(i) => i.as_f64(event),
            Num::Float(f) => f.as_f64(event),
            Num::Field(field) => event.get_field(&field.0)?.as_f64(event),
        }
    }
}
