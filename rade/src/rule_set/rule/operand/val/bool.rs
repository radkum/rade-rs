use serde::{Deserialize, Serialize};

use super::{Cast, Comparator, Compare, InsensitiveFlag, Val};
use crate::{Event, RadeResult, ResultMap, prelude::*};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub struct Bool(pub bool);
impl From<bool> for Bool {
    fn from(i: bool) -> Self {
        Bool(i)
    }
}

impl Cast for Bool {
    fn as_bool<'a>(&'a self, _: &'a Event, _cache: Option<&mut ResultMap>) -> RadeResult<bool> {
        Ok(self.0)
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
        let i1 = self.as_bool(event, None)?;
        let i2 = elem.as_bool(event, None)?;
        match comparator {
            Comparator::Eq => Ok(i1 == i2),
            Comparator::Neq => Ok(i1 != i2),
            _ => Err(format!("Cannot compare bool with {:?}", comparator).into()),
        }
    }
}
