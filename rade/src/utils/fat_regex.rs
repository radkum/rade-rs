#![allow(unused)]
use std::hash::Hash;

use regex::Regex;
use serde::Deserialize;

use super::InsensitiveFlag;

#[derive(Clone)]
pub struct FatRegex {
    pub plain: Regex,
    pub case_insensitive: Regex,
    pub apostrophe_insensitive: Regex,
    pub ac_insensitive: Regex,
}

impl core::cmp::PartialEq for FatRegex {
    fn eq(&self, other: &Self) -> bool {
        self.plain.as_str() == other.plain.as_str()
    }
}

impl Hash for FatRegex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.plain.as_str().hash(state);
    }
}

impl alloc::fmt::Debug for FatRegex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FatRegex({})", self.plain)
    }
}

impl<'de> Deserialize<'de> for FatRegex {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FatRegex::from_str(s.as_str()).map_err(serde::de::Error::custom)
    }
}

impl serde::Serialize for FatRegex {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.plain.as_str())
    }
}

impl FatRegex {
    pub fn from_str(re: &str) -> Result<Self, Box<dyn core::error::Error>> {
        let case_insensitive = re.to_lowercase();
        let apostrophe_insensitive = re.replace("'", "\"");
        let ac_insensitive = apostrophe_insensitive.to_lowercase();
        Ok(Self {
            plain: Regex::new(re).map_err(|e| format!("Failed to compile regex: {}", e))?,
            case_insensitive: Regex::new(&case_insensitive)
                .map_err(|e| format!("Failed to compile regex: {}", e))?,
            apostrophe_insensitive: Regex::new(&apostrophe_insensitive)
                .map_err(|e| format!("Failed to compile regex: {}", e))?,
            ac_insensitive: Regex::new(&ac_insensitive)
                .map_err(|e| format!("Failed to compile regex: {}", e))?,
        })
    }
}

impl FatRegex {
    pub fn choose<'a>(&'a self, comp_flag: &Option<InsensitiveFlag>) -> &'a Regex {
        if let Some(comp_flag) = comp_flag {
            match comp_flag {
                InsensitiveFlag::CaseAndApostrophe => &self.ac_insensitive,
                InsensitiveFlag::Case => &self.case_insensitive,
                InsensitiveFlag::Apostrophe => &self.apostrophe_insensitive,
            }
        } else {
            &self.plain
        }
    }

    pub fn plain(&self) -> &Regex {
        &self.plain
    }

    pub fn lowercased(&self) -> &Regex {
        &self.case_insensitive
    }
}
