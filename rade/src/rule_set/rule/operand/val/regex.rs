use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{Cast, Field, InsensitiveFlag, Match};
use crate::{Event, FatRegex};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub struct RadeRegex(pub FatRegex);
impl RadeRegex {
    pub fn from_str(s: &str) -> Result<Self, Box<dyn core::error::Error>> {
        Ok(RadeRegex(FatRegex::from_str(s)?))
    }
}

impl Match for RadeRegex {
    fn match_<'a>(
        &'a self,
        elem: &Field,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> bool {
        let (Some(re), Some(s2)) = (
            self.as_regex(event, comp_flag),
            elem.as_str(event, comp_flag),
        ) else {
            return false;
        };
        re.is_match(s2)
    }

    fn not_match<'a>(
        &'a self,
        elem: &Field,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> bool {
        let (Some(re), Some(s2)) = (
            self.as_regex(event, comp_flag),
            elem.as_str(event, comp_flag),
        ) else {
            return false;
        };
        !re.is_match(s2)
    }
}

impl RadeRegex {
    pub fn as_regex<'a>(
        &'a self,
        _: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> Option<&'a Regex> {
        Some(self.0.choose(comp_flag))
    }
}
