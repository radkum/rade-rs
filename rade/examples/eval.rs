use rade::{Event, Events, RadeEngine, Rule};

const RULE_YAML: &str = r#"
id: 43025534-69e4-4e81-a78f-fad61111a7df
name: Encoded PowerShell Command
description: Detects base64-encoded PowerShell invocations
condition: |
  Image.ends_with('powershell.exe') && CommandLine.contains('-enc')
"#;

fn build_events() -> Events {
    let mut event = Event::default();
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rule = Rule::from_yaml(RULE_YAML)?;

    let iterative = RadeEngine::from_rules(vec![rule.clone()].into());
    println!("iterative evaluation:\n{}", iterative.eval_iterative(build_events()));

    let mut compiled = RadeEngine::from_rules(vec![rule].into());
    compiled.compile_rules();
    println!(
        "predicate evaluation:\n{}",
        compiled.eval_with_predicates(build_events())?
    );

    Ok(())
}
