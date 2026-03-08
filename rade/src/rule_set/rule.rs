mod operand;
mod parser;
use core::str::FromStr;

//use operand::Operand;
pub use operand::*;
use parser::ConditionParser;
use serde::{Deserialize, Serialize};

use super::predicates::ResultMap;
#[cfg(feature = "std")]
use super::RuleResult;
use crate::prelude::*;
use crate::{Event, Guid};

type Condition = OperandContainer;

/// Rule struct - uses derived Deserialize for bincode (binary) format.
/// For YAML deserialization, use `Rule::from_yaml()` or `FromStr`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    id: Guid,
    name: Option<String>,
    description: Option<String>,
    #[serde(default)]
    categories: Option<Vec<String>>,
    mitre_tactic: Option<String>,
    mitre_tactic_id: Option<String>,
    mitre_id: Option<String>,
    example: Option<String>,
    condition: Condition,
}

/// Helper struct for YAML deserialization where condition is a string
#[derive(Deserialize)]
struct RuleYaml {
    id: Guid,
    name: Option<String>,
    description: Option<String>,
    #[serde(default)]
    categories: Option<Vec<String>>,
    mitre_tactic: Option<String>,
    mitre_tactic_id: Option<String>,
    mitre_id: Option<String>,
    example: Option<String>,
    condition: String,
}

impl Rule {
    /// Deserialize from YAML string (parses condition string using DSL parser)
    pub fn from_yaml(yaml: &str) -> crate::RadeResult<Self> {
        let raw: RuleYaml = serde_yaml_bw::from_str(yaml)?;
        let condition = ConditionParser::parse_condition(&raw.condition)?;

        Ok(Rule {
            id: raw.id,
            name: raw.name,
            description: raw.description,
            categories: raw.categories,
            mitre_tactic: raw.mitre_tactic,
            mitre_tactic_id: raw.mitre_tactic_id,
            mitre_id: raw.mitre_id,
            example: raw.example,
            condition,
        })
    }
}

impl FromStr for Rule {
    type Err = anyhow::Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Rule::from_yaml(src).map_err(|e| anyhow::anyhow!("{}", e))
    }
}

impl Rule {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        guid: Guid,
        name: &str,
        description: &str,
        mitre_tactic: &str,
        mitre_tactic_id: &str,
        mitre_id: &str,
        example: &str,
        condition: Condition,
    ) -> Self {
        Self {
            id: guid,
            name: Some(name.to_string()),
            description: Some(description.to_string()),
            categories: None,
            mitre_tactic: Some(mitre_tactic.to_string()),
            mitre_tactic_id: Some(mitre_tactic_id.to_string()),
            mitre_id: Some(mitre_id.to_string()),
            example: Some(example.to_string()),
            condition,
        }
    }

    #[cfg(feature = "std")]
    pub fn from_path(src: &std::path::Path) -> RuleResult<Self> {
        let content = std::fs::read_to_string(src)?;
        Ok(serde_yaml_bw::from_str::<Rule>(&content)?)
    }

    pub fn id(&self) -> &Guid {
        &self.id
    }

    pub fn evaluate(&self, event: &Event) -> bool {
        self.condition.evaluate(event, &mut ResultMap::new())
    }

    pub fn operands(&self) -> (Vec<OperandContainer>, Vec<OperandContainer>) {
        let mut simple_operands = Vec::new();
        let mut complex_operands = Vec::new();
        self.condition
            .operands(&mut simple_operands, &mut complex_operands);
        (simple_operands, complex_operands)
    }

    pub fn condition_hash(&self) -> OpHash {
        self.condition.hash()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_rule_from_yaml() {
        let yaml = r#"
id: 1ee8be03-0f8b-4808-9c3d-260ae456f051
name: Simple
description: The simplest rule
categories: ["test"]
mitre_tactic: Test
example: 'text = "old value"'
condition: |
  text_old.replace('old', 'new') == text_new
"#;
        let rule: Rule = serde_yaml_bw::from_str(yaml).unwrap();
        assert_eq!(rule.name, Some("Simple".to_string()));
        assert_eq!(rule.categories, Some(vec!["test".to_string()]));
        // The condition should be parsed into an OperandContainer
        println!("Parsed rule: {:?}", rule);
    }

    #[test]
    fn test_simple_rule_from_file() {
        let rule = Rule::from_path(std::path::Path::new(
            "test_data/rules/amsi_disable/Simple.yaml",
        ))
        .unwrap();
        assert_eq!(rule.name, Some("Simple".to_string()));
        println!("Parsed rule from file: {:?}", rule);
    }
}
