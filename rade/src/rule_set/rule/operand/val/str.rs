use serde::{Deserialize, Serialize};

use super::{Cast, Comparator, Compare, InsensitiveFlag, Val};
use crate::prelude::*;
use crate::{Event, FatString, RadeResult};

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
    pub fn contains(
        &self,
        elem: &Val,
        event: &Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        let s1 = self.as_str(event, comp_flag)?;
        let s2 = elem.as_str(event, comp_flag)?;
        //log::trace!("Checking if '{}' contains '{}'", s1, s2);
        Ok(s1.contains(s2))
    }

    pub fn starts_with(
        &self,
        elem: &Str,
        event: &Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        let s1 = self.as_str(event, comp_flag)?;
        let s2 = elem.as_str(event, comp_flag)?;
        Ok(s1.starts_with(s2))
    }

    pub fn ends_with(
        &self,
        elem: &Str,
        event: &Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        let s1 = self.as_str(event, comp_flag)?;
        let s2 = elem.as_str(event, comp_flag)?;
        Ok(s1.ends_with(&s2))
    }
}

impl Cast for Str {
    fn as_str<'a>(
        &'a self,
        _: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<&'a str> {
        let fat_string = &self.0;
        Ok(fat_string.choose(comp_flag))
    }
}

impl Compare for Str {
    fn cmp<'a>(
        &'a self,
        elem: &Val,
        event: &'a Event,
        comparator: &Comparator,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        let s1 = self.as_str(event, comp_flag)?;
        let s2 = elem.as_str(event, comp_flag)?;
        match comparator {
            Comparator::Eq => Ok(s1 == s2),
            Comparator::Neq => Ok(s1 != s2),
            _ => Err(format!("Invalid comparator for strings: {:?}", comparator).into()),
        }
    }
}
