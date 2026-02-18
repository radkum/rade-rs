use serde::{Deserialize, Serialize};

use super::{Cast, Comparator, Compare, Contains, Eq, InsensitiveFlag, Num, Val};
use crate::{Event, Result};

pub(super) fn float_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Float(pub f64);
impl From<f64> for Float {
    fn from(i: f64) -> Self {
        Float(i)
    }
}

impl std::hash::Hash for Float {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Convert the f64 to its bit representation and hash that
        self.0.to_bits().hash(state);
    }
}

impl Cast for Float {
    fn as_u64<'a>(&'a self, _: &'a Event) -> Option<u64> {
        Some(self.0 as u64)
    }

    fn as_f64<'a>(&'a self, _: &'a Event) -> Option<f64> {
        Some(self.0)
    }
}

impl Eq for Float {
    fn equal(&self, elem: &Val, event: &Event, _: &Option<InsensitiveFlag>) -> bool {
        let (Some(i1), Some(i2)) = (self.as_f64(event), elem.as_f64(event)) else {
            return false;
        };
        float_eq(i1, i2)
    }

    fn neq(&self, elem: &Val, event: &Event, _: &Option<InsensitiveFlag>) -> bool {
        !self.equal(elem, event, &None)
    }
}

impl Contains for Float {
    fn contains(&self, elem: &Val, event: &Event, _: &Option<InsensitiveFlag>) -> bool {
        self.equal(elem, event, &None)
    }
}

impl Compare for Float {
    fn ncmp<'a>(&'a self, elem: &Num, event: &'a Event, coparator: &Comparator) -> Result<bool> {
        let (Some(i1), Some(i2)) = (self.as_f64(event), elem.as_f64(event)) else {
            return Err(format!("Not a float").into());
        };
        Ok(match coparator {
            Comparator::Gt => i1 > i2,
            Comparator::Lt => i1 < i2,
            Comparator::Ge => i1 >= i2,
            Comparator::Le => i1 <= i2,
        })
    }
}
