use pest::Parser;
use pest_derive::Parser;

use super::{Cast, Comparator, Condition, Operand, OperandContainer, Val};
use crate::{InsensitiveFlag, Result};

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
    pub fn parse_condition(input: &str) -> Result<Condition> {
        let mut pairs = Self::parse(Rule::program, input)?;
        let program_pair = pairs.next().unwrap();
        let expression_token = program_pair.into_inner().next().unwrap();
        Self::parse_expression(expression_token)
    }

    fn parse_expression(token: Pair) -> Result<OperandContainer> {
        check_rule!(token, Rule::expression);
        let inner = token.into_inner().next().unwrap();
        Self::parse_logical_or(inner)
    }

    fn parse_logical_or(token: Pair) -> Result<OperandContainer> {
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

    fn parse_logical_and(token: Pair) -> Result<OperandContainer> {
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

    fn parse_comparison(token: Pair) -> Result<OperandContainer> {
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
                Rule::AIEQ => {
                    Operand::Eq(left_op, right_op, Some(InsensitiveFlag::CaseAndApostrophe))
                },
                Rule::GE => Operand::Ncmp(left_op.as_num()?, right_op.as_num()?, Comparator::Ge),
                Rule::LE => Operand::Ncmp(left_op.as_num()?, right_op.as_num()?, Comparator::Le),
                Rule::GT => Operand::Ncmp(left_op.as_num()?, right_op.as_num()?, Comparator::Gt),
                Rule::LT => Operand::Ncmp(left_op.as_num()?, right_op.as_num()?, Comparator::Lt),
                _ => unexpected_token!(eq_op_token),
            };

            Ok(OperandContainer::from(op))
        } else {
            // implement bool predicate
            todo!()
        }
    }

    fn parse_unary(token: Pair) -> Result<Val> {
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

    fn parse_primary(token: Pair) -> Result<Val> {
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

    fn parse_literal(token: Pair) -> Result<Val> {
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
    use super::*;
    //use crate::rule_set::rule::parser::Rule;

    #[test]
    fn test_rule_parser() {
        let input = "b >= 10 && flag > 2 || c == 'test'";

        let parsed = ConditionParser::parse_condition(input).expect("Parse error");

        println!("{:#?}", parsed);
        assert!(false);
    }
}
