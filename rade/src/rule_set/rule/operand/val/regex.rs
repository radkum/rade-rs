use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{Cast, InsensitiveFlag, Match, Val};
use crate::{Comparator, Event, FatRegex, RadeResult};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub struct RadeRegex(pub FatRegex, pub Option<InsensitiveFlag>);
impl RadeRegex {
    pub fn new(s: &str, flag: &str) -> RadeResult<Self> {
        let mut regex_str = s.to_string();
        let flag = flag.to_string();
        if flag.contains("m") {
            regex_str = format!("(?m){}", regex_str);
        }
        if flag.contains("s") {
            regex_str = format!("(?s){}", regex_str);
        }
        let flag = flag
            .chars()
            .filter(|&c| c == 'i' || c == 'a')
            .collect::<String>();
        Ok(RadeRegex(
            FatRegex::from_str(&regex_str)?,
            InsensitiveFlag::from_regex_flags(flag.as_str()),
        ))
    }

    pub fn from_str(s: &str) -> RadeResult<Self> {
        Ok(RadeRegex(FatRegex::from_str(s)?, None))
    }
}

impl Match for RadeRegex {
    fn match_<'a>(
        &'a self,
        elem: &Val,
        event: &'a Event,
        comparator: &Comparator,
    ) -> RadeResult<bool> {
        let re = self.as_regex(event);
        let s1 = elem.as_str(event, &self.1)?;

        match comparator {
            Comparator::Match => Ok(re.is_match(s1)),
            Comparator::Nmatch => Ok(!re.is_match(s1)),
            _ => Err(format!(
                "Comparator {:?} not supported for regex matching",
                comparator
            )
            .into()),
        }
    }
}

impl RadeRegex {
    pub fn as_regex<'a>(&'a self, _: &'a Event) -> &'a Regex {
        self.0.choose(&self.1)
    }
}
