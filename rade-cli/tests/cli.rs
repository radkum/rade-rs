use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const RULE_UUID: &str = "11111111-2222-3333-4444-555555555555";
const RULE_YAML: &str = r#"
id: 11111111-2222-3333-4444-555555555555
name: PowerShell Execution
condition: |
  process_name == 'powershell'
"#;
const EVENT_YAML: &str = r#"
process_name: powershell
process_id: 1234
"#;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_rade-cli")
}

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "rade-cli-{tag}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn compile(rules: &Path, ruleset: &Path) -> std::process::Output {
    Command::new(bin())
        .args(["compile", "-r"])
        .arg(rules)
        .arg("-o")
        .arg(ruleset)
        .output()
        .unwrap()
}

#[test]
fn compile_then_eval_matches_in_both_modes() {
    let dir = unique_dir("compile-eval");
    let rules_dir = dir.join("rules");
    let events_dir = dir.join("events");
    let ruleset = dir.join("ruleset.bin");

    write(&rules_dir.join("rule.yaml"), RULE_YAML);
    write(&events_dir.join("event.yaml"), EVENT_YAML);

    let compiled = compile(&rules_dir, &ruleset);
    assert!(
        compiled.status.success(),
        "compile failed: {}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    assert!(ruleset.exists());

    for extra in [&[][..], &["--predicates"][..]] {
        let output = Command::new(bin())
            .args(["eval", "-r"])
            .arg(&ruleset)
            .arg("-e")
            .arg(&events_dir)
            .args(extra)
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "eval failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains(RULE_UUID),
            "expected matched rule id in stdout, got: {stdout}"
        );
    }
}

#[test]
fn compile_rejects_invalid_condition() {
    let dir = unique_dir("invalid-rule");
    let rules_dir = dir.join("rules");
    let ruleset = dir.join("ruleset.bin");

    write(
        &rules_dir.join("broken.yaml"),
        r#"
id: 22222222-3333-4444-5555-666666666666
name: Broken
condition: ''
"#,
    );

    let compiled = compile(&rules_dir, &ruleset);
    assert!(
        !compiled.status.success(),
        "expected compile to fail for an empty condition"
    );
    assert!(!ruleset.exists());
}
