use serde::{Deserialize, Serialize};

use super::{Cast, Contains, Eq, InsensitiveFlag, Val};
use crate::Event;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub struct Int(pub u64);
impl From<u64> for Int {
    fn from(i: u64) -> Self {
        Int(i)
    }
}

impl Cast for Int {
    fn as_u64<'a>(&'a self, _: &'a Event) -> Option<u64> {
        Some(self.0)
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
