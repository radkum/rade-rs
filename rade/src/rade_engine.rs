use super::rule_set::{Predicates, ResultMap};
use crate::{Events, MatchedRules, Matches, RadeResult, Rules};

#[derive(Default)]
pub struct RadeEngine {
    rules: Rules,
    predicates: Option<Predicates>,
}

impl RadeEngine {
    pub fn from_rules(rules: Rules) -> Self {
        Self {
            rules,
            predicates: None,
        }
    }

    pub fn load_rules(&mut self, rules: Rules) {
        self.rules = rules;
    }

    pub fn compile_rules(&mut self) {
        self.predicates = Some(Predicates::from_rules(&self.rules));
    }

    pub fn eval_iterative(&mut self, events: Events) -> Matches {
        let mut matches = Matches::default();

        for rule in self.rules.iter() {
            for event in events.iter() {
                let mut matched_rules = MatchedRules::new();
                if rule.evaluate(event) {
                    matched_rules.add(*rule.id());
                }
                matches.add_match(event.clone(), matched_rules);
            }
        }

        matches
    }

    pub fn eval_with_predicates(&mut self, events: Events) -> RadeResult<Matches> {
        let mut matches = Matches::default();
        let Some(predicates) = &self.predicates else {
            Err("Predicates not compiled. Call compile_rules() before eval_with_predicates().")?
        };

        for event in events.get() {
            let event_matches = self.eval_one_event(&event, predicates);
            matches.add_match(event, event_matches?);
        }
        Ok(matches)
    }

    fn eval_one_event(
        &self,
        event: &crate::Event,
        predicates: &Predicates,
    ) -> RadeResult<MatchedRules> {
        let mut matched_rules = MatchedRules::new();
        let mut pred_results = ResultMap::default();

        //we want to evaluate all predicates for the event at first
        for predicate in predicates.simple().values() {
            predicate.0(event, &mut pred_results);
        }

        //if simple predicates passed, evaluate complex ones
        for predicate in predicates.complex().values() {
            predicate.0(event, &mut pred_results);
        }

        for rule in self.rules.iter() {
            let Some(res) = pred_results.get(&rule.condition_hash()) else {
                log::error!("Hash {} wasn't evaluated. Strange", rule.id());
                continue;
            };
            if *res {
                matched_rules.add(*rule.id());
            }
        }

        Ok(matched_rules)
    }
}
