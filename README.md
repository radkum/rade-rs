# RADE-RS — Real-time Advanced Detection Engine

[![Rust](https://img.shields.io/badge/rust-1.89%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![CI](https://github.com/radkum/rade-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/radkum/rade-rs/actions/workflows/rust.yml)

**RADE** (Real-time Advanced Detection Engine) is a Rust rule evaluation engine
for security events. Rules are written in a small YAML DSL, parsed with a
[pest](https://pest.rs/) grammar, and evaluated against typed events (strings,
integers, floats, booleans and lists). Rules can be compiled to a binary format
with a SHA-256 checksum; the core crate builds with `no_std`.

The workspace is at `0.1.0`: the library and CLI work and are covered by tests.
Distribution is via git and GitHub tags; the crates are intentionally **not
published to crates.io**, and RADE is not integrated into any kernel.

## Status and roadmap

- **Distribution**: git / GitHub tags only. No crates.io publication; this avoids
  a permanent maintenance and support commitment before there is a real
  downstream consumer.
- **Scope**: user-space rule engine. Kernel integration is out of scope.
- **Non-goals for now**: new DSL operators, CIDR literals, kernel integration,
  unverified performance claims.
- **Next**: RAMON integration in user space (Rust dependency or `rade.dll` via
  `rade-ffi`) and MSRV/lockfile cleanup.

## Workspace

```
rade-rs/
├── rade/         # Core library: event model, DSL parser, evaluator, serialization
├── rade-cli/     # CLI: compile YAML rules to binary, evaluate events
├── rade-ffi/     # C ABI: builds rade.dll / librade.so + include/rade.h
└── rade-macros/  # Internal proc-macro used to register DSL functions
```

## Quick start

Requires Rust 1.89+ (edition 2024).

```bash
cargo build --workspace
cargo test --workspace
cargo bench -p rade
```

### CLI

```bash
# Compile YAML rules into a binary ruleset
cargo run -p rade-cli -- compile -r rade/test_data/rules -o rade/test_data/ruleset.bin

# Evaluate events (iterative mode)
cargo run -p rade-cli -- eval -r rade/test_data/ruleset.bin -e rade/test_data/events

# Evaluate events using pre-compiled predicates
cargo run -p rade-cli -- eval -r rade/test_data/ruleset.bin -e rade/test_data/events --predicates
```

### Library

Add the dependency via git (or a local path):

```toml
[dependencies]
rade = { git = "https://github.com/radkum/rade-rs.git" }
```

Self-contained example, also available as a runnable
[`examples/eval.rs`](./rade/examples/eval.rs):

```rust
use rade::{Event, Events, RadeEngine, Rule};

const RULE_YAML: &str = r#"
id: 43025534-69e4-4e81-a78f-fad61111a7df
name: Encoded PowerShell Command
condition: |
  Image.ends_with('powershell.exe') && CommandLine.contains('-enc')
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rule = Rule::from_yaml(RULE_YAML)?;

    let mut event = Event::default();
    event.add_string(
        "Image".to_string(),
        r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe".to_string(),
    );
    event.add_string(
        "CommandLine".to_string(),
        "powershell.exe -enc SGVsbG8gV29ybGQ=".to_string(),
    );

    let mut engine = RadeEngine::from_rules(vec![rule].into());
    engine.compile_rules();
    let matches = engine.eval_with_predicates(Events::new(vec![event]))?;

    println!("{matches}");
    Ok(())
}
```

Loading a pre-compiled ruleset:

```rust
use rade::{Events, RadeEngine, RuleSet};
use std::fs::File;
use std::path::Path;

let rule_set = RuleSet::deserialize(&mut File::open("ruleset.bin")?)?;
let mut engine = RadeEngine::from_rules(rule_set.retain_rules());
engine.compile_rules();
let matches = engine.eval_with_predicates(Events::from_dir(Path::new("events/"))?)?;
```

## Evaluation modes

`RadeEngine` exposes two ways to evaluate events:

| Method | Needs `compile_rules()` | When to use |
|--------|--------------------------|-------------|
| `eval_iterative(events)` | no | one-off batches, debugging |
| `eval_with_predicates(events)` | yes | many batches against the same rules (preferred) |

Both are public and equivalent in results. `eval_with_predicates` returns an
error if predicates were not compiled first.

## C / C++ integration

`rade-ffi` builds a shared library named `rade` plus a C header:

```bash
cargo build -p rade-ffi --release
# Windows: target/release/rade.dll
# Linux:   target/release/librade.so
# macOS:   target/release/librade.dylib
```

Header: [`rade-ffi/include/rade.h`](./rade-ffi/include/rade.h).

```c
#include "rade.h"
#include <stdio.h>

int main(void) {
    char *err = NULL;
    RadeEngine *engine = rade_engine_new_from_yaml_dir("rade/test_data/rules", &err);
    if (!engine) {
        fprintf(stderr, "%s\n", err ? err : "failed to load rules");
        rade_string_free(err);
        return 1;
    }

    char *out = NULL;
    int32_t rc = rade_engine_eval_json(
        engine,
        "{\"Image\":\"C:/Windows/System32/WindowsPowerShell/v1.0/powershell.exe\","
        "\"CommandLine\":\"powershell.exe -enc SGVsbG8=\"}",
        &out, &err);

    if (rc == 0) {
        printf("%s\n", out);
    } else {
        fprintf(stderr, "%s\n", err ? err : "evaluation failed");
    }

    rade_string_free(out);
    rade_string_free(err);
    rade_engine_free(engine);
    return 0;
}
```

Use `rade_engine_eval_events_json` for a JSON array of events; it returns one
result object per input event (including non-matches).

Notes:

- A handle is **not thread-safe**. Use one `RadeEngine` per thread; the ABI does
  not require changes for that model.
- Returned strings and error messages must be released with `rade_string_free`.
- Panics inside the library are caught and reported as status `2`, so a bad rule
  or event does not abort the host process. This relies on the default
  `panic = "unwind"`; building with `panic = "abort"` disables that protection.
- Rust applications should depend on the `rade` crate directly instead of going
  through the C ABI.

## Rule DSL

Rules are YAML files with a UUID, optional metadata and a `condition` string:

```yaml
id: 43025534-69e4-4e81-a78f-fad61111a7df
name: AMSI Bypass Detection
description: Detects attempts to bypass AMSI
categories: ["defense_evasion", "amsi"]
mitre_tactic: Defense Evasion
mitre_tactic_id: TA0005
mitre_id: T1562.001
condition: |
  content.contains('AmsiUtils') && content.contains('amsiInitFailed')
```

### Operators

| Operator | Meaning | Example |
|----------|---------|---------|
| `==`, `!=` | equality | `Status == 0` |
| `>`, `<`, `>=`, `<=` | ordering | `EventId > 1000` |
| `~=` | case-insensitive equality | `Image ~= 'CMD.EXE'` |
| `^=` | apostrophe-insensitive equality | `CommandLine ^= '...'` |
| `^~=` | case- and apostrophe-insensitive equality | `CommandLine ^~= '...'` |
| `=~`, `!~` | regex match / non-match | `CommandLine =~ /-enc[a-z]*/i` |
| `&&`, `\|\|`, `!` | logical operators | `A && (B \|\| !C)` |
| `()` | grouping | `(A \|\| B) && C` |

### Methods and functions

String methods: `.contains(s)`, `.starts_with(s)`, `.ends_with(s)`, `.len()`,
`.is_empty()`, `.to_lowercase()`, `.to_uppercase()`, `.trim()`,
`.trim_start()`, `.trim_end()`, `.replace(a, b)`, `.to_string()`.

Integer and float methods: `.abs()`, `.to_string()`; floats additionally
`.floor()`, `.ceil()`, `.round()`.

List methods: `.len()`, `.is_empty()`, `.first()`, `.last()`, `.get(i)`,
`.contains(v)`, `.join(sep)` (string lists), `.sum()`, `.min()`, `.max()`,
`.reverse()`.

Functions: `length(s)`, `is_empty(s)`, `concat(list)`, `split(s, sep)`,
`split_to_digits(n)`, `float_sum(a, b)`.

Field indexing uses brackets: `Arguments[0] == 'cmd.exe'`.

## Events

Events are YAML maps of typed fields:

```yaml
process_name: powershell
process_id: 1234
enabled: true
arguments: ['-enc', 'SGV...']
ports: [443, 8443]
```

Supported types: string, integer, float, boolean, string list, integer list.

## `no_std`

The core crate can be built without the standard library:

```bash
cargo build -p rade --no-default-features
```

Parsing YAML rules and reading files require the default `std` feature.
Pre-compiled binary rulesets and in-memory events can be evaluated in `no_std`.

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | yes | standard library, YAML and file I/O |

## Documentation

- [Library guide](./rade/Readme.md)
- [CLI guide](./rade-cli/Readme.md)
- API reference: `cargo doc -p rade --open`

## Testing

```bash
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build -p rade --no-default-features --locked
cargo run -p rade --example eval --locked
cargo build -p rade-ffi --release --locked
```

The CI also compiles `rade-ffi/tests/smoke.c` as both C and C++ and runs it
against the built shared library.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

- [pest](https://pest.rs/) — PEG parser generator
- [serde](https://serde.rs/) — serialization framework
- [criterion](https://github.com/bheisler/criterion.rs) — benchmarks
- [hashbrown](https://github.com/rust-lang/hashbrown) — `no_std` HashMap

## Contact

- Author: Radosław Kumor
- GitHub: [@radkum](https://github.com/radkum)
