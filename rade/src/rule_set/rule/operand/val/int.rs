use serde::{Deserialize, Serialize};

use super::{Cast, Contains, Eq, InsensitiveFlag, Val};
use crate::Event;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub enum FieldInt {
    Pid,
    Tid,
    Session,
    RequestNumber,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub enum Int {
    Lit(u64),
    Field(FieldInt),
}

impl Cast for Int {
    fn as_u64<'a>(&'a self, event: &'a Event) -> Option<u64> {
        match self {
            Self::Lit(i) => Some(*i),
            Self::Field(FieldInt::Pid) => event.pid,
            Self::Field(FieldInt::Tid) => event.tid,
            Self::Field(FieldInt::Session) => event.session,
            Self::Field(FieldInt::RequestNumber) => event.request_number,
        }
    }

    // fn as_u64_list<'a>(&'a self, event: &'a Event) -> Option<&'a Vec<u64>> {
    //     self.as_u64(event).map(|i| vec![i].as_ref())
    // }
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
