use rade::{Event, Events, RadeEngine, Rule, RuleSet, Rules};

const RULE_UUID: &str = "43025534-69e4-4e81-a78f-fad61111a7df";
const RULE_YAML: &str = r#"
id: 43025534-69e4-4e81-a78f-fad61111a7df
name: Encoded PowerShell Command
condition: |
  Image.ends_with('powershell.exe') && CommandLine.contains('-enc')
"#;

fn rule() -> Rule {
    Rule::from_yaml(RULE_YAML).unwrap()
}

fn events() -> Events {
    let mut event = Event::default();
    event.set_name("encoded");
    event.add_string(
        "Image".to_string(),
        r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe".to_string(),
    );
    event.add_string(
        "CommandLine".to_string(),
        "powershell.exe -enc SGVsbG8gV29ybGQ=".to_string(),
    );
    Events::new(vec![event])
}

#[test]
fn public_api_evaluates_in_both_modes() {
    let mut engine = RadeEngine::from_rules(Rules::from(vec![rule()]));

    let iterative = engine.eval_iterative(events());
    assert_eq!(iterative.len(), 1);

    let (event, matched) = iterative.iter().next().unwrap();
    assert_eq!(event.name(), Some("encoded"));

    let ids: Vec<String> = matched.iter().map(ToString::to_string).collect();
    assert_eq!(ids, vec![RULE_UUID]);

    let rule_name = engine
        .rules()
        .get(matched.iter().next().unwrap())
        .and_then(Rule::name);
    assert_eq!(rule_name, Some("Encoded PowerShell Command"));

    engine.compile_rules();
    let predicate = engine.eval_with_predicates(events()).unwrap();
    assert_eq!(predicate.len(), 1);
    assert_eq!(
        predicate
            .iter()
            .next()
            .unwrap()
            .1
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec![RULE_UUID]
    );
}

#[test]
fn ruleset_roundtrips_through_binary() {
    let ruleset = RuleSet::from(Rules::from(vec![rule()]));
    let mut bytes = ruleset.serialize_to_bytes().unwrap();

    let restored = RuleSet::deserialize_from_bytes(&mut bytes).unwrap();
    assert_eq!(restored.rules().iter().count(), 1);

    let engine = RadeEngine::from_rules(restored.retain_rules());
    assert_eq!(engine.eval_iterative(events()).len(), 1);
}

#[test]
fn predicate_eval_requires_compilation() {
    let engine = RadeEngine::from_rules(Rules::from(vec![rule()]));
    assert!(engine.eval_with_predicates(events()).is_err());
}
