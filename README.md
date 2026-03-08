# RADE-RS — Real-time Advanced Detection Engine

[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

<p align="center">
  <strong>A high-performance, Rust-based rule evaluation engine for real-time security event detection</strong>
</p>

---

## 🎯 Overview

**RADE** (Real-time Advanced Detection Engine) is a powerful pattern matching and rule evaluation engine written in Rust. It's designed for analyzing security events in real-time, detecting malicious or suspicious activities through customizable rules and advanced pattern matching techniques.

### Key Highlights

- 🚀 **Blazingly Fast** — Optimized for millions of events per second
- 🔒 **Memory Safe** — Written in 100% safe Rust
- 📦 **`no_std` Compatible** — Run in embedded or kernel environments
- 🎨 **Expressive DSL** — Intuitive rule definition language
- 🔌 **Extensible** — Easy to add custom operators and functions

---

## 📦 Workspace Structure

This repository is a Cargo workspace containing multiple crates:

```
rade-rs/
├── rade/           # Core detection engine library
├── rade-cli/       # Command-line interface tool
└── macros/         # Procedural macros for code generation
```

| Crate | Description |
|-------|-------------|
| [`rade`](./rade/) | Core library implementing the detection engine, rule DSL parser, and evaluation logic |
| [`rade-cli`](./rade-cli/) | CLI tool for compiling rules and evaluating events |
| `macros` | Internal procedural macros (function registration) |

---

## 🚀 Quick Start

### Prerequisites

- **Rust 1.85+** (2024 edition)
- **Cargo** (comes with Rust)

### Installation

```bash
# Clone the repository
git clone https://github.com/radkum/rade-rs.git
cd rade-rs

# Build all crates
cargo build --release

# Run tests
cargo test --workspace

# Run benchmarks
cargo bench -p rade
```

### Using the CLI

```bash
# Compile YAML rules to binary format
cargo run -p rade-cli -- compile -r rules/ -o ruleset.bin

# Evaluate events against rules
cargo run -p rade-cli -- eval -r ruleset.bin -e events/
```

### Using as a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
rade = { git = "https://github.com/radkum/rade-rs.git" }
```

Basic usage:

```rust
use rade::{Event, RadeEngine, Rules, RuleSet};

fn main() -> anyhow::Result<()> {
    // Load compiled rules
    let ruleset = RuleSet::from_bin_file("ruleset.bin")?;
    
    // Create the engine
    let engine = RadeEngine::new(ruleset);
    
    // Create an event
    let event = Event::default()
        .with_string("CommandLine", "powershell.exe -enc SGVsbG8gV29ybGQ=")
        .with_string("Image", "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe");
    
    // Evaluate
    let matches = engine.eval(&event);
    
    for rule in matches {
        println!("Matched: {}", rule.name());
    }
    
    Ok(())
}
```

---

## 📝 Rule DSL

RADE uses an expressive Domain-Specific Language for defining detection rules:

### YAML Rule Format

```yaml
name: Suspicious PowerShell Execution
description: Detects encoded PowerShell commands
author: Security Team
severity: high
tags:
  - attack.execution
  - attack.t1059.001

condition: |
  Image endswith "powershell.exe" and
  (CommandLine.contains('-enc') or CommandLine.contains('-encodedcommand')
```

### Supported Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `==`, `!=` | Equality | `Status == 0` |
| `>`, `<`, `>=`, `<=` | Comparison | `EventId > 1000` |
| `contains` | Substring match | `CommandLine contains "mimikatz"` |
| `startswith` | Prefix match | `Image startswith "C:\\Windows"` |
| `endswith` | Suffix match | `Image endswith ".exe"` |
| `matches` | Regex match | `CommandLine matches /[A-Za-z0-9+\/=]{40,}/` |
| `in` | List membership | `EventId in [1, 4688, 4624]` |
| `and`, `or`, `not` | Logical operators | `A and (B or not C)` |

### String Methods

```yaml
# Case-insensitive comparison
condition: CommandLine.to_lowercase().contains('password') 

# String manipulation
condition: length(CommandLine) > 1000

# Field indexing (for lists)
condition: CommandLineArgs[0] == "cmd.exe"
```

---

## ⚡ Performance

RADE is designed for high-throughput scenarios:

| Benchmark | Events/sec | Notes |
|-----------|------------|-------|
| Simple rule evaluation | ~2M | Single field comparison |
| Complex rule (5+ conditions) | ~500K | Multiple operators |
| Predicate-based evaluation | ~1.5M | Pre-compiled conditions |
| 100 rules × 1000 events | ~50K | Batch processing |

Run benchmarks yourself:

```bash
cargo bench -p rade
```

---

## 🔧 Features

### `no_std` Support

RADE core can run without the standard library:

```toml
[dependencies]
rade = { git = "...", default-features = false }
```

**Note**: Some features (rule parsing, file I/O) require `std`. Pre-compiled binary rules can be evaluated in `no_std` environments.

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | ✅ | Standard library support |

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      RADE Engine                         │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │
│  │   Parser    │  │  Compiler   │  │   Evaluator     │  │
│  │   (pest)    │──│  (binary)   │──│  (iterative/    │  │
│  │             │  │             │  │   predicate)    │  │
│  └─────────────┘  └─────────────┘  └─────────────────┘  │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────┐    │
│  │                    Event Model                   │    │
│  │  strings | integers | booleans | lists          │    │
│  └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

### Core Components

1. **Parser** — Parses rule DSL using PEG grammar (pest)
2. **Compiler** — Converts rules to optimized binary format
3. **Evaluator** — Matches events against compiled rules
4. **Event Model** — Flexible structure for security events

---

## 📚 Documentation

- [RADE Library Documentation](./rade/Readme.md)
- [RADE-CLI Usage Guide](./rade-cli/Readme.md)
- [Rule Writing Guide](./docs/rules.md) *(coming soon)*
- [API Reference](https://docs.rs/rade) *(when published)*

---

## 🧪 Testing

```bash
# Run all tests
cargo test --workspace

# Run with logging
RUST_LOG=debug cargo test --workspace -- --nocapture

# Run specific test
cargo test -p rade test_rule_evaluation
```

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Setup

```bash
# Install Rust nightly (for some dev tools)
rustup install nightly

# Format code
cargo +nightly fmt

# Run clippy
cargo clippy --workspace --all-targets

# Run all checks before committing
cargo test --workspace && cargo clippy --workspace
```

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- [pest](https://pest.rs/) — PEG parser generator
- [serde](https://serde.rs/) — Serialization framework
- [criterion](https://github.com/bheisler/criterion.rs) — Benchmarking library
- [hashbrown](https://github.com/rust-lang/hashbrown) — `no_std` compatible HashMap

---

## 📬 Contact

- **Author**: Radosław Kumor
- **Email**: radoslaw.kumorekit@gmail.com
- **GitHub**: [@radkum](https://github.com/radkum)

---

<p align="center">
  <sub>Built with ❤️ in Rust</sub>
</p>
