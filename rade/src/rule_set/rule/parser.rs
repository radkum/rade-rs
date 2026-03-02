use pest::Parser;
use pest_derive::Parser;

use super::{Cast, Comparator, Condition, Operand, OperandContainer, Val};
use crate::{InsensitiveFlag, RadeResult};

type PestError = pest::error::Error<Rule>;
type Pair<'i> = ::pest::iterators::Pair<'i, Rule>;
type Pairs<'i> = ::pest::iterators::Pairs<'i, Rule>;

macro_rules! unexpected_token {
    ($pair:expr) => {
        panic!("Unexpected token: {:?}", $pair.as_rule())
    };
}

macro_rules! check_rule {
    ($pair:expr, $rule:pat) => {
        if !matches!($pair.as_rule(), $rule) {
            panic!(
                "Unexpected token: {:?}, instead of {}",
                $pair.as_rule(),
                stringify!($rule)
            );
        }
    };
}

#[derive(Parser)]
#[grammar = "rule_set/rule/condition.pest"]
pub struct ConditionParser;

impl ConditionParser {
    pub fn parse_condition(input: &str) -> RadeResult<Condition> {
        let mut pairs = Self::parse(Rule::program, input)?;
        let program_pair = pairs.next().unwrap();
        let expression_token = program_pair.into_inner().next().unwrap();
        Self::parse_expression(expression_token)
    }

    fn parse_expression(token: Pair) -> RadeResult<OperandContainer> {
        check_rule!(token, Rule::expression);
        let inner = token.into_inner().next().unwrap();
        Self::parse_logical_or(inner)
    }

    fn parse_logical_or(token: Pair) -> RadeResult<OperandContainer> {
        check_rule!(token, Rule::logical_or);
        let pairs = token.into_inner();
        let mut logical_ors = Vec::new();
        for token in pairs {
            check_rule!(token, Rule::logical_and);
            logical_ors.push(Self::parse_logical_and(token)?);
        }
        if logical_ors.len() == 1 {
            Ok(logical_ors.into_iter().next().unwrap())
        } else {
            Ok(OperandContainer::from(Operand::Or(logical_ors)))
        }
    }

    fn parse_logical_and(token: Pair) -> RadeResult<OperandContainer> {
        check_rule!(token, Rule::logical_and);
        let pairs = token.into_inner();
        let mut logical_ands = Vec::new();
        for token in pairs {
            check_rule!(token, Rule::comparison);
            logical_ands.push(Self::parse_comparison(token)?);
        }
        if logical_ands.len() == 1 {
            Ok(logical_ands.into_iter().next().unwrap())
        } else {
            Ok(OperandContainer::from(Operand::And(logical_ands)))
        }
    }

    fn parse_comparison(token: Pair) -> RadeResult<OperandContainer> {
        check_rule!(token, Rule::comparison);
        let mut pairs = token.into_inner();
        let left_token = pairs.next().unwrap();
        let left_op = Self::parse_unary(left_token)?;
        if let Some(eq_op_token) = pairs.next() {
            let right_token = pairs.next().unwrap();
            let right_op = Self::parse_unary(right_token)?;

            let op = match eq_op_token.as_rule() {
                Rule::EQ => Operand::Eq(left_op, right_op, None),
                Rule::NEQ => Operand::Neq(left_op, right_op, None),
                Rule::IEQ => Operand::Eq(left_op, right_op, Some(InsensitiveFlag::Case)),
                Rule::AEQ => Operand::Eq(left_op, right_op, Some(InsensitiveFlag::Apostrophe)),
                Rule::AIEQ => Operand::Eq(left_op, right_op, Some(InsensitiveFlag::CaseAndApostrophe)),
                Rule::GE => Operand::Cmp(left_op.into_num()?, right_op.into_num()?, Comparator::Ge),
                Rule::LE => Operand::Cmp(left_op.into_num()?, right_op.into_num()?, Comparator::Le),
                Rule::GT => Operand::Cmp(left_op.into_num()?, right_op.into_num()?, Comparator::Gt),
                Rule::LT => Operand::Cmp(left_op.into_num()?, right_op.into_num()?, Comparator::Lt),
                _ => unexpected_token!(eq_op_token),
            };

            Ok(OperandContainer::from(op))
        } else {
            Ok(OperandContainer::from(Operand::Bool(left_op.validate_bool()?)))
        }
    }

    fn parse_unary(token: Pair) -> RadeResult<Val> {
        check_rule!(token, Rule::unary);
        let mut pairs = token.into_inner();
        let mut token = pairs.next().unwrap();
        let negate = if let Rule::NOT_OP = token.as_rule() {
            token = pairs.next().unwrap();
            true
        } else {
            false
        };

        check_rule!(token, Rule::primary);
        let primary = Self::parse_primary(token)?;

        for postfix in pairs {
            match postfix.as_rule() {
                Rule::function_call => {
                    todo!()
                },
                Rule::element_access => {
                    todo!()
                },
                _ => unexpected_token!(postfix),
            }
        }
        Ok(primary)
    }

    fn parse_primary(token: Pair) -> RadeResult<Val> {
        check_rule!(token, Rule::primary);
        let mut inner = token.into_inner();
        let primary_token = inner.next().unwrap();
        Ok(match primary_token.as_rule() {
            Rule::literal => Self::parse_literal(primary_token)?,
            Rule::identifier => Val::Field(primary_token.as_str().into()),
            Rule::parenthesis_expression => {
                todo!()
            },
            _ => unexpected_token!(primary_token),
        })
    }

    fn parse_literal(token: Pair) -> RadeResult<Val> {
        check_rule!(token, Rule::literal);
        let inner = token.into_inner().next().unwrap();
        Ok(match inner.as_rule() {
            Rule::string => Val::Str(inner.as_str().into()),
            Rule::number => Val::Int(inner.as_str().parse::<u64>()?.into()),
            Rule::cidr => todo!(),
            Rule::regex => todo!(),
            _ => unexpected_token!(inner),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::Event;
    use std::collections::HashMap;
    use super::*;
    use crate::EventSerialized;

    const CONDITION: &str = "b >= 10 && flag || c == 'test'";
    #[test]
    fn rule_match() {
        let condition = ConditionParser::parse_condition(CONDITION).expect("Parse error");
        let map = HashMap::from([
            (
                "b".to_string(),
                1234.into(),
            ),
            (
                "flag".to_string(),
                true.into(),
            ),
            (
                "c".to_string(),
                "test".into(),
            ),
        ]);
        let event = Event::from(EventSerialized::new(map));
        let result = condition.evaluate(&event);
        println!(" Result: {}", result);
        assert!(result);
    }

    #[test]
    fn rule_match_even_string_is_incorrect() {
        let condition = ConditionParser::parse_condition(CONDITION).expect("Parse error");
        let map = HashMap::from([
            (
                "b".to_string(),
                1234.into(),
            ),
            (
                "flag".to_string(),
                true.into(),
            ),
            (
                "c".to_string(),
                "none".into(),
            ),
        ]);
        let event = Event::from(EventSerialized::new(map));
        let result = condition.evaluate(&event);
        println!(" Result: {}", result);
        assert!(result);
    }

    #[test]
    fn rule_does_not_match() {
        let condition = ConditionParser::parse_condition(CONDITION).expect("Parse error");
        let map = HashMap::from([
            (
                "b".to_string(),
                2.into(),
            ),
            (
                "flag".to_string(),
                true.into(),
            ),
            (
                "c".to_string(),
                "none".into(),
            ),
        ]);
        let event = Event::from(EventSerialized::new(map));
        let result = condition.evaluate(&event);
        println!(" Result: {}", result);
        assert!(!result);
    }

    #[test]
    fn rule_with_float() {
        let condition = ConditionParser::parse_condition(CONDITION).expect("Parse error");
        let map = HashMap::from([
            (
                "b".to_string(),
                12.5.into(),
            ),
            (
                "flag".to_string(),
                true.into(),
            ),
            (
                "c".to_string(),
                "test".into(),
            ),
        ]);
        let event = Event::from(EventSerialized::new(map));
        let result = condition.evaluate(&event);
        println!(" Result: {}", result);
        assert!(result);
    }

    #[test]
    fn rule_incorrect_types_but_still_valid() {
        let condition = ConditionParser::parse_condition(CONDITION).expect("Parse error");
        let map = HashMap::from([
            (
                "b".to_string(),
                12.5.into(),
            ),
            (
                "flag".to_string(),
                true.into(),
            ),
            (
                "c".to_string(),
                1.into(),
            ),
        ]);
        let event = Event::from(EventSerialized::new(map));
        let result = condition.evaluate(&event);
        println!(" Result: {}", result);
        assert!(result);
    }

    #[test]
    fn rule_incorrect_types_but_still_invalid() {
        let condition = ConditionParser::parse_condition(CONDITION).expect("Parse error");
        let map = HashMap::from([
            (
                "b".to_string(),
                "a".into(),
            ),
            (
                "flag".to_string(),
                true.into(),
            ),
            (
                "c".to_string(),
                "test".into(),
            ),
        ]);
        let event = Event::from(EventSerialized::new(map));
        let result = condition.evaluate(&event);
        println!(" Result: {}", result);
        assert!(!result);
    }
}
