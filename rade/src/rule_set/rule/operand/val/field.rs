use serde::{Deserialize, Serialize};

use super::{Cast, Contains, Eq, InsensitiveFlag, Val};
use crate::{Event, FatString, RadeResult, rule_set::rule::operand::val::Compare};

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
    ) -> RadeResult<&'a str> {
        let str_field = event.get_str_field(&self.0)?;
        Ok(str_field.choose(comp_flag))
    }

    fn as_str_list<'a>(
        &'a self,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<Vec<&'a str>> {
        let str_list = event.get_strlist_field(&self.0)?;
        Ok(str_list.iter().map(|s| s.choose(comp_flag)).collect())
    }

    fn as_u64<'a>(&'a self, event: &'a Event) -> RadeResult<u64> {
        event.get_int_field(&self.0)
    }

    fn as_bool<'a>(&'a self, event: &'a Event) -> RadeResult<bool> {
        event.get_bool_field(&self.0)
    }

    fn as_u64_list<'a>(&'a self, event: &'a Event) -> RadeResult<&'a Vec<u64>> {
        event.get_intlist_field(&self.0)
    }
}

impl Eq for Field {
    fn equal<'a>(
        &'a self,
        elem: &Val,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        let val = event.get_field(&self.0)?;
        val.equal(elem, event, comp_flag)
    }

    fn neq<'a>(
        &'a self,
        elem: &Val,
        event: &'a Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        let val = event.get_field(&self.0)?;
        val.neq(elem, event, comp_flag)
    }
}

impl Contains for Field {
    fn contains(
        &self,
        elem: &Val,
        event: &crate::Event,
        comp_flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        let val = event.get_field(&self.0)?;
        val.contains(elem, event, comp_flag)
    }
}

impl Compare for Field {
    fn cmp<'a>(
        &'a self,
        elem: &super::Num,
        event: &'a Event,
        comparator: &super::Comparator,
    ) -> RadeResult<bool> {
        let val = event.get_field(&self.0)?;
        let num = val.clone().into_num()?;
        num.cmp(elem, event, comparator)
    }
}