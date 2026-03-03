use serde::{Deserialize, Serialize};

use super::{Cast, InsensitiveFlag, Val};
use crate::{Event, RadeResult, rule_set::rule::operand::val::Contains};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub struct IntList(pub Vec<i64>);
impl From<Vec<i64>> for IntList {
    fn from(v: Vec<i64>) -> Self {
        IntList(v)
    }
}

impl IntList {
    pub fn new(list: Vec<i64>) -> Self {
        Self(list)
    }
    pub fn eq(&self, elem: &Val, event: &Event) -> RadeResult<bool> {
        let i1 = self.as_i64_list(event)?;
        let i2 = elem.as_i64_list(event)?;
        Ok(i1 == i2)
    }

    pub fn get(&self, index: usize) -> RadeResult<&i64> {
        self.0
            .get(index)
            .ok_or_else(|| format!("Index out of bounds").into())
    }
}

impl Cast for IntList {
    fn as_i64<'a>(&'a self, _event: &'a Event) -> RadeResult<i64> {
        if self.0.len() == 1 {
            Ok(self.0[0].into())
        } else {
            Err(format!("Not a single value").into())
        }
    }

    fn as_i64_list<'a>(&'a self, _event: &'a Event) -> RadeResult<&'a Vec<i64>> {
        Ok(&self.0)
    }
}

impl Contains for IntList {
    fn contains(&self, elem: &Val, event: &Event, _: &Option<InsensitiveFlag>) -> RadeResult<bool> {
        let elem = elem.as_i64(event)?;
        Ok(self.0.contains(&elem))
    }
}
