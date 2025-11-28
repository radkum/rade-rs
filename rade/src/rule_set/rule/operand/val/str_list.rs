use serde::{Deserialize, Serialize};

use super::{Cast, CastLit, Contains, InsensitiveFlag, Val, Field};
use crate::{Event, FatString};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub enum StrList {
    Lit(Vec<FatString>),
    Field(Field),
}

impl Cast for StrList {
    fn as_str<'a>(
        &'a self,
        event: &'a crate::Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> Option<&'a str> {
        let list = match self {
            Self::Lit(s) => Some(s),
            Self::Field(field) => event.get_strlist_field(field),
        };

        let list = list?;
        if list.is_empty() {
            return None;
        }

        Some(list[0].choose(comp_flag))
    }

    fn as_str_list<'a>(
        &'a self,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> Option<Vec<&'a str>> {
        let list = match self {
            Self::Lit(s) => Some(s),
            Self::Field(field) => event.get_strlist_field(field),
        };
        let list = list?;
        if list.is_empty() {
            return None;
        }

        Some(list.iter().map(|fs| fs.choose(comp_flag)).collect())
    }
}

impl CastLit for StrList {
    fn str_lit<'a>(
        &'a self,
    ) -> Option<&'a FatString> {
        let list = match self {
            Self::Lit(s) => Some(s),
            _ => None,
        };

        let list = list?;
        if list.is_empty() {
            return None;
        }

        Some(&list[0])
    }

    fn str_list_lit<'a>(
        &'a self,
    ) -> Option<Vec<&'a FatString>> {
        let list = match self {
            Self::Lit(s) => Some(s),
            _ => None,
        };
        let list = list?;
        if list.is_empty() {
            return None;
        }

        Some(list.iter().map(|fs| fs).collect())
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
