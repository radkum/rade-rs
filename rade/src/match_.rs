use alloc::vec::Vec;

use crate::{Event, Guid};

#[derive(Debug, Default)]
pub struct Matches(Vec<(Event, MatchedRules)>);
impl Matches {
    pub fn add_match(&mut self, event: Event, matched_rules: MatchedRules) {
        if !matched_rules.0.is_empty() {
            self.0.push((event, matched_rules));
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Event, &MatchedRules)> {
        self.0.iter().map(|pair| (&pair.0, &pair.1))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug, Default)]
pub struct MatchedRules(Vec<Guid>);
impl MatchedRules {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, rule_id: Guid) {
        self.0.push(rule_id);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Guid> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl core::fmt::Display for Matches {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Matches {{ ")?;
        for (i, (event, matched_rules)) in self.0.iter().enumerate() {
            writeln!(
                f,
                "\t{i}: \"{}\", Matched: {:?}",
                event.name().unwrap_or("Unnamed event"),
                matched_rules.0
            )?;
        }
        write!(f, "}}")
    }
}
