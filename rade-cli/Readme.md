# RADE-CLI — Command Line Interface for RADE

[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Overview

**RADE-CLI** is the command-line interface for the [RADE](../rade/Readme.md) (Real-time Advanced Detection Engine). It provides a convenient way to:

- Load and compile detection rules from YAML files
- Serialize rules to optimized binary format
- Evaluate events against compiled rules
- Display match results

---

## Features

- **Rule Compilation** — Convert YAML rules to binary format for faster loading
- **Event Evaluation** — Process events and detect matches
- **Debug Logging** — Detailed logging for troubleshooting
- **Batch Processing** — Evaluate multiple events against multiple rules

---

## Installation

### Build from Source

```bash
# Clone the repository
git clone https://github.com/radkum/rade-rs.git
cd rade-rs

# Build in release mode
cargo build -p rade-cli --release

# The binary will be at: target/release/rade-cli.exe (Windows)
# Or: target/release/rade-cli (Linux/macOS)
```

### Development Build

```bash
cargo build -p rade-cli
```

---

## Quick Start

### 1. Prepare Your Rules

Create YAML rule files in a directory. Each rule should have a unique UUID:

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

### 2. Prepare Your Events

Create YAML event files in a directory:

```yaml
# events/suspicious_event.yaml
process_name: 'powershell'
process_id: 1234
content: '[Ref].Assembly.GetType("System.Management.Automation.AmsiUtils").GetField("amsiInitFailed")'
```

### 3. Run Detection

```bash
cargo run -p rade-cli
```

The CLI will:
1. Load rules from `rade/test_data/rules/`
2. Compile and serialize rules to `rade/test_data/ruleset.bin`
3. Load events from `rade/test_data/events/`
4. Evaluate all events against all rules
5. Display matches

---

## Usage

### Basic Execution

```bash
# From workspace root
cargo run -p rade-cli

# Or run the compiled binary directly
./target/release/rade-cli
```

### Sample Output

```
Rules serialized successfully
Loaded 16 events
Loaded 11 rules
Event Matches:
╔══════════════════════════════════════════════════════════════════════════════╗
║ Event: amsi_bypass                                                            ║
║ Matched Rules: [43025534-69e4-4e81-a78f-fad61111a7df]                        ║
╠══════════════════════════════════════════════════════════════════════════════╣
║ Event: credential_dump                                                        ║
║ Matched Rules: [55667788-99aa-bbcc-ddee-ff0011223344]                        ║
╠══════════════════════════════════════════════════════════════════════════════╣
║ Event: encoded_command                                                        ║
║ Matched Rules: [87654321-4321-8765-4321-876543210987]                        ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### Log Levels

The CLI uses `env_logger` for logging. Control verbosity with `RUST_LOG`:

```bash
# Windows PowerShell
$env:RUST_LOG="debug"; cargo run -p rade-cli

# Windows CMD
set RUST_LOG=debug && cargo run -p rade-cli

# Linux/macOS
RUST_LOG=debug cargo run -p rade-cli
```

Available log levels:
- `error` — Only errors
- `warn` — Warnings and errors
- `info` — Informational messages
- `debug` — Detailed debug output (default)
- `trace` — Very verbose tracing

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

## Directory Structure

The CLI expects the following structure relative to the workspace root:

```
rade-rs/
├── rade/
│   └── test_data/
│       ├── ruleset.bin          # Generated: compiled rules
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
        ├── main.rs              # CLI entry point
        └── utils.rs             # Helper functions
```

---

## Testing

### Run CLI Tests

```bash
cargo test -p rade-cli
```

### Manual Testing

1. **Add a new rule:**
   ```yaml
   # rade/test_data/rules/amsi_disable/MyRule.yaml
   id: 11111111-2222-3333-4444-555555555555
   name: My Custom Rule
   condition: |
     process_name == 'suspicious.exe'
   ```

2. **Add a test event:**
   ```yaml
   # rade/test_data/events/test_event.yaml
   process_name: 'suspicious.exe'
   process_id: 9999
   ```

3. **Run and verify:**
   ```bash
   cargo run -p rade-cli
   ```

### Verify Rule Compilation

The CLI outputs "Rules serialized successfully" when rules are compiled without errors. Check for parse errors in the debug output if rules fail to load.

---

## Benchmarking

The RADE library (not CLI) includes Criterion benchmarks:

```bash
# Run all benchmarks
cargo bench -p rade

# Run specific benchmark
cargo bench -p rade -- comparison

# Quick test (no full measurement)
cargo bench -p rade -- --test
```

See [RADE Readme](../rade/Readme.md#benchmarks) for benchmark details.

---

## Troubleshooting

### Common Issues

#### "Failed to load events from dir"
- Ensure you're running from the workspace root (`rade-rs/`)
- Check that `rade/test_data/events/` exists and contains YAML files

#### "Failed to read rule from path"
- Check YAML syntax in rule files
- Verify UUID format: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`
- Ensure condition syntax is valid

#### "Field not found" warnings
- This is expected when a rule checks for a field that doesn't exist in an event
- Events only need fields referenced by matching rules

#### "Predicates not compiled"
- Call `engine.compile_rules()` before `eval_with_predicates()`
- Or use `eval_iterative()` which doesn't require compilation

### Debug Mode

Enable full debug logging:

```bash
# Windows PowerShell
$env:RUST_LOG="trace"; cargo run -p rade-cli

# This shows:
# - Each rule being loaded
# - Each event being parsed
# - Each predicate evaluation
# - Match/no-match decisions
```

---

## Configuration

### Customizing Paths

Edit `rade-cli/src/main.rs` to change default paths:

```rust
// Change event directory
let events_path = PathBuf::from("rade/test_data/events");

// Change rule directory (in utils.rs)
let rules_path = PathBuf::from("rade/test_data/rules");

// Change output binary path
let output_path = PathBuf::from("rade/test_data/ruleset.bin");
```

### Adding New Features

The CLI is designed to be extended. Common additions:

1. **Command-line arguments** — Use `clap` for argument parsing
2. **JSON output** — Serialize matches to JSON for tooling integration
3. **Watch mode** — Monitor directories for new events
4. **Remote rules** — Fetch rules from HTTP endpoints

---

## Examples

### Example: Security Monitoring Setup

```bash
# 1. Create rules for your environment
mkdir -p rules/custom

# 2. Add detection rules
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

# 3. Run detection
cargo run -p rade-cli
```

### Example: Batch Event Processing

```bash
# Process a directory of exported events
# (assuming events are in YAML format)

# 1. Copy events to test_data/events
cp /path/to/exported/events/*.yaml rade/test_data/events/

# 2. Run detection
cargo run -p rade-cli > results.txt

# 3. Review matches
cat results.txt
```

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## See Also

- [RADE Library Documentation](../rade/Readme.md) — Core engine documentation
- [Rule DSL Reference](../rade/Readme.md#rule-dsl-reference) — Condition syntax guide
- [Benchmarks](../rade/Readme.md#benchmarks) — Performance information

