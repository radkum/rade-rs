use serde::{Deserialize, Serialize};

use super::{Cast, Comparator, Compare, Eq, InsensitiveFlag, Val};
use crate::{Event, RadeResult};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub struct Bool(pub bool);
impl From<bool> for Bool {
    fn from(i: bool) -> Self {
        Bool(i)
    }
}

impl Cast for Bool {
    fn as_bool<'a>(&'a self, _: &'a Event) -> RadeResult<bool> {
        Ok(self.0)
    }
}

impl Eq for Bool {
    fn equal(&self, elem: &Val, event: &Event, _: &Option<InsensitiveFlag>) -> RadeResult<bool> {
        let i1 = self.as_bool(event)?;
        let i2 = elem.as_bool(event)?;
        Ok(i1 == i2)
    }

    fn neq(&self, elem: &Val, event: &Event, _: &Option<InsensitiveFlag>) -> RadeResult<bool> {
        Ok(!self.equal(elem, event, &None)?)
    }
}

impl Compare for Bool {
    fn cmp(
        &self,
        elem: &Val,
        event: &Event,
        comparator: &Comparator,
        flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        if flag.is_some() {
            return Err("Cannot apply case-insensitive flag to boolean comparison".into());
        }
        let i1 = self.as_bool(event)?;
        let i2 = elem.as_bool(event)?;
        match comparator {
            Comparator::Eq => Ok(i1 == i2),
            Comparator::Neq => Ok(i1 != i2),
            _ => Err(format!("Cannot compare bool with {:?}", comparator).into()),
        }
    }
}
