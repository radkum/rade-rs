# RADE-CLI — Command Line Interface for RADE

[![Rust](https://img.shields.io/badge/rust-1.89%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/radkum/rade-rs/blob/master/LICENSE-MIT)

## Overview

**RADE-CLI** is the command-line interface for the [RADE](../rade/Readme.md)
(Real-time Advanced Detection Engine). It provides a convenient way to:

- Load and parse detection rules from YAML files
- Serialize rules to an optimized binary format
- Evaluate events against compiled rules
- Display match results

The CLI is a thin wrapper around the `rade` library. Paths are always passed
explicitly with flags; there are no hard-coded default paths.

---

## Installation

### Build from Source

```bash
git clone https://github.com/radkum/rade-rs.git
cd rade-rs

cargo build -p rade-cli --release
```

The binary is written to `target/release/rade-cli.exe` on Windows or
`target/release/rade-cli` on Linux/macOS.

### Development Build

```bash
cargo build -p rade-cli
```

---

## Commands

```
rade-cli [--verbose] <COMMAND>

Commands:
  compile  Compile YAML rules to binary format
  eval     Evaluate events against compiled rules
```

### `compile`

| Flag | Description |
|------|-------------|
| `-r, --rules <PATH>` | Rules directory (searched recursively) or a single YAML file |
| `-o, --output <FILE>` | Output path for the compiled binary ruleset |

```bash
cargo run -p rade-cli -- compile \
  -r rade/test_data/rules \
  -o rade/test_data/ruleset.bin
```

### `eval`

| Flag | Description |
|------|-------------|
| `-r, --rules <FILE>` | Compiled binary ruleset |
| `-e, --events <PATH>` | Events directory or a single YAML file |
| `-p, --predicates` | Use predicate-based evaluation (compiled once, faster for repeated use) |

```bash
cargo run -p rade-cli -- eval \
  -r rade/test_data/ruleset.bin \
  -e rade/test_data/events

cargo run -p rade-cli -- eval \
  -r rade/test_data/ruleset.bin \
  -e rade/test_data/events \
  --predicates
```

With a compiled binary you can also invoke the built executable directly:

```bash
./target/release/rade-cli compile -r rules/ -o ruleset.bin
./target/release/rade-cli eval -r ruleset.bin -e events/
```

Use `--help` on the binary or subcommand for the full flag list:

```bash
cargo run -p rade-cli -- --help
cargo run -p rade-cli -- eval --help
```

---

## Quick Start

### 1. Prepare rules

Create YAML rule files (a directory or a single file). Each rule needs a UUID
and a `condition`:

```yaml
# rules/amsi_bypass.yaml
id: 43025534-69e4-4e81-a78f-fad61111a7df
name: AMSI Bypass Detection
description: Detects attempts to bypass AMSI
categories: ["defense_evasion", "amsi"]
mitre_tactic: Defense Evasion
condition: |
  content.contains('AmsiUtils') && content.contains('amsiInitFailed')
```

### 2. Prepare events

Create YAML event files:

```yaml
# events/suspicious_event.yaml
process_name: powershell
process_id: 1234
content: '[Ref].Assembly.GetType("System.Management.Automation.AmsiUtils").GetField("amsiInitFailed")'
```

### 3. Compile and evaluate

```bash
cargo run -p rade-cli -- compile -r rules/ -o ruleset.bin
cargo run -p rade-cli -- eval -r ruleset.bin -e events/
```

---

## Sample Output

`compile` reports how many rules were loaded and where the binary was written:

```
[INFO  rade_cli] Compiling rules from: rade/test_data/rules
[INFO  rade_cli] Loaded 11 rule(s)
[INFO  rade_cli] Successfully compiled 11 rule(s) to: rade/test_data/ruleset.bin
✓ Compiled 11 rule(s) to rade/test_data/ruleset.bin
```

`eval` prints one entry per event that matched at least one rule:

```
Evaluating 16 event(s) against 11 rule(s)...

Matches { 
	0: "amsi_bypass", Matched: [43025534-69e4-4e81-a78f-fad61111a7df]
	1: "credential_dump", Matched: [99887766-5544-3322-1100-ffeeddccbbaa]
	2: "encoded_command", Matched: [87654321-4321-8765-4321-876543210987]
}

✓ Evaluation complete for 16 event(s)
```

The rule UUIDs come from your rule files, so the exact values will differ.

---

## Log Levels

Logging uses `env_logger`. The default level is `info`; pass `-v` / `--verbose`
for `debug`:

```bash
cargo run -p rade-cli -- --verbose compile -r rules/ -o ruleset.bin
cargo run -p rade-cli -- -v eval -r ruleset.bin -e events/
```

`RUST_LOG` can be used for per-module filtering:

```bash
# Windows PowerShell
$env:RUST_LOG="rade=debug"; cargo run -p rade-cli -- eval -r ruleset.bin -e events/

# Linux/macOS
RUST_LOG=rade=debug cargo run -p rade-cli -- eval -r ruleset.bin -e events/
```

Available levels: `error`, `warn`, `info`, `debug`, `trace`.

---

## Workflow

### Rule Processing Pipeline

```
┌─────────────────┐
│  YAML Rules     │  ← Human-readable rule definitions
│  (rules/*.yaml) │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Rule Parser    │  ← Parse condition DSL
│  (pest grammar) │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Binary RuleSet │  ← Optimized binary format
│  (ruleset.bin)  │     with SHA256 checksum
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  RadeEngine     │  ← Compile predicates
│  (in memory)    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Evaluation     │  ← Match events against rules
│  (matches)      │
└─────────────────┘
```

### Event Processing Pipeline

```
┌─────────────────┐
│  YAML Events    │  ← Event data files
│  (events/*.yaml)│
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Event Parser   │  ← Parse fields by type
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Evaluation     │  ← Check each event against
│                 │     compiled predicates
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Match Results  │  ← List of matched rule IDs
│                 │     per event
└─────────────────┘
```

---

## Example Layout

The repository ships sample data under `rade/test_data/`:

```
rade-rs/
├── rade/
│   └── test_data/
│       ├── events/              # Input: event YAML files
│       │   ├── event1.yaml
│       │   ├── event2.yaml
│       │   └── ...
│       └── rules/               # Input: rule YAML files
│           └── amsi_disable/
│               ├── rule1.yaml
│               ├── rule2.yaml
│               └── ...
└── rade-cli/
    └── src/
        └── main.rs              # CLI entry point
```

`ruleset.bin` is generated wherever you point `-o`; it is not committed.

---

## Testing

```bash
cargo test -p rade-cli
```

### Manual Testing

1. Add a rule:

   ```yaml
   # rade/test_data/rules/amsi_disable/MyRule.yaml
   id: 11111111-2222-3333-4444-555555555555
   name: My Custom Rule
   condition: |
     process_name == 'suspicious.exe'
   ```

2. Add an event:

   ```yaml
   # rade/test_data/events/test_event.yaml
   process_name: suspicious.exe
   process_id: 9999
   ```

3. Compile and evaluate:

   ```bash
   cargo run -p rade-cli -- compile -r rade/test_data/rules -o rade/test_data/ruleset.bin
   cargo run -p rade-cli -- eval -r rade/test_data/ruleset.bin -e rade/test_data/events
   ```

### Verify Rule Compilation

On success `compile` prints:

```
✓ Compiled N rule(s) to <output path>
```

If a rule fails to parse, `compile` exits with an error that names the failing
rule and the parser message.

---

## Benchmarking

The RADE library (not the CLI) includes Criterion benchmarks:

```bash
cargo bench -p rade
cargo bench -p rade -- comparison
cargo bench -p rade -- --test
```

See [RADE Readme](../rade/Readme.md#benchmarks) for benchmark details.

---

## Troubleshooting

### `Failed to load events`
- Check the path passed with `-e`.
- The path must be an existing file or directory.
- Event files must be valid YAML.

### `Failed to read rule` / parse error
- Check YAML syntax in rule files.
- Verify the UUID format: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`.
- Ensure the condition uses the DSL syntax (`&&`, `||`, `!`, methods such as
  `.contains('...')`).

### Fields missing in events
- A rule referencing a field that is absent from an event is treated as a
  non-match; this is expected.
- Events only need the fields referenced by the rules you evaluate.

### Predicate mode
- With `--predicates` the CLI calls `compile_rules()` automatically before
  `eval_with_predicates()`. Without the flag it uses `eval_iterative()`, which
  needs no compilation.

### Debug mode

```bash
# Windows PowerShell
cargo run -p rade-cli -- -v eval -r rade/test_data/ruleset.bin -e rade/test_data/events
```

This shows rule loading, event parsing, predicate evaluation and per-rule
match decisions.

---

## Configuration

All input/output locations are command-line arguments:

| Value | Flag | Notes |
|-------|------|-------|
| Rules input | `-r` on `compile` | directory or single YAML file |
| Ruleset output | `-o` on `compile` | parent directories are created |
| Ruleset input | `-r` on `eval` | binary produced by `compile` |
| Events input | `-e` on `eval` | directory or single YAML file |
| Evaluation mode | `--predicates` on `eval` | omit for iterative mode |

There are no default paths to edit.

### Future additions

The CLI is intentionally small. Possible extensions (not implemented):

1. JSON output for tooling integration
2. Watch mode for new event files
3. Fetching rules from a remote endpoint

---

## Examples

### Security monitoring setup

```bash
mkdir -p rules/custom

cat > rules/custom/detect_mimikatz.yaml << EOF
id: aaaabbbb-cccc-dddd-eeee-ffffffffffff
name: Mimikatz Detection
description: Detects Mimikatz execution patterns
categories: ["credential_access"]
mitre_tactic: Credential Access
mitre_id: T1003
condition: |
  content.contains('sekurlsa') || content.contains('kerberos::')
EOF

cargo run -p rade-cli -- compile -r rules/custom -o ruleset.bin
cargo run -p rade-cli -- eval -r ruleset.bin -e events/
```

### Batch event processing

```bash
cp /path/to/exported/events/*.yaml events/
cargo run -p rade-cli -- compile -r rules/ -o ruleset.bin
cargo run -p rade-cli -- eval -r ruleset.bin -e events/ > results.txt
```

---

## License

Licensed under either of Apache-2.0
([LICENSE-APACHE](https://github.com/radkum/rade-rs/blob/master/LICENSE-APACHE))
or MIT ([LICENSE-MIT](https://github.com/radkum/rade-rs/blob/master/LICENSE-MIT)),
at your option.

---

## See Also

- [RADE Library Documentation](../rade/Readme.md) — Core engine documentation
- [Rule DSL Reference](../rade/Readme.md#rule-dsl-reference) — Condition syntax guide
- [Benchmarks](../rade/Readme.md#benchmarks) — Performance information
