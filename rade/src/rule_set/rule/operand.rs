mod val;
use core::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
pub use val::{Cast, Comparator, FnCall, MethodCall, RadeRegex, Val};
use val::{Compare, Match};

use crate::prelude::*;
use crate::{Event, InsensitiveFlag, ResultMap};

pub type OpHash = u64;

/// Simple hasher for no_std environments
#[derive(Default)]
struct SimpleHasher {
    state: u64,
}

impl Hasher for SimpleHasher {
    fn finish(&self) -> u64 {
        self.state
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.state = self.state.wrapping_mul(31).wrapping_add(*byte as u64);
        }
    }
}

#[derive(Debug, PartialEq, Clone, Hash)]
pub struct OperandContainer {
    op: Operand,
    hash: OpHash,
}

impl Serialize for OperandContainer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.op.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for OperandContainer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let op = Operand::deserialize(deserializer)?;
        Ok(OperandContainer::from(op))
    }
}

impl From<Operand> for OperandContainer {
    fn from(op: Operand) -> Self {
        let hash = op.gen_hash();
        Self { op, hash }
    }
}

impl OperandContainer {
    pub fn op(&self) -> &Operand {
        &self.op
    }

    pub fn evaluate(&self, event: &Event, cache: &mut ResultMap) -> bool {
        if let Some(cached_result) = cache.get(&self.hash()) {
            return *cached_result;
        }

        let res = match &self.op {
            Operand::Or(operands) => {
                let mut result = false;
                for operand in operands {
                    if operand.evaluate(event, cache) {
                        result = true;
                        break;
                    }
                }
                Ok(result)
            },
            Operand::And(operands) => {
                let mut result = true;
                for operand in operands {
                    if !operand.evaluate(event, cache) {
                        result = false;
                        break;
                    }
                }
                Ok(result)
            },
            Operand::Cmp(val1, val2, comparator, flag) => val1.cmp(val2, event, comparator, flag),
            Operand::Match(val, regex, comparator) => regex.match_(val, event, comparator),
            Operand::Val(val) => val.as_bool(event, Some(cache)),
            Operand::Negate(op) => Ok(!op.evaluate(event, cache)),
        };

        let e = res.unwrap_or_else(|e| {
            if e.to_string().contains("Field not found") {
                log::trace!("Error evaluating operand: {:?}, error: {}", self, e);
            } else {
                log::error!("Error evaluating operand: {:?}, error: {}", self, e);
            }
            false
        });

        cache.insert(self.hash(), e);

        e
    }

    pub fn operands(
        &self,
        simple_op_vec: &mut Vec<OperandContainer>,
        complex_op_vec: &mut Vec<OperandContainer>,
    ) {
        match &self.op {
            Operand::Or(operands) => {
                for operand in operands {
                    operand.operands(simple_op_vec, complex_op_vec);
                }
                complex_op_vec.push(self.clone());
            },
            Operand::And(operands) => {
                for operand in operands {
                    operand.operands(simple_op_vec, complex_op_vec);
                }
                complex_op_vec.push(self.clone());
            },
            _ => simple_op_vec.push(self.clone()),
        }
    }

    pub fn hash(&self) -> OpHash {
        self.hash
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Hash)]
pub enum Operand {
    And(Vec<OperandContainer>),
    Or(Vec<OperandContainer>),
    Cmp(
        Val,
        Val,
        Comparator,
        #[serde(default)] Option<InsensitiveFlag>,
    ),
    Match(Val, RadeRegex, Comparator),
    Val(Val),
    Negate(Box<OperandContainer>),
}

impl Operand {
    pub fn gen_hash(&self) -> OpHash {
        let mut hasher = SimpleHasher::default();
        self.hash(&mut hasher);
        hasher.finish()
    }
}
