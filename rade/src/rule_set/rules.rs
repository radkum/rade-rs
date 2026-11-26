use serde::{Deserialize, Serialize};

use super::Rule;
#[cfg(feature = "std")]
use crate::{RadeResult, utils::is_yaml_path};
use crate::prelude::*;

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Rules(Vec<Rule>);
impl Rules {
    pub fn iter(&self) -> core::slice::Iter<'_, Rule> {
        self.0.iter()
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.0.push(rule);
    }

    /// Look up a rule by its id.
    pub fn get(&self, id: &crate::Guid) -> Option<&Rule> {
        self.0.iter().find(|rule| rule.id() == id)
    }

    #[cfg(feature = "std")]
    pub fn from_dir(path: &std::path::Path) -> RadeResult<Self> {
        fn imp_from_dir(path: &std::path::Path, rules: &mut Rules) -> RadeResult<()> {
            if path.is_dir() {
                for entry in std::fs::read_dir(path)? {
                    let Ok(entry) = entry else {
                        log::warn!("Failed to read directory entry in {}", path.display());
                        continue;
                    };
                    let entry_path = entry.path();
                    if entry_path.is_dir() || is_yaml_path(&entry_path) {
                        imp_from_dir(&entry_path, rules)?;
                    }
                }
            } else if path.is_file() {
                let content = std::fs::read_to_string(path).map_err(|err| {
                    format!("Failed to read rule file {}: {err}", path.display())
                })?;
                let rule = Rule::from_yaml(&content).map_err(|err| {
                    format!("Failed to parse rule file {}: {err}", path.display())
                })?;
                rules.add_rule(rule);
            } else {
                return Err(format!(
                    "Path {} is neither file nor directory",
                    path.display()
                )
                .into());
            }
            Ok(())
        }
        let mut rules = Rules::default();
        imp_from_dir(path, &mut rules)?;
        Ok(rules)
    }
}

impl From<Vec<Rule>> for Rules {
    fn from(rules: Vec<Rule>) -> Self {
        Self(rules)
    }
}
