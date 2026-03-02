use serde::{Deserialize, Serialize};

use super::{Cast, Comparator, Compare, Contains, Eq, InsensitiveFlag, Num, Val};
use crate::{Event, RadeResult};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub struct Int(pub u64);
impl From<u64> for Int {
    fn from(i: u64) -> Self {
        Int(i)
    }
}

impl Cast for Int {
    fn as_u64<'a>(&'a self, _: &'a Event) -> RadeResult<u64> {
        Ok(self.0)
    }

    fn as_f64<'a>(&'a self, _: &'a Event) -> RadeResult<f64> {
        Ok(self.0 as f64)
    }
}

impl Eq for Int {
    fn equal(&self, elem: &Val, event: &Event, _: &Option<InsensitiveFlag>) -> RadeResult<bool> {
        Ok(self.as_u64(event)? == elem.as_u64(event)?)
    }

    fn neq(&self, elem: &Val, event: &Event, _: &Option<InsensitiveFlag>) -> RadeResult<bool> {
        self.equal(elem, event, &None).map(|eq| !eq)
    }
}

impl Contains for Int {
    fn contains(&self, elem: &Val, event: &Event, _: &Option<InsensitiveFlag>) -> RadeResult<bool> {
        self.equal(elem, event, &None)
    }
}

impl Compare for Int {
    fn cmp<'a>(&'a self, elem: &Num, event: &'a Event, coparator: &Comparator) -> RadeResult<bool> {
        Ok(if let Num::Int(i2) = elem {
            match coparator {
                Comparator::Gt => self.0 > i2.0,
                Comparator::Lt => self.0 < i2.0,
                Comparator::Ge => self.0 >= i2.0,
                Comparator::Le => self.0 <= i2.0,
            }
        } else {
            let i1 = self.as_f64(event)?;
            let i2 = elem.as_f64(event)?;
            match coparator {
                Comparator::Gt => i1 > i2,
                Comparator::Lt => i1 < i2,
                Comparator::Ge => i1 >= i2,
                Comparator::Le => i1 <= i2,
            }
        })
    }
}
