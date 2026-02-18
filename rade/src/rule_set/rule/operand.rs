mod val;
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use serde::{Deserialize, Serialize};
pub use val::{Cast, Comparator, Val};
use val::{Compare, Contains, Eq, Field, Match, Num, RadeRegex, Str};

use crate::{Event, InsensitiveFlag};

pub type OpHash = u64;

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
    pub fn evaluate(&self, event: &Event) -> bool {
        match &self.op {
            Operand::Or(operands) => {
                let mut result = false;
                for operand in operands {
                    if operand.evaluate(event) {
                        result = true;
                        break;
                    }
                }
                result
            },
            Operand::And(operands) => {
                let mut result = true;
                for operand in operands {
                    if !operand.evaluate(event) {
                        result = false;
                        break;
                    }
                }
                result
            },
            Operand::Eq(val1, val2, flag) => val1.equal(val2, event, flag),
            Operand::Neq(val1, val2, flag) => val1.neq(val2, event, flag),
            Operand::Ncmp(val1, val2, coparator) => {
                val1.ncmp(val2, event, coparator).unwrap_or(false)
            },
            Operand::StartsWith(val1, val2, flag) => val1.starts_with(val2, event, flag),
            Operand::EndsWith(val1, val2, flag) => val1.ends_with(val2, event, flag),
            Operand::Contains(val1, val2, flag) => val1.contains(val2, event, flag),
            Operand::Match(field, regex, flag) => regex.match_(field, event, flag),
            Operand::NotMatch(field, regex, flag) => regex.not_match(field, event, flag),
        }
    }

    pub fn eval_with_cache(&self, event: &Event, cache: &mut HashMap<OpHash, bool>) -> bool {
        if let Some(cached_result) = cache.get(&self.hash()) {
            return *cached_result;
        }

        let res = self.evaluate(event);
        cache.insert(self.hash(), res);
        res
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
    Eq(Val, Val, #[serde(default)] Option<InsensitiveFlag>),
    Neq(Val, Val, #[serde(default)] Option<InsensitiveFlag>),
    Ncmp(Num, Num, Comparator),
    StartsWith(Str, Str, #[serde(default)] Option<InsensitiveFlag>),
    EndsWith(Str, Str, #[serde(default)] Option<InsensitiveFlag>),
    Contains(Val, Val, #[serde(default)] Option<InsensitiveFlag>),
    Match(Field, RadeRegex, #[serde(default)] Option<InsensitiveFlag>),
    NotMatch(Field, RadeRegex, #[serde(default)] Option<InsensitiveFlag>),
}

impl Operand {
    pub fn gen_hash(&self) -> OpHash {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}
