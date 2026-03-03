use serde::{Deserialize, Serialize};

use super::{Cast, Contains, InsensitiveFlag, Val};
use crate::{FatString, RadeResult};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub struct StrList(pub Vec<FatString>);
impl StrList {
    pub fn get(&self, index: usize) -> RadeResult<&FatString> {
        self.0
            .get(index)
            .ok_or_else(|| format!("Index out of bounds").into())
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

impl Contains for StrList {
    fn contains(
        &self,
        elem: &Val,
        event: &crate::Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        let str_list = self.as_str_list(event, comp_flag)?;
        let str = elem.as_str(event, comp_flag)?;
        Ok(str_list.contains(&str))
    }
}
