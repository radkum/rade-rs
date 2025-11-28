use serde::{Deserialize, Serialize};

use super::{Cast, Contains, InsensitiveFlag, Val};
use crate::FatString;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub struct StrList(pub Vec<FatString>);
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
    ) -> Option<&'a str> {
        let list = &self.0;
        if list.is_empty() {
            return None;
        }

        Some(list[0].choose(comp_flag))
    }

    fn as_str_list<'a>(
        &'a self,
        _: &'a crate::Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> Option<Vec<&'a str>> {
        let list = &self.0;
        if list.is_empty() {
            return None;
        }

        Some(list.iter().map(|fs| fs.choose(comp_flag)).collect())
    }
}

impl Contains for StrList {
    fn contains(
        &self,
        elem: &Val,
        event: &crate::Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> bool {
        let Some(str_list) = self.as_str_list(event, comp_flag) else {
            return false;
        };

        let Some(str) = elem.as_str(event, comp_flag) else {
            return false;
        };

        str_list.contains(&str)
    }
}
