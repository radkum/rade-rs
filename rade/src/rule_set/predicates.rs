use alloc::boxed::Box;
use hashbrown::HashMap;

use super::{rule::OpHash, rules::Rules};

pub type ResultMap = HashMap<OpHash, bool>;
pub type PredType = Box<dyn Fn(&crate::Event, &mut ResultMap) -> bool>;

pub(crate) struct Predicate(pub(crate) PredType);
pub(crate) struct Predicates {
    simple_predicates: HashMap<OpHash, Predicate>,
    complex_predicates: HashMap<OpHash, Predicate>,
}

impl Predicates {
    pub(crate) fn simple(&self) -> &HashMap<OpHash, Predicate> {
        &self.simple_predicates
    }

    pub(crate) fn complex(&self) -> &HashMap<OpHash, Predicate> {
        &self.complex_predicates
    }

    pub(crate) fn from_rules(rules: &Rules) -> Self {
        let mut simple_predicates = HashMap::new();
        let mut complex_predicates = HashMap::new();

        // Logic to compile rules into a more efficient representation
        for r in rules.iter() {
            let (simple, complex) = r.operands();
            for op in simple {
                let hash = op.hash();
                simple_predicates.insert(
                    hash,
                    Predicate(Box::new(
                        move |event: &crate::Event, cache: &mut ResultMap| {
                            op.evaluate(event, cache)
                        },
                    )),
                );
            }
            for op in complex {
                let hash = op.hash();
                complex_predicates.insert(
                    hash,
                    Predicate(Box::new(
                        move |event: &crate::Event, cache: &mut ResultMap| {
                            op.evaluate(event, cache)
                        },
                    )),
                );
            }
        }
        Self {
            simple_predicates,
            complex_predicates,
        }
    }
}
