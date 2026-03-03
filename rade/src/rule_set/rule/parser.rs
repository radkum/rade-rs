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

            let (comparator, flag) = match eq_op_token.as_rule() {
                Rule::EQ => (Comparator::Eq, None),
                Rule::IEQ => (Comparator::Eq, Some(InsensitiveFlag::Case)),
                Rule::AEQ => (Comparator::Eq, Some(InsensitiveFlag::Apostrophe)),
                Rule::AIEQ => (Comparator::Eq, Some(InsensitiveFlag::CaseAndApostrophe)),
                Rule::NEQ => (Comparator::Neq, None),
                Rule::GE => (Comparator::Ge, None),
                Rule::LE => (Comparator::Le, None),
                Rule::GT => (Comparator::Gt, None),
                Rule::LT => (Comparator::Lt, None),
                Rule::MATCH => (Comparator::Match, None),
                Rule::NMATCH => (Comparator::Nmatch, None),
                _ => unexpected_token!(eq_op_token),
            };

            // Handle regex matching: if right operand is a regex, create Match operand
            let op = if let Val::Regex(regex) = right_op {
                if comparator == Comparator::Match || comparator == Comparator::Nmatch {
                    Operand::Match(left_op, regex, comparator)
                } else {
                    return Err("Regex can only be used with == operator".into());
                }
            } else {
                Operand::Cmp(left_op, right_op, comparator, flag)
            };

            Ok(OperandContainer::from(op))
        } else {
            Ok(OperandContainer::from(Operand::Val(
                left_op.validate_bool()?,
            )))
        }
    }

    fn parse_unary(token: Pair) -> RadeResult<Val> {
        check_rule!(token, Rule::unary);
        let mut pairs = token.into_inner();
        let token = pairs.next().unwrap();
        let mut primary = Self::parse_primary(token)?;

        for postfix in pairs {
            match postfix.as_rule() {
                Rule::function_call => {
                    todo!()
                },
                Rule::element_access => {
                    primary = Self::parse_element_access(primary, postfix)?;
                },
                _ => unexpected_token!(postfix),
            }
        }
        Ok(primary)
    }

    fn parse_element_access(primary: Val, token: Pair) -> RadeResult<Val> {
        check_rule!(token, Rule::element_access);
        let field = match primary {
            Val::Field(f) => f,
            _ => {
                return Err(format!(
                    "Element access is only supported on fields, got {:?}",
                    primary
                )
                .into());
            },
        };
        let mut pairs = token.into_inner();
        let index_expr = pairs.next().unwrap();
        // Parse the index - for now we only support integer literals
        let index = Self::parse_integer(index_expr)?;
        Ok(Val::FieldIndex(field, index))
    }

    fn parse_primary(token: Pair) -> RadeResult<Val> {
        check_rule!(token, Rule::primary);
        let mut pairs = token.into_inner();
        let primary_token = pairs.next().unwrap();
        Ok(match primary_token.as_rule() {
            Rule::literal => Self::parse_literal(primary_token)?,
            Rule::identifier => Val::Field(primary_token.as_str().into()),
            Rule::parenthesis_expression => Self::parse_parenthesis(primary_token)?,
            _ => unexpected_token!(primary_token),
        })
    }

    fn parse_parenthesis(token: Pair) -> RadeResult<Val> {
        check_rule!(token, Rule::parenthesis_expression);
        let mut pairs = token.into_inner();
        let mut token = pairs.next().unwrap();
        let negate = if let Rule::NOT_OP = token.as_rule() {
            token = pairs.next().unwrap();
            true
        } else {
            false
        };
        // if negate {
        //     Ok(OperandContainer::from(Operand::Negate(Box::new(Self::parse_expression(token)?)))
        // } else {
        //     Self::parse_expression(token)
        // }
        todo!()
    }

    fn parse_literal(token: Pair) -> RadeResult<Val> {
        check_rule!(token, Rule::literal);
        let inner = token.into_inner().next().unwrap();
        Ok(match inner.as_rule() {
            Rule::bool => Val::Bool(inner.as_str().parse::<bool>()?.into()),
            Rule::string => Val::Str(inner.as_str().replace("\\", "").into()),
            Rule::integer => Val::Int(Self::parse_integer(inner)?.into()),
            Rule::float => Val::Float(inner.as_str().parse::<f64>()?.into()),
            Rule::cidr => todo!(),
            Rule::regex => Self::parse_regex(inner)?,
            _ => unexpected_token!(inner),
        })
    }

    fn parse_regex(token: Pair) -> RadeResult<Val> {
        check_rule!(token, Rule::regex);
        let regex_str = token.as_str();
        // Remove the leading and trailing slashes and any flags
        // Format: /pattern/flags
        let regex_str = regex_str.trim_start_matches('/');
        let (pattern, flags) = if let Some(last_slash) = regex_str.rfind('/') {
            let (pattern, flags) = regex_str.split_at(last_slash);
            (pattern, &flags[1..]) // Skip the slash in flags part
        } else {
            (regex_str, "")
        };
        // Unescape \/ to /
        let pattern = pattern.replace("\\/", "/");
        Ok(Val::Regex(super::RadeRegex::new(&pattern, flags)?))
    }

    fn parse_integer(token: Pair) -> RadeResult<i64> {
        check_rule!(token, Rule::integer);
        let inner = token.into_inner().next().unwrap();
        let str = inner.as_str().replace("_", "");
        // 1️⃣ Handle sign
        let (negative, str_without_sign) = if let Some(rest) = str.strip_prefix('-') {
            (true, rest)
        } else if let Some(rest) = str.strip_prefix('+') {
            (false, rest)
        } else {
            (false, str.as_str())
        };
        let value = match inner.as_rule() {
            Rule::hex => i64::from_str_radix(str_without_sign.trim_start_matches("0x"), 16)?,
            Rule::octal => i64::from_str_radix(str_without_sign.trim_start_matches("0o"), 8)?,
            Rule::binary => i64::from_str_radix(str_without_sign.trim_start_matches("0b"), 2)?,
            Rule::decimal => i64::from_str_radix(str_without_sign.trim_start_matches("0d"), 10)?,
            _ => unexpected_token!(inner),
        };

        Ok(if negative { -value } else { value })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{super::ResultMap, *};
    use crate::{Event, EventSerialized};

    const CONDITION: &str = "b >= 10 && flag || c == 'test'";
    #[test]
    fn rule_match() {
        let condition = ConditionParser::parse_condition(CONDITION).expect("Parse error");
        let map = HashMap::from([
            ("b".to_string(), 1234.into()),
            ("flag".to_string(), true.into()),
            ("c".to_string(), "test".into()),
        ]);
        let event = Event::from(EventSerialized::new(map));
        let result = condition.evaluate(&event, &mut ResultMap::new());
        println!(" Result: {}", result);
        assert!(result);
    }

    #[test]
    fn rule_match_even_string_is_incorrect() {
        let condition = ConditionParser::parse_condition(CONDITION).expect("Parse error");
        let map = HashMap::from([
            ("b".to_string(), 1234.into()),
            ("flag".to_string(), true.into()),
            ("c".to_string(), "none".into()),
        ]);
        let event = Event::from(EventSerialized::new(map));
        let result = condition.evaluate(&event, &mut ResultMap::new());
        println!(" Result: {}", result);
        assert!(result);
    }

    #[test]
    fn rule_does_not_match() {
        let condition = ConditionParser::parse_condition(CONDITION).expect("Parse error");
        let map = HashMap::from([
            ("b".to_string(), 2.into()),
            ("flag".to_string(), true.into()),
            ("c".to_string(), "none".into()),
        ]);
        let event = Event::from(EventSerialized::new(map));
        let result = condition.evaluate(&event, &mut ResultMap::new());
        println!(" Result: {}", result);
        assert!(!result);
    }

    #[test]
    fn rule_with_float() {
        let condition = ConditionParser::parse_condition(CONDITION).expect("Parse error");
        let map = HashMap::from([
            ("b".to_string(), 12.5.into()),
            ("flag".to_string(), true.into()),
            ("c".to_string(), "test".into()),
        ]);
        let event = Event::from(EventSerialized::new(map));
        let result = condition.evaluate(&event, &mut ResultMap::new());
        println!(" Result: {}", result);
        assert!(result);
    }

    #[test]
    fn rule_incorrect_types_but_still_valid() {
        let condition = ConditionParser::parse_condition(CONDITION).expect("Parse error");
        let map = HashMap::from([
            ("b".to_string(), 12.5.into()),
            ("flag".to_string(), true.into()),
            ("c".to_string(), 1.into()),
        ]);
        let event = Event::from(EventSerialized::new(map));
        let result = condition.evaluate(&event, &mut ResultMap::new());
        println!(" Result: {}", result);
        assert!(result);
    }

    #[test]
    fn rule_incorrect_types() {
        let condition = ConditionParser::parse_condition(CONDITION).expect("Parse error");
        let map = HashMap::from([
            ("b".to_string(), "a".into()),
            ("flag".to_string(), true.into()),
            ("c".to_string(), "test".into()),
        ]);
        let event = Event::from(EventSerialized::new(map));
        let result = condition.evaluate(&event, &mut ResultMap::new());
        println!(" Result: {}", result);
        assert!(result);
    }

    // ============================================
    // Simple Equality Tests
    // ============================================

    #[test]
    fn test_simple_eq_string_match() {
        let condition = ConditionParser::parse_condition("name == 'alice'").unwrap();
        let map = HashMap::from([("name".to_string(), "alice".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_simple_eq_string_no_match() {
        let condition = ConditionParser::parse_condition("name == 'alice'").unwrap();
        let map = HashMap::from([("name".to_string(), "bob".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(!condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_simple_neq_string() {
        let condition = ConditionParser::parse_condition("name != 'alice'").unwrap();
        let map = HashMap::from([("name".to_string(), "bob".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_simple_eq_number() {
        let condition = ConditionParser::parse_condition("count == 42").unwrap();
        let map = HashMap::from([("count".to_string(), 42.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    // ============================================
    // Case-Insensitive Comparison Tests
    // ============================================

    #[test]
    fn test_case_insensitive_eq() {
        let condition = ConditionParser::parse_condition("name ~= 'ALICE'").unwrap();
        let map = HashMap::from([("name".to_string(), "alice".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_case_insensitive_eq_mixed_case() {
        let condition = ConditionParser::parse_condition("name ~= 'AlIcE'").unwrap();
        let map = HashMap::from([("name".to_string(), "aLiCe".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_apostrophe_insensitive_eq() {
        let condition = ConditionParser::parse_condition("name ^= 'don\"t'").unwrap();
        let map = HashMap::from([("name".to_string(), "don't".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_case_and_apostrophe_insensitive_eq() {
        let condition = ConditionParser::parse_condition("name ^~= 'DON\"T'").unwrap();
        let map = HashMap::from([("name".to_string(), "don't".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    // ============================================
    // Numeric Comparison Tests
    // ============================================

    #[test]
    fn test_greater_than() {
        let condition = ConditionParser::parse_condition("value > 10").unwrap();
        let map = HashMap::from([("value".to_string(), 15.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_greater_than_equal_boundary() {
        let condition = ConditionParser::parse_condition("value > 10").unwrap();
        let map = HashMap::from([("value".to_string(), 10.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(!condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_less_than() {
        let condition = ConditionParser::parse_condition("value < 10").unwrap();
        let map = HashMap::from([("value".to_string(), 5.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_greater_than_or_equal() {
        let condition = ConditionParser::parse_condition("value >= 10").unwrap();
        let map = HashMap::from([("value".to_string(), 10.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_less_than_or_equal() {
        let condition = ConditionParser::parse_condition("value <= 10").unwrap();
        let map = HashMap::from([("value".to_string(), 10.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    // ============================================
    // Different Number Format Tests
    // ============================================

    #[test]
    fn test_hex_number() {
        let condition = ConditionParser::parse_condition("value == 0xFF").unwrap();
        let map = HashMap::from([("value".to_string(), 255.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_binary_number() {
        let condition = ConditionParser::parse_condition("value == 0b1010").unwrap();
        let map = HashMap::from([("value".to_string(), 10.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_octal_number() {
        let condition = ConditionParser::parse_condition("value == 0o77").unwrap();
        let map = HashMap::from([("value".to_string(), 63.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_float_comparison() {
        let condition = ConditionParser::parse_condition("value > 3.14").unwrap();
        let map = HashMap::from([("value".to_string(), 3.5.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_negative_number() {
        let condition = ConditionParser::parse_condition("value > -5").unwrap();
        let map = HashMap::from([("value".to_string(), 0.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    // ============================================
    // Boolean Field Tests
    // ============================================

    #[test]
    fn test_boolean_field_true() {
        let condition = ConditionParser::parse_condition("enabled").unwrap();
        let map = HashMap::from([("enabled".to_string(), true.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_boolean_field_false() {
        let condition = ConditionParser::parse_condition("enabled").unwrap();
        let map = HashMap::from([("enabled".to_string(), false.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(!condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_negated_boolean_field() {
        let condition = ConditionParser::parse_condition("disabled == false").unwrap();
        let map = HashMap::from([("disabled".to_string(), false.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    // ============================================
    // Logical AND Tests
    // ============================================

    #[test]
    fn test_simple_and_both_true() {
        let condition = ConditionParser::parse_condition("a == 1 && b == 2").unwrap();
        let map = HashMap::from([("a".to_string(), 1.into()), ("b".to_string(), 2.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_simple_and_one_false() {
        let condition = ConditionParser::parse_condition("a == 1 && b == 2").unwrap();
        let map = HashMap::from([("a".to_string(), 1.into()), ("b".to_string(), 3.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(!condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_multiple_and() {
        let condition = ConditionParser::parse_condition("a == 1 && b == 2 && c == 3").unwrap();
        let map = HashMap::from([
            ("a".to_string(), 1.into()),
            ("b".to_string(), 2.into()),
            ("c".to_string(), 3.into()),
        ]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    // ============================================
    // Logical OR Tests
    // ============================================

    #[test]
    fn test_simple_or_first_true() {
        let condition = ConditionParser::parse_condition("a == 1 || b == 2").unwrap();
        let map = HashMap::from([("a".to_string(), 1.into()), ("b".to_string(), 99.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_simple_or_second_true() {
        let condition = ConditionParser::parse_condition("a == 1 || b == 2").unwrap();
        let map = HashMap::from([("a".to_string(), 99.into()), ("b".to_string(), 2.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_simple_or_both_false() {
        let condition = ConditionParser::parse_condition("a == 1 || b == 2").unwrap();
        let map = HashMap::from([("a".to_string(), 99.into()), ("b".to_string(), 99.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(!condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_multiple_or() {
        let condition = ConditionParser::parse_condition("a == 1 || b == 2 || c == 3").unwrap();
        let map = HashMap::from([
            ("a".to_string(), 99.into()),
            ("b".to_string(), 99.into()),
            ("c".to_string(), 3.into()),
        ]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    // ============================================
    // Complex AND/OR Combinations
    // ============================================

    #[test]
    fn test_and_or_precedence() {
        // AND has higher precedence than OR: a || b && c means a || (b && c)
        let condition = ConditionParser::parse_condition("a == 1 || b == 2 && c == 3").unwrap();
        let map = HashMap::from([
            ("a".to_string(), 99.into()),
            ("b".to_string(), 2.into()),
            ("c".to_string(), 3.into()),
        ]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_and_or_precedence_false() {
        // a || (b && c) where a=false, b=true, c=false => false || false => false
        let condition = ConditionParser::parse_condition("a == 1 || b == 2 && c == 3").unwrap();
        let map = HashMap::from([
            ("a".to_string(), 99.into()),
            ("b".to_string(), 2.into()),
            ("c".to_string(), 99.into()),
        ]);
        let event = Event::from(EventSerialized::new(map));
        assert!(!condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_complex_nested_logic() {
        let condition =
            ConditionParser::parse_condition("status == 'active' && level >= 5 || admin").unwrap();
        let map = HashMap::from([
            ("status".to_string(), "active".into()),
            ("level".to_string(), 10.into()),
            ("admin".to_string(), false.into()),
        ]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    // ============================================
    // String with Special Characters Tests
    // ============================================

    #[test]
    fn test_string_with_escaped_quote() {
        let condition = ConditionParser::parse_condition(r#"msg == 'it\'s'"#).unwrap();
        let map = HashMap::from([("msg".to_string(), "it's".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_empty_string() {
        let condition = ConditionParser::parse_condition("name == ''").unwrap();
        let map = HashMap::from([("name".to_string(), "".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_string_with_spaces() {
        let condition = ConditionParser::parse_condition("msg == 'hello world'").unwrap();
        let map = HashMap::from([("msg".to_string(), "hello world".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    // ============================================
    // Missing Field Tests
    // ============================================

    #[test]
    fn test_missing_field_in_comparison() {
        let condition = ConditionParser::parse_condition("missing == 'value'").unwrap();
        let map = HashMap::from([("other".to_string(), "value".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(!condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_missing_field_as_boolean() {
        let condition = ConditionParser::parse_condition("missing").unwrap();
        let map = HashMap::from([("other".to_string(), true.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(!condition.evaluate(&event, &mut ResultMap::new()));
    }

    // ============================================
    // Identifier Edge Cases
    // ============================================

    #[test]
    fn test_identifier_with_underscore() {
        let condition = ConditionParser::parse_condition("user_name == 'test'").unwrap();
        let map = HashMap::from([("user_name".to_string(), "test".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_identifier_with_numbers() {
        let condition = ConditionParser::parse_condition("field123 == 'value'").unwrap();
        let map = HashMap::from([("field123".to_string(), "value".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_identifier_starting_with_underscore() {
        let condition = ConditionParser::parse_condition("_private == 42").unwrap();
        let map = HashMap::from([("_private".to_string(), 42.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    // ============================================
    // Whitespace Handling Tests
    // ============================================

    #[test]
    fn test_extra_whitespace() {
        let condition =
            ConditionParser::parse_condition("  a   ==   1   &&   b   ==   2  ").unwrap();
        let map = HashMap::from([("a".to_string(), 1.into()), ("b".to_string(), 2.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_newlines_in_condition() {
        let condition = ConditionParser::parse_condition("a == 1\n&&\nb == 2").unwrap();
        let map = HashMap::from([("a".to_string(), 1.into()), ("b".to_string(), 2.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    // ============================================
    // Parse Error Tests
    // ============================================

    #[test]
    fn test_invalid_syntax_missing_operand() {
        let result = ConditionParser::parse_condition("a ==");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_syntax_double_operator() {
        let result = ConditionParser::parse_condition("a == == b");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_syntax_unclosed_string() {
        let result = ConditionParser::parse_condition("name == 'unclosed");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_condition() {
        let result = ConditionParser::parse_condition("");
        assert!(result.is_err());
    }

    // ============================================
    // Field-to-Field Comparison Tests
    // ============================================

    #[test]
    fn test_field_to_field_eq() {
        let condition = ConditionParser::parse_condition("field_a == field_b").unwrap();
        let map = HashMap::from([
            ("field_a".to_string(), "same".into()),
            ("field_b".to_string(), "same".into()),
        ]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_field_to_field_neq() {
        let condition = ConditionParser::parse_condition("field_a != field_b").unwrap();
        let map = HashMap::from([
            ("field_a".to_string(), "value1".into()),
            ("field_b".to_string(), "value2".into()),
        ]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_field_to_field_numeric_comparison() {
        let condition = ConditionParser::parse_condition("min_val < max_val").unwrap();
        let map = HashMap::from([
            ("min_val".to_string(), 10.into()),
            ("max_val".to_string(), 100.into()),
        ]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    // ============================================
    // Large Number Tests
    // ============================================

    #[test]
    fn test_large_number() {
        let condition = ConditionParser::parse_condition("big_num > 1000000000").unwrap();
        let map = HashMap::from([("big_num".to_string(), 9999999999u64.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_number_with_underscores() {
        let condition = ConditionParser::parse_condition("value == 1_000_000").unwrap();
        let map = HashMap::from([("value".to_string(), 1000000.into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    // ============================================
    // FieldIndex Tests (element access with [index])
    // ============================================

    #[test]
    fn test_field_index_string_list_first_element() {
        use serde_yaml_bw::Value as YamlValue;
        let condition = ConditionParser::parse_condition("items[0] == 'apple'").unwrap();
        let items = YamlValue::Sequence(vec![
            YamlValue::String("apple".to_string()),
            YamlValue::String("banana".to_string()),
            YamlValue::String("cherry".to_string()),
        ]);
        let map = HashMap::from([("items".to_string(), items)]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_field_index_string_list_middle_element() {
        use serde_yaml_bw::Value as YamlValue;
        let condition = ConditionParser::parse_condition("items[1] == 'banana'").unwrap();
        let items = YamlValue::Sequence(vec![
            YamlValue::String("apple".to_string()),
            YamlValue::String("banana".to_string()),
            YamlValue::String("cherry".to_string()),
        ]);
        let map = HashMap::from([("items".to_string(), items)]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_field_index_string_list_last_element() {
        use serde_yaml_bw::Value as YamlValue;
        let condition = ConditionParser::parse_condition("items[2] == 'cherry'").unwrap();
        let items = YamlValue::Sequence(vec![
            YamlValue::String("apple".to_string()),
            YamlValue::String("banana".to_string()),
            YamlValue::String("cherry".to_string()),
        ]);
        let map = HashMap::from([("items".to_string(), items)]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_field_index_string_list_no_match() {
        use serde_yaml_bw::Value as YamlValue;
        let condition = ConditionParser::parse_condition("items[0] == 'orange'").unwrap();
        let items = YamlValue::Sequence(vec![
            YamlValue::String("apple".to_string()),
            YamlValue::String("banana".to_string()),
        ]);
        let map = HashMap::from([("items".to_string(), items)]);
        let event = Event::from(EventSerialized::new(map));
        assert!(!condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_field_index_int_list_first_element() {
        use serde_yaml_bw::Value as YamlValue;
        let condition = ConditionParser::parse_condition("numbers[0] == 10").unwrap();
        let numbers = YamlValue::Sequence(vec![
            YamlValue::Number(10.into()),
            YamlValue::Number(20.into()),
            YamlValue::Number(30.into()),
        ]);
        let map = HashMap::from([("numbers".to_string(), numbers)]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_field_index_int_list_comparison() {
        use serde_yaml_bw::Value as YamlValue;
        let condition = ConditionParser::parse_condition("numbers[1] > 15").unwrap();
        let numbers = YamlValue::Sequence(vec![
            YamlValue::Number(10.into()),
            YamlValue::Number(20.into()),
            YamlValue::Number(30.into()),
        ]);
        let map = HashMap::from([("numbers".to_string(), numbers)]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_field_index_in_complex_condition() {
        use serde_yaml_bw::Value as YamlValue;
        let condition =
            ConditionParser::parse_condition("items[0] == 'start' && values[1] > 50").unwrap();
        let items = YamlValue::Sequence(vec![
            YamlValue::String("start".to_string()),
            YamlValue::String("middle".to_string()),
        ]);
        let values = YamlValue::Sequence(vec![
            YamlValue::Number(25.into()),
            YamlValue::Number(75.into()),
        ]);
        let map = HashMap::from([("items".to_string(), items), ("values".to_string(), values)]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_field_index_with_or_condition() {
        use serde_yaml_bw::Value as YamlValue;
        let condition =
            ConditionParser::parse_condition("items[0] == 'wrong' || items[1] == 'banana'")
                .unwrap();
        let items = YamlValue::Sequence(vec![
            YamlValue::String("apple".to_string()),
            YamlValue::String("banana".to_string()),
        ]);
        let map = HashMap::from([("items".to_string(), items)]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_field_index_out_of_bounds() {
        use serde_yaml_bw::Value as YamlValue;
        let condition = ConditionParser::parse_condition("items[99] == 'missing'").unwrap();
        let items = YamlValue::Sequence(vec![YamlValue::String("apple".to_string())]);
        let map = HashMap::from([("items".to_string(), items)]);
        let event = Event::from(EventSerialized::new(map));
        // Out of bounds should evaluate to false (error case)
        assert!(!condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_field_index_neq() {
        use serde_yaml_bw::Value as YamlValue;
        let condition = ConditionParser::parse_condition("items[0] != 'orange'").unwrap();
        let items = YamlValue::Sequence(vec![YamlValue::String("apple".to_string())]);
        let map = HashMap::from([("items".to_string(), items)]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_field_index_parse_only() {
        // Just test that parsing works correctly
        let condition = ConditionParser::parse_condition("arr[0] == 'test'");
        assert!(condition.is_ok());
    }

    #[test]
    fn test_field_index_parse_with_larger_index() {
        let condition = ConditionParser::parse_condition("arr[123] == 'test'");
        assert!(condition.is_ok());
    }

    // ============================================
    // Regex Tests
    // ============================================

    #[test]
    fn test_regex_simple_match() {
        let condition = ConditionParser::parse_condition("name =~ /alice/").unwrap();
        let map = HashMap::from([("name".to_string(), "alice".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_partial_match() {
        let condition = ConditionParser::parse_condition("name =~ /ali/").unwrap();
        let map = HashMap::from([("name".to_string(), "alice".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_no_match() {
        let condition = ConditionParser::parse_condition("name =~ /bob/").unwrap();
        let map = HashMap::from([("name".to_string(), "alice".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(!condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_with_special_chars() {
        let condition = ConditionParser::parse_condition(r"email =~ /\w+@\w+\.\w+/").unwrap();
        let map = HashMap::from([("email".to_string(), "test@example.com".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_start_anchor() {
        let condition = ConditionParser::parse_condition("name =~ /^alice/").unwrap();
        let map = HashMap::from([("name".to_string(), "alice123".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_start_anchor_no_match() {
        let condition = ConditionParser::parse_condition("name =~ /^alice/").unwrap();
        let map = HashMap::from([("name".to_string(), "xxx_alice".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(!condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_end_anchor() {
        let condition = ConditionParser::parse_condition("name =~ /alice$/").unwrap();
        let map = HashMap::from([("name".to_string(), "xxx_alice".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_full_match_anchors() {
        let condition = ConditionParser::parse_condition("name =~ /^alice$/").unwrap();
        let map = HashMap::from([("name".to_string(), "alice".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_full_match_anchors_no_match() {
        let condition = ConditionParser::parse_condition("name =~ /^alice$/").unwrap();
        let map = HashMap::from([("name".to_string(), "alice123".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(!condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_digit_pattern() {
        let condition = ConditionParser::parse_condition(r"code =~ /\d{3}-\d{4}/").unwrap();
        let map = HashMap::from([("code".to_string(), "123-4567".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_alternation() {
        let condition = ConditionParser::parse_condition("fruit =~ /apple|banana|cherry/").unwrap();
        let map = HashMap::from([("fruit".to_string(), "banana".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_character_class() {
        let condition = ConditionParser::parse_condition("grade =~ /[A-F]/").unwrap();
        let map = HashMap::from([("grade".to_string(), "B".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_negated_character_class() {
        let condition = ConditionParser::parse_condition("value =~ /[^0-9]/").unwrap();
        let map = HashMap::from([("value".to_string(), "abc".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_quantifiers() {
        let condition = ConditionParser::parse_condition("text =~ /a+b*c?/").unwrap();
        let map = HashMap::from([("text".to_string(), "aaabbc".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_in_and_condition() {
        let condition =
            ConditionParser::parse_condition("name =~ /^test/ && status == 'active'").unwrap();
        let map = HashMap::from([
            ("name".to_string(), "test_user".into()),
            ("status".to_string(), "active".into()),
        ]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_in_or_condition() {
        let condition =
            ConditionParser::parse_condition("name =~ /^admin/ || name =~ /^test/").unwrap();
        let map = HashMap::from([("name".to_string(), "test_user".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_parse_only() {
        let condition = ConditionParser::parse_condition("field =~ /pattern/");
        assert!(condition.is_ok());
    }

    #[test]
    fn test_regex_parse_complex_pattern() {
        let condition = ConditionParser::parse_condition(
            r"field =~ /^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$/",
        );
        assert!(condition.is_ok());
    }

    #[test]
    fn test_regex_word_boundary() {
        let condition = ConditionParser::parse_condition(r"text =~ /\bword\b/").unwrap();
        let map = HashMap::from([("text".to_string(), "a word here".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_word_boundary_no_match() {
        let condition = ConditionParser::parse_condition(r"text =~ /\bword\b/").unwrap();
        let map = HashMap::from([("text".to_string(), "awordhere".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(!condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_optional_group() {
        let condition = ConditionParser::parse_condition("text =~ /colou?r/").unwrap();
        let map = HashMap::from([("text".to_string(), "color".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_optional_group_british() {
        let condition = ConditionParser::parse_condition("text =~ /colou?r/").unwrap();
        let map = HashMap::from([("text".to_string(), "colour".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    // ============================================
    // Regex Flags Tests
    // ============================================

    #[test]
    fn test_regex_flag_i_case_insensitive() {
        let condition = ConditionParser::parse_condition("name =~ /alice/i").unwrap();
        let map = HashMap::from([("name".to_string(), "ALICE".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_flag_i_mixed_case() {
        let condition = ConditionParser::parse_condition("name =~ /hello world/i").unwrap();
        let map = HashMap::from([("name".to_string(), "HeLLo WoRLd".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_flag_i_no_match_without_flag() {
        let condition = ConditionParser::parse_condition("name =~ /alice/").unwrap();
        let map = HashMap::from([("name".to_string(), "ALICE".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(!condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_flag_m_multiline() {
        // With 'm' flag, ^ and $ match at line boundaries
        let condition = ConditionParser::parse_condition("text =~ /^line2/m").unwrap();
        let map = HashMap::from([("text".to_string(), "line1\nline2\nline3".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_flag_m_no_match_without_flag() {
        // Without 'm' flag, ^ only matches at the start of the string
        let condition = ConditionParser::parse_condition("text =~ /^line2/").unwrap();
        let map = HashMap::from([("text".to_string(), "line1\nline2\nline3".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(!condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_flag_s_dotall() {
        // With 's' flag, dot matches newlines
        let condition = ConditionParser::parse_condition("text =~ /start.*end/s").unwrap();
        let map = HashMap::from([("text".to_string(), "start\nmiddle\nend".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_flag_s_no_match_without_flag() {
        // Without 's' flag, dot doesn't match newlines
        let condition = ConditionParser::parse_condition("text =~ /start.*end/").unwrap();
        let map = HashMap::from([("text".to_string(), "start\nmiddle\nend".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(!condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_flags_combined_im() {
        // Combined 'i' and 'm' flags
        let condition = ConditionParser::parse_condition("text =~ /^HELLO/im").unwrap();
        let map = HashMap::from([("text".to_string(), "first line\nhello world".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_flags_combined_is() {
        // Combined 'i' and 's' flags
        let condition = ConditionParser::parse_condition("text =~ /START.*END/is").unwrap();
        let map = HashMap::from([("text".to_string(), "start\nmiddle\nEND".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_flags_combined_ims() {
        // All three flags combined
        let condition = ConditionParser::parse_condition("text =~ /^MIDDLE.*END/ims").unwrap();
        let map = HashMap::from([("text".to_string(), "start\nMIDDLE\nend".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_flag_i_with_character_class() {
        let condition = ConditionParser::parse_condition("text =~ /[a-z]+/i").unwrap();
        let map = HashMap::from([("text".to_string(), "ABCDEF".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_flags_parse_only_i() {
        let condition = ConditionParser::parse_condition("field =~ /pattern/i");
        assert!(condition.is_ok());
    }

    #[test]
    fn test_regex_flags_parse_only_m() {
        let condition = ConditionParser::parse_condition("field =~ /pattern/m");
        assert!(condition.is_ok());
    }

    #[test]
    fn test_regex_flags_parse_only_s() {
        let condition = ConditionParser::parse_condition("field =~ /pattern/s");
        assert!(condition.is_ok());
    }

    #[test]
    fn test_regex_flags_parse_only_ims() {
        let condition = ConditionParser::parse_condition("field =~ /pattern/ims");
        assert!(condition.is_ok());
    }

    #[test]
    fn test_regex_flag_i_with_anchors() {
        let condition = ConditionParser::parse_condition("name =~ /^ALICE$/i").unwrap();
        let map = HashMap::from([("name".to_string(), "alice".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }

    #[test]
    fn test_regex_flag_m_end_of_line() {
        // With 'm' flag, $ matches at end of each line
        let condition = ConditionParser::parse_condition("text =~ /line1$/m").unwrap();
        let map = HashMap::from([("text".to_string(), "line1\nline2\nline3".into())]);
        let event = Event::from(EventSerialized::new(map));
        assert!(condition.evaluate(&event, &mut ResultMap::new()));
    }
}
