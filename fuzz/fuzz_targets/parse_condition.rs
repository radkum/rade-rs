#![no_main]

use libfuzzer_sys::fuzz_target;
use rade::Rule;

const RULE_PREFIX: &str =
    "id: 11111111-2222-3333-4444-555555555555\nname: fuzz\ncondition: |\n";

fuzz_target!(|data: &[u8]| {
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };

    let mut yaml = String::from(RULE_PREFIX);
    for line in input.lines() {
        yaml.push_str("  ");
        yaml.push_str(line);
        yaml.push('\n');
    }

    let _ = Rule::from_yaml(&yaml);
});
