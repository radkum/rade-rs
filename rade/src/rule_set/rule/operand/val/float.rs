use serde::{Deserialize, Serialize};

use super::{Cast, Comparator, Compare, InsensitiveFlag, Val};
use crate::{Event, RadeResult};

pub(super) fn float_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Float(pub f64);
impl From<f64> for Float {
    fn from(i: f64) -> Self {
        Float(i)
    }
}

impl std::hash::Hash for Float {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Convert the f64 to its bit representation and hash that
        self.0.to_bits().hash(state);
    }
}

impl Cast for Float {
    fn as_i64<'a>(&'a self, _: &'a Event) -> RadeResult<i64> {
        Ok(self.0 as i64)
    }

    fn as_f64<'a>(&'a self, _: &'a Event) -> RadeResult<f64> {
        Ok(self.0)
    }
}

impl Compare for Float {
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
        if let Val::Field(field) = elem {
            field.cmp(&Val::Float(self.0.into()), event, &coparator.swap(), flag)
        } else {
            let i1 = self.as_f64(event)?;
            let i2 = elem.as_f64(event)?;

            Ok(match coparator {
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
            })
        }
    }
}
