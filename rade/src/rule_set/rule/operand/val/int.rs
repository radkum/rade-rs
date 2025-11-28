use serde::{Deserialize, Serialize};

use super::{Cast, CastLit, Contains, Eq, InsensitiveFlag, Val, Field};
use crate::Event;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub enum Int {
    Lit(u64),
    Field(Field),
}

impl Cast for Int {
    fn as_u64<'a>(&'a self, event: &'a Event) -> Option<u64> {
        match self {
            Self::Lit(i) => Some(*i),
            Self::Field(field_name) => event.get_int_field(field_name),
        }
    }
}

impl CastLit for Int {
    fn u64_lit<'a>(&'a self) -> Option<u64> {
        match self {
            Self::Lit(i) => Some(*i),
            Self::Field(_) => None,
        }
    }
}

impl Eq for Int {
    fn equal(&self, elem: &Val, event: &Event, _: &Option<InsensitiveFlag>) -> bool {
        let (Some(i1), Some(i2)) = (self.as_u64(event), elem.as_u64(event)) else {
            return false;
        };
        i1 == i2
    }

    fn neq(&self, elem: &Val, event: &Event, _: &Option<InsensitiveFlag>) -> bool {
        !self.equal(elem, event, &None)
    }
}

impl Contains for Int {
    fn contains(&self, elem: &Val, event: &Event, _: &Option<InsensitiveFlag>) -> bool {
        self.equal(elem, event, &None)
    }
}
