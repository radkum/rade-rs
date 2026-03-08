# RADE — Real-time Advanced Detection Engine

[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Overview

**RADE** (Real-time Advanced Detection Engine) is a high-performance pattern matching and rule evaluation engine written in Rust. It's designed to analyze security events in real-time, with a focus on detecting malicious or suspicious activities through customizable rules and advanced pattern matching techniques.

RADE provides a powerful Domain-Specific Language (DSL) for writing detection rules that can match against event fields, perform string manipulations, and combine conditions using logical operators.

---

## Features

- **High-Performance Rule Evaluation**  
  Two evaluation modes: iterative and predicate-based (2-3x faster for repeated evaluations)

- **Expressive Rule DSL**  
  Write conditions using field comparisons, string methods, and logical operators

- **Flexible Event Model**  
  Events support multiple field types: strings, integers, booleans, and lists

- **Binary Serialization**  
  Compile rules to optimized binary format with SHA256 integrity verification

- **`no_std` Compatible**  
  Core engine can run in embedded or kernel environments

- **Extensible Architecture**  
  Easy to add new comparison operators and string methods

---

## Installation

Add RADE to your `Cargo.toml`:

```toml
[dependencies]
rade = { path = "../rade" }  # or from crates.io when published
```

## Quick Start

### Basic Usage

```rust
use rade::{Event, Events, RadeEngine, Rules, RuleSet};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load rules from YAML files
    let rules = Rules::from_dir(std::path::Path::new("rules/"))?;
    
    // Create engine and compile predicates for optimal performance
    let mut engine = RadeEngine::from_rules(rules);
    engine.compile_rules();
    
    // Load events to evaluate
    let events = Events::from_dir(std::path::Path::new("events/"))?;
    
    // Evaluate and get matches
    let matches = engine.eval_with_predicates(events)?;
    println!("{}", matches);
    
    Ok(())
}
```

### Using Pre-compiled Rule Sets

```rust
use rade::{Events, RadeEngine, RuleSet};
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load pre-compiled binary ruleset (faster startup)
    let rule_set = RuleSet::deserialize(&mut File::open("ruleset.bin")?)?;
    
    let mut engine = RadeEngine::from_rules(rule_set.retain_rules());
    engine.compile_rules();
    
    let events = Events::from_dir(std::path::Path::new("events/"))?;
    let matches = engine.eval_with_predicates(events)?;
    
    Ok(())
}
```

---

## Rule DSL Reference

Rules are written in YAML with a powerful condition DSL.

### Rule Structure

```yaml
id: 43025534-69e4-4e81-a78f-fad61111a7df       # UUID (required)
name: AMSI Bypass Detection                      # Rule name
description: Detects attempts to bypass AMSI     # Description
categories: ["defense_evasion", "amsi"]          # Category tags
mitre_tactic: Defense Evasion                    # MITRE ATT&CK tactic
mitre_tactic_id: TA0005                          # MITRE tactic ID
mitre_id: T1562.001                              # MITRE technique ID
example: 'Example malicious content'             # Example trigger
condition: |                                     # Detection condition
  content.contains('AmsiUtils') && content.contains('amsiInitFailed')
```

### Condition Syntax

#### Field Access
Access event fields directly by name:
```yaml
condition: process_name == 'powershell'
```

#### String Methods

| Method | Description | Example |
|--------|-------------|---------|
| `.contains(str)` | Check if field contains substring | `content.contains('malware')` |
| `.to_lowercase()` | Convert to lowercase | `process_name.to_lowercase() == 'cmd'` |
| `.to_uppercase()` | Convert to uppercase | `content.to_uppercase().contains('AMSI')` |
| `.replace(old, new)` | Replace substring | `path.replace('\\', '/').contains('/temp/')` |
| `.len()` | Get string length | `content.len() > 1000` |

#### Comparison Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `==` | Equal | `process_name == 'cmd'` |
| `!=` | Not equal | `process_id != 0` |
| `>` | Greater than | `content.len() > 100` |
| `<` | Less than | `process_id < 1000` |
| `>=` | Greater or equal | `severity >= 5` |
| `<=` | Less or equal | `risk_score <= 10` |

#### Logical Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `&&` | Logical AND | `a == 1 && b == 2` |
| `\|\|` | Logical OR | `a == 1 \|\| b == 2` |
| `()` | Grouping | `(a == 1 \|\| b == 2) && c == 3` |

### Example Rules

**Detect AMSI Bypass:**
```yaml
id: 43025534-69e4-4e81-a78f-fad61111a7df
name: AMSI Bypass Detection
condition: |
  content.contains('AmsiUtils') && content.contains('amsiInitFailed')
```

**Detect Encoded PowerShell Commands:**
```yaml
id: 87654321-4321-8765-4321-876543210987
name: Encoded PowerShell Command
condition: |
  process_name.to_lowercase() == 'powershell' && 
  content.contains('-enc') || content.contains('-EncodedCommand')
```

**Detect Suspicious Process IDs:**
```yaml
id: 22334455-6677-8899-aabb-ccddeeff0011
name: Unusual Process ID
condition: |
  process_id > 50000
```

**Complex String Manipulation:**
```yaml
id: 33445566-7788-99aa-bbcc-ddeeff001122
name: String Manipulation Test
condition: |
  content.len() > 10 && content.to_uppercase().contains('HELLO')
```

---

## Event Format

Events are YAML files with key-value pairs:

```yaml
# Event fields - all fields are optional
process_name: 'powershell'
process_id: 1234
parent_process_id: 5678
process_path: 'C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe'
content: '[Ref].Assembly.GetType("System.Management.Automation.AmsiUtils")'
```

### Supported Field Types

- **String**: `field_name: 'string value'`
- **Integer**: `field_name: 1234`
- **Boolean**: `field_name: true`
- **String List**: `field_name: ['item1', 'item2']`
- **Integer List**: `field_name: [1, 2, 3]`

---

## API Reference

### Core Types

| Type | Description |
|------|-------------|
| `RadeEngine` | Main evaluation engine |
| `Rules` | Collection of detection rules |
| `RuleSet` | Serializable rule container with metadata |
| `Events` | Collection of events to evaluate |
| `Event` | Single event with typed fields |
| `Matches` | Evaluation results |

### RadeEngine Methods

```rust
impl RadeEngine {
    /// Create engine from rules
    pub fn from_rules(rules: Rules) -> Self;
    
    /// Load rules into existing engine
    pub fn load_rules(&mut self, rules: Rules);
    
    /// Compile predicates for faster evaluation
    pub fn compile_rules(&mut self);
    
    /// Evaluate using iterative method (no compilation needed)
    pub fn eval_iterative(&mut self, events: Events) -> Matches;
    
    /// Evaluate using compiled predicates (faster, requires compile_rules())
    pub fn eval_with_predicates(&mut self, events: Events) -> RadeResult<Matches>;
}
```

### Loading Data

```rust
// Load rules from directory (recursively)
let rules = Rules::from_dir(Path::new("rules/"))?;

// Load events from directory
let events = Events::from_dir(Path::new("events/"))?;

// Deserialize pre-compiled ruleset
let rule_set = RuleSet::deserialize(&mut file)?;
```

---

## Testing

### Run Unit Tests

```bash
cargo test -p rade
```

### Run with Verbose Output

```bash
cargo test -p rade -- --nocapture
```

### Test Specific Module

```bash
cargo test -p rade rule_set::
cargo test -p rade event::
```

---

## Benchmarks

RADE includes comprehensive benchmarks using [Criterion](https://github.com/bheisler/criterion.rs).

### Run All Benchmarks

```bash
cargo bench -p rade
```

### Run Specific Benchmark Group

```bash
# Compare evaluation methods
cargo bench -p rade -- comparison

# Test scaling with different event counts
cargo bench -p rade -- scaling

# Benchmark rule loading
cargo bench -p rade -- load_rules
```

### Benchmark Groups

| Group | Description |
|-------|-------------|
| `load_rules_from_yaml` | Time to parse YAML rule files |
| `load_events_from_yaml` | Time to parse YAML event files |
| `engine_from_rules` | Engine creation overhead |
| `compile_predicates` | Predicate compilation time |
| `eval_iterative` | Iterative evaluation (no compilation) |
| `eval_with_predicates` | Predicate-based evaluation |
| `scaling_events` | Performance with 1x-8x event counts |
| `comparison` | Direct comparison of evaluation methods |

### Quick Benchmark Test

```bash
# Verify benchmarks compile without running full suite
cargo bench -p rade -- --test
```

### Performance Characteristics

Based on typical workloads (11 rules, 16 events):

| Method | Time | Notes |
|--------|------|-------|
| Iterative | ~1.2ms | Simple, no compilation needed |
| Predicate (with compile) | ~600µs | Includes compilation overhead |
| Predicate (pre-compiled) | ~500µs | **Fastest** for repeated use |

**Recommendation**: Use predicate-based evaluation with `compile_rules()` when evaluating multiple event batches against the same ruleset.

---

## Project Structure

```
rade/
├── Cargo.toml              # Package manifest
├── Readme.md               # This file
├── benches/
│   └── engine_benchmarks.rs # Criterion benchmarks
├── src/
│   ├── lib.rs              # Public API exports
│   ├── event.rs            # Event types and loading
│   ├── match_.rs           # Match result types
│   ├── rade_engine.rs      # Core evaluation engine
│   ├── rule_set.rs         # RuleSet serialization
│   ├── utils.rs            # Utility types
│   ├── event/
│   │   └── serializer.rs   # Event serialization
│   ├── rule_set/
│   │   ├── error.rs        # Error types
│   │   ├── predicates.rs   # Predicate compilation
│   │   ├── rule.rs         # Rule struct and parsing
│   │   └── rules.rs        # Rules collection
│   └── utils/
│       ├── fat_regex.rs    # Regex wrapper
│       └── fat_string.rs   # String wrapper
└── test_data/
    ├── ruleset.bin         # Pre-compiled rules
    ├── events/             # Test event files
    └── rules/              # Test rule files
```

---

## Use Cases

- **Endpoint Detection and Response (EDR)** — Real-time threat detection
- **SIEM Integration** — Log analysis and alerting
- **Malware Research** — Behavioral analysis and classification
- **Compliance Monitoring** — Policy violation detection
- **Incident Response** — Forensic analysis automation

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request
