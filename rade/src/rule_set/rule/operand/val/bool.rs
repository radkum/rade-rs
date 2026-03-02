use serde::{Deserialize, Serialize};

use super::{Cast, Eq, InsensitiveFlag, Val};
use crate::Event;
use crate::RadeResult;

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