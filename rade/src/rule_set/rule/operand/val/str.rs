use serde::{Deserialize, Serialize};

use super::{Cast, Eq, InsensitiveFlag, Val};
use crate::{Event, FatString};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub struct Str(pub FatString);
impl From<&str> for Str {
    fn from(s: &str) -> Self {
        Str(FatString::from(s))
    }
}
impl From<String> for Str {
    fn from(s: String) -> Self {
        Str(FatString::from(s))
    }
}

impl Str {
    pub fn contains(&self, elem: &Val, event: &Event, comp_flag: &Option<InsensitiveFlag>) -> bool {
        let (Some(s1), Some(s2)) = (self.as_str(event, comp_flag), elem.as_str(event, comp_flag))
        else {
            return false;
        };
        //log::trace!("Checking if '{}' contains '{}'", s1, s2);
        s1.contains(s2)
    }

    pub fn starts_with(
        &self,
        elem: &Str,
        event: &Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> bool {
        let (Some(s1), Some(s2)) = (self.as_str(event, comp_flag), elem.as_str(event, comp_flag))
        else {
            return false;
        };
        s1.starts_with(s2)
    }

    pub fn ends_with(
        &self,
        elem: &Str,
        event: &Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> bool {
        let (Some(s1), Some(s2)) = (self.as_str(event, comp_flag), elem.as_str(event, comp_flag))
        else {
            return false;
        };
        s1.ends_with(&s2)
    }
}

impl Cast for Str {
    fn as_str<'a>(&'a self, _: &'a Event, comp_flag: &Option<InsensitiveFlag>) -> Option<&'a str> {
        let fat_string = &self.0;

        Some(fat_string.choose(comp_flag))
    }
}

impl Eq for Str {
    fn equal<'a>(
        &'a self,
        elem: &Val,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> bool {
        let (Some(s1), Some(s2)) = (self.as_str(event, comp_flag), elem.as_str(event, comp_flag))
        else {
            return false;
        };
        s1 == s2
    }

    fn neq<'a>(
        &'a self,
        elem: &Val,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> bool {
        let (Some(s1), Some(s2)) = (self.as_str(event, comp_flag), elem.as_str(event, comp_flag))
        else {
            return false;
        };
        s1 != s2
    }
}
