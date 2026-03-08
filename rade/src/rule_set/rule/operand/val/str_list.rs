use serde::{Deserialize, Serialize};

use super::{Cast, InsensitiveFlag};
use crate::{FatString, RadeResult, prelude::*};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub struct StrList(pub Vec<FatString>);
impl StrList {
    pub fn get(&self, index: i64) -> RadeResult<&FatString> {
        let len = self.0.len() as i64;
        let actual_index = if index < 0 {
            // Python-style negative indexing: -1 is last element, -2 is second to last,
            // etc.
            let positive_index = len + index;
            if positive_index < 0 {
                return Err(
                    format!("Index {} out of bounds for list of length {}", index, len).into(),
                );
            }
            positive_index as usize
        } else {
            index as usize
        };
        self.0.get(actual_index).ok_or_else(|| {
            format!("Index {} out of bounds for list of length {}", index, len).into()
        })
    }
}

impl From<Vec<String>> for StrList {
    fn from(v: Vec<String>) -> Self {
        StrList(v.into_iter().map(FatString::from).collect())
    }
}

impl Cast for StrList {
    fn as_str<'a>(
        &'a self,
        _: &'a crate::Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<&'a str> {
        let list = &self.0;
        if list.is_empty() {
            return Err("StrList is empty".into());
        }

        Ok(list[0].choose(comp_flag))
    }

    fn as_str_list<'a>(
        &'a self,
        _: &'a crate::Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<Vec<&'a str>> {
        let list = &self.0;
        if list.is_empty() {
            return Err("StrList is empty".into());
        }

        Ok(list.iter().map(|fs| fs.choose(comp_flag)).collect())
    }
}
