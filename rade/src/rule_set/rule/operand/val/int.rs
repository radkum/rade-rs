use serde::{Deserialize, Serialize};

use super::{Cast, Comparator, Compare, InsensitiveFlag, Val, float::float_eq};
use crate::{Event, RadeResult, prelude::*};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub struct Int(pub i64);
impl From<i64> for Int {
    fn from(i: i64) -> Self {
        Int(i)
    }
}

impl Cast for Int {
    fn as_i64<'a>(&'a self, _: &'a Event) -> RadeResult<i64> {
        Ok(self.0)
    }

    fn as_f64<'a>(&'a self, _: &'a Event) -> RadeResult<f64> {
        Ok(self.0 as f64)
    }
}

impl Compare for Int {
    fn cmp<'a>(
        &'a self,
        elem: &Val,
        event: &'a Event,
        coparator: &Comparator,
        flag: &Option<InsensitiveFlag>,
    ) -> RadeResult<bool> {
        if flag.is_some() {
            return Err("Cannot use case-insensitive flag with numbers".into());
        }
        Ok(if let Val::Field(field) = elem {
            field.cmp(&Val::Int(self.0.into()), event, &coparator.swap(), flag)?
        } else if let Val::Int(i2) = elem {
            match coparator {
                Comparator::Eq => self.0 == i2.0,
                Comparator::Neq => self.0 != i2.0,
                Comparator::Gt => self.0 > i2.0,
                Comparator::Lt => self.0 < i2.0,
                Comparator::Ge => self.0 >= i2.0,
                Comparator::Le => self.0 <= i2.0,
                Comparator::Match | Comparator::Nmatch => {
                    return Err(format!(
                        "Comparator {:?} supported only for regex matching",
                        coparator
                    )
                    .into());
                },
            }
        } else {
            let i1 = self.as_f64(event)?;
            let i2 = elem.as_f64(event)?;
            match coparator {
                Comparator::Eq => float_eq(i1, i2),
                Comparator::Neq => !float_eq(i1, i2),
                Comparator::Gt => i1 > i2,
                Comparator::Lt => i1 < i2,
                Comparator::Ge => i1 >= i2,
                Comparator::Le => i1 <= i2,
                Comparator::Match | Comparator::Nmatch => {
                    return Err(format!(
                        "Comparator {:?} supported only for regex matching",
                        coparator
                    )
                    .into());
                },
            }
        })
    }
}
