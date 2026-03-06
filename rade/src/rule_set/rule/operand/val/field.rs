use serde::{Deserialize, Serialize};

use super::{Cast, Eq, InsensitiveFlag, Val};
use crate::{
    Event, FatString, RadeResult, ResultMap,
    rule_set::rule::operand::val::{Compare, Int, Str},
};
#[derive(Debug, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub struct Field(pub String);
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
        Field(String::from(field_name))
    }

    pub fn plain(&self) -> &str {
        &self.0
    }

    pub fn get_val<'a>(&self, event: &'a Event, index: i64) -> RadeResult<Val> {
        match event.get_field(&self.0)? {
            Val::StrList(str_list) => Ok(Val::Str(Str(str_list.get(index)?.clone()))),
            Val::IntList(int_list) => Ok(Val::Int(Int(*int_list.get(index)?))),
            _ => Err(format!("Field index {} not found", index).into()),
        }
    }

    pub fn get_str<'a>(&self, event: &'a Event, index: i64) -> RadeResult<&'a FatString> {
        match event.get_field(&self.0)? {
            Val::StrList(str_list) => Ok(str_list.get(index)?),
            _ => Err(format!("Field index {} not found", index).into()),
        }
    }

    pub fn get_int<'a>(&self, event: &'a Event, index: i64) -> RadeResult<i64> {
        match event.get_field(&self.0)? {
            Val::IntList(int_list) => Ok(*int_list.get(index)?),
            _ => Err(format!("Field index {} not found", index).into()),
        }
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

    fn as_i64<'a>(&'a self, event: &'a Event) -> RadeResult<i64> {
        event.get_int_field(&self.0)
    }

    fn as_bool<'a>(&'a self, event: &'a Event, _cache: Option<&mut ResultMap>) -> RadeResult<bool> {
        event.get_bool_field(&self.0)
    }

    fn as_i64_list<'a>(&'a self, event: &'a Event) -> RadeResult<&'a Vec<i64>> {
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

// impl Contains for Field {
//     fn contains(
//         &self,
//         elem: &Val,
//         event: &crate::Event,
//         comp_flag: &Option<InsensitiveFlag>,
//     ) -> RadeResult<bool> {
//         let val = event.get_field(&self.0)?;
//         val.contains(elem, event, comp_flag)
//     }
// }

impl Compare for Field {
    fn cmp<'a>(
        &'a self,
        elem: &Val,
        event: &'a Event,
        comparator: &super::Comparator,
        flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        let val = event.get_field(&self.0)?;
        val.cmp(elem, event, comparator, flag)
    }
}
