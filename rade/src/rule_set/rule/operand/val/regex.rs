use std::{fmt::Display, hash::Hash};

use regex::Regex;
use serde::Deserialize;

use super::{Cast, InsensitiveFlag, Match, Val};
use crate::{Comparator, Event, RadeResult};

#[derive(Debug, Clone)]
pub struct RadeRegex(pub Regex, pub Option<InsensitiveFlag>);
impl RadeRegex {
    pub fn new(s: &str, flags: &str) -> RadeResult<Self> {
        let mut regex_str = s.to_string();
        let flag = flags.to_string();
        if !flags.is_empty() {
            regex_str = format!("(?{}){}", flags, regex_str);
        }

        Ok(RadeRegex(
            Regex::new(&regex_str)?,
            InsensitiveFlag::from_regex_flags(flag.as_str()),
        ))
    }

    pub fn from_str(regex_str: &str) -> RadeResult<Self> {
        let regex_str = regex_str.trim_start_matches('/');
        let (pattern, flags) = if let Some(last_slash) = regex_str.rfind('/') {
            let (pattern, flags) = regex_str.split_at(last_slash);
            (pattern, &flags[1..]) // Skip the slash in flags part
        } else {
            (regex_str, "")
        };
        // Unescape \/ to /
        let pattern = pattern.replace("\\/", "/");
        Self::new(&pattern, flags)
    }
}

impl Display for RadeRegex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(rest) = self.0.as_str().strip_prefix("(?") {
            //flags are present
            let (flags, regex_body) = rest.split_once(")").unwrap_or_default();
            write!(f, "/{}/{}", regex_body, flags)
        } else {
            write!(f, "/{}/", self.0)
        }
    }
}

impl Hash for RadeRegex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_str().hash(state);
        self.1.hash(state);
    }
}

impl PartialEq for RadeRegex {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str() == other.0.as_str() && self.1 == other.1
    }
}

impl<'de> Deserialize<'de> for RadeRegex {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        RadeRegex::from_str(s.as_str()).map_err(serde::de::Error::custom)
    }
}

impl serde::Serialize for RadeRegex {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
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
        &self.0
    }
}
