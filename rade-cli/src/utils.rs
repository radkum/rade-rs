#![allow(dead_code)]
use std::{collections::HashMap, str::FromStr};

use anyhow::anyhow;
use rade::*;
type Result<T> = core::result::Result<T, Box<dyn core::error::Error>>;

fn create_event() {
    let map = HashMap::from([
        (
            "process_id".to_string(),
            1234.into(),
        ),
        (
            "parent_process_id".to_string(),
            5678.into(),
        ),
        (
            "process_path".to_string(),
            "C:\\path\\to\\exe".into(),
        ),
        (
            "process_name".to_string(),
            "powershell".into(),
        ),
        (
            "script_name".to_string(),
            "script.ps1".into(),
        ),
        (
            "content".to_string(),
            r#""[Ref].Assembly.GetType("System.Management.Automation.AmsiUtils").GetField('amsiInitFailed','NonPublic,Static').SetValue($null,$true)""#.into(),
        ),
        (
            "content_tokens".to_string(),
            vec!["Ref", ".Assembly.GetType", "System.Management.Automation.AmsiUtils", ".GetField", "amsiInitFailed", "NonPublic", "Static", ".SetValue", "null", "true"].into(),
        ),
        (
            "thread_id".to_string(),
            4321.into(),
        ),
        (
            "logon_type".to_string(),
            1.into(),
        ),
    ]);
    let event1 = Event::from(EventSerialized::new(map));
    //let event2 = Event::new(Some(1234), Some(5678),
    // Some("C:\\path\\to\\exe".into()), Some("powershell".into()),
    // Some("script.ps1".into()),
    // Some(r#""[Ref].Assembly.GetType("System.Management.Automation.AmsiUtils").
    // GetField('amsiInitFailed','NonPublic,Static').SetValue($null,$true)""#.
    // into()), Some(4321), Some(1));

    //let events = Events::new(vec![event1, event2]);
    // events
    //     .serialize(&mut
    // std::fs::File::create("rade/test_data/events.yaml").unwrap())
    //     .unwrap();
    println!("{}", serde_yaml_bw::to_string(&event1).unwrap());
}

fn read_rule() -> Result<()> {
    let rule_path = "rade/test_data/rules/amsi_disable/AmsiInitFailed.yaml";
    let rule_yaml = std::fs::read_to_string(rule_path)?;
    let rule = Rule::from_str(&rule_yaml)?;
    println!("RULE: {:?}", rule);
    Ok(())
}

fn serialiaze_ruleset() -> Result<()> {
    let rule_path = "rade/test_data/rules";
    let rule_yaml = std::fs::read_to_string(rule_path)?;
    let rule = Rule::from_str(&rule_yaml)?;
    println!("RULE: {:?}", rule);

    let rule_set = RuleSet::from(rule);
    rule_set.serialize(&mut std::fs::File::create("rade/test_data/ruleset.bin")?)?;
    Ok(())
}

pub fn serialize_ruleset_from_dir() -> Result<()> {
    let rule_path = "rade/test_data/rules";
    let rules = Rules::from_dir(std::path::Path::new(rule_path))
        .map_err(|err| anyhow!("Failed to load rules from dir {}: {:?}", rule_path, err))?;

    let rule_set = RuleSet::from(rules);
    rule_set
        .serialize(&mut std::fs::File::create("rade/test_data/ruleset.bin")?)
        .map_err(|err| anyhow!("Failed to serialize ruleset: {:?}", err))?;
    Ok(())
}

fn create_rule() -> Rule {
    let condition = Operand::And(vec![
        Operand::Contains(
            Val::Field("Content".into()),
            Val::Str("[Ref].Assembly.GetType('System.Management.Automation.AmsiUtils')".into()),
            Some(InsensitiveFlag::CaseAndApostrophe),
        )
        .into(),
        Operand::Contains(
            Val::Field("Content".into()),
            Val::Str(".GetField('amsiInitFailed'".into()),
            Some(InsensitiveFlag::CaseAndApostrophe),
        )
        .into(),
    ]);
    let rule = Rule::new(
        uuid::Uuid::from_str("43025534-69e4-4e81-a78f-fad61111a7df").unwrap(),
        "Bypass Amsi",
        "This rule detects the modification of the amsiInitFailed field to bypass AMSI.",
        "Defense Evasion",
        "TA0005",
        "T1562.001",
        r#""[Ref].Assembly.GetType("System.Management.Automation.AmsiUtils").GetField('amsiInitFailed','NonPublic,Static').SetValue($null,$true)""#,
        condition.into(),
    );
    println!("{}", serde_yaml_bw::to_string(&rule).unwrap());
    rule
}
