use std::hash::Hash;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Hash)]
pub enum InsensitiveFlag {
    #[serde(rename = "CaseInsensitive")]
    Case,
    Apostrophe,
    CaseAndApostrophe,
}

impl InsensitiveFlag {
    pub fn from_regex_flags(flags: &str) -> Option<Self> {
        if flags.contains("ia") {
            Some(InsensitiveFlag::CaseAndApostrophe)
        } else if flags.contains("i") {
            Some(InsensitiveFlag::Case)
        } else if flags.contains("a") {
            Some(InsensitiveFlag::Apostrophe)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct FatString {
    pub plain: String,
    pub case_insensitive: String,
    pub apostrophe_insensitive: String,
    pub ac_insensitive: String,
}

impl core::cmp::PartialEq for FatString {
    fn eq(&self, other: &Self) -> bool {
        self.plain == other.plain
    }
}

impl Hash for FatString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.plain.hash(state);
    }
}

impl alloc::fmt::Debug for FatString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FatString({})", self.plain)
    }
}

impl<'de> Deserialize<'de> for FatString {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(FatString::from(s))
    }
}

impl serde::Serialize for FatString {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.plain)
    }
}

impl From<&str> for FatString {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
    }
}

impl From<&String> for FatString {
    fn from(s: &String) -> Self {
        Self::from(s.to_string())
    }
}

impl From<String> for FatString {
    fn from(s: String) -> Self {
        let case_insensitive = s.to_lowercase();
        let apostrophe_insensitive = s.replace("'", "\"");
        let ac_insensitive = apostrophe_insensitive.to_lowercase();
        Self {
            plain: s,
            case_insensitive,
            apostrophe_insensitive,
            ac_insensitive,
        }
    }
}

impl FatString {
    pub fn choose<'a>(&'a self, comp_flag: &Option<InsensitiveFlag>) -> &'a str {
        if let Some(comp_flag) = comp_flag {
            match comp_flag {
                InsensitiveFlag::CaseAndApostrophe => self.ac_insensitive.as_ref(),
                InsensitiveFlag::Case => self.case_insensitive.as_ref(),
                InsensitiveFlag::Apostrophe => self.apostrophe_insensitive.as_ref(),
            }
        } else {
            self.plain.as_ref()
        }
    }

    pub fn plain(&self) -> &str {
        &self.plain
    }

    pub fn lowercased(&self) -> &str {
        &self.case_insensitive
    }
}
