use serde::{Deserialize, Serialize};

use super::{Cast, InsensitiveFlag, Val};
use crate::{Event, rule_set::rule::operand::val::Contains};
use crate::RadeResult;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub struct IntList(pub Vec<u64>);
impl From<Vec<u64>> for IntList {
    fn from(v: Vec<u64>) -> Self {
        IntList(v)
    }
}

impl IntList {
    pub fn new(list: Vec<u64>) -> Self {
        Self(list)
    }
    pub fn eq(&self, elem: &Val, event: &Event) -> RadeResult<bool> {
        let i1 = self.as_u64_list(event)?;
        let i2 = elem.as_u64_list(event)?;
        Ok(i1 == i2)
    }


}

impl Cast for IntList {
    fn as_u64<'a>(&'a self, _event: &'a Event) -> RadeResult<u64> {
        if self.0.len() == 1 {
            Ok(self.0[0].into())
        } else {
            Err(format!("Not a single value").into())
        }
    }

    fn as_u64_list<'a>(&'a self, _event: &'a Event) -> RadeResult<&'a Vec<u64>> {
        Ok(&self.0)
    }
}

impl Contains for IntList {
    fn contains(&self, elem: &Val, event: &Event, _: &Option<InsensitiveFlag>) -> RadeResult<bool> {
        let elem = elem.as_u64(event)?;
        Ok(self.0.contains(&elem))
    }
}
