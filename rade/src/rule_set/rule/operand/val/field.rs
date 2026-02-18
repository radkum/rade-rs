use serde::{Deserialize, Serialize};

use super::{Cast, Contains, Eq, InsensitiveFlag, Val};
use crate::{Event, FatString, Result, rule_set::rule::operand::val::Compare};

#[derive(Debug, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct Field(pub FatString);
impl From<String> for Field {
    fn from(s: String) -> Self {
        Field::new(s)
    }
}
impl From<&str> for Field {
    fn from(s: &str) -> Self {
        Field::new(s.to_string())
    }
}

impl Field {
    pub fn new(field_name: String) -> Self {
        Field(FatString::from(field_name.to_ascii_lowercase()))
    }

    pub fn plain(&self) -> &str {
        self.0.plain()
    }
}

impl Cast for Field {
    fn as_str<'a>(
        &'a self,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> Option<&'a str> {
        event.get_str_field(&self.0).map(|fs| fs.choose(comp_flag))
    }

    fn as_str_list<'a>(
        &'a self,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> Option<Vec<&'a str>> {
        event
            .get_strlist_field(&self.0)
            .map(|fs| fs.iter().map(|s| s.choose(comp_flag)).collect())
    }

    fn as_u64<'a>(&'a self, event: &'a Event) -> Option<u64> {
        event.get_int_field(&self.0)
    }

    fn as_u64_list<'a>(&'a self, _event: &'a Event) -> Option<&'a Vec<u64>> {
        todo!()
    }
}

impl Eq for Field {
    fn equal<'a>(
        &'a self,
        elem: &Val,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> bool {
        if let Some(val) = event.get_field(&self.0) {
            val.equal(elem, event, comp_flag)
        } else {
            false
        }
    }

    fn neq<'a>(
        &'a self,
        elem: &Val,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> bool {
        if let Some(val) = event.get_field(&self.0) {
            val.neq(elem, event, comp_flag)
        } else {
            false
        }
    }
}

impl Contains for Field {
    fn contains(
        &self,
        elem: &Val,
        event: &crate::Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> bool {
        if let Some(val) = event.get_field(&self.0) {
            val.contains(elem, event, comp_flag)
        } else {
            false
        }
    }
}

impl Compare for Field {
    fn ncmp<'a>(
        &'a self,
        elem: &super::Num,
        event: &'a Event,
        comparator: &super::Comparator,
    ) -> Result<bool> {
        if let Some(val) = event.get_field(&self.0) {
            let num = val.as_num()?;
            num.ncmp(elem, event, comparator)
        } else {
            Ok(false)
        }
    }
}
