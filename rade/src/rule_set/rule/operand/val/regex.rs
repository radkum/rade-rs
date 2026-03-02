use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{Cast, Field, InsensitiveFlag, Match};
use crate::{Event, FatRegex};
use crate::RadeResult;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub struct RadeRegex(pub FatRegex);
impl RadeRegex {
    pub fn from_str(s: &str) -> RadeResult<Self> {
        Ok(RadeRegex(FatRegex::from_str(s)?))
    }
}

impl Match for RadeRegex {
    fn match_<'a>(
        &'a self,
        elem: &Field,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        let re = self.as_regex(event, comp_flag);
        let s1 = elem.as_str(event, comp_flag)?;
        Ok(re.is_match(s1))
    }

    fn not_match<'a>(
        &'a self,
        elem: &Field,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        let re = self.as_regex(event, comp_flag);
        let s1 = elem.as_str(event, comp_flag)?;
        Ok(!re.is_match(s1))
    }
}

impl RadeRegex {
    pub fn as_regex<'a>(
        &'a self,
        _: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> &'a Regex {
        self.0.choose(comp_flag)
    }
}
