mod utils;

use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use rade::*;

#[derive(Parser)]
#[command(name = "rade-cli")]
#[command(author = "Radosław Kumor <radoslaw.kumorekit@gmail.com>")]
#[command(version = "0.0.1")]
#[command(about = "RADE - Real-time Advanced Detection Engine CLI", long_about = None)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile YAML rules to binary format
    Compile {
        /// Path to rules directory or single YAML file
        #[arg(short, long, value_name = "PATH")]
        rules: PathBuf,

        /// Output path for compiled binary ruleset
        #[arg(short, long, value_name = "FILE")]
        output: PathBuf,
    },

    /// Evaluate events against compiled rules
    Eval {
        /// Path to compiled binary ruleset
        #[arg(short, long, value_name = "FILE")]
        rules: PathBuf,

        /// Path to events directory or single YAML file
        #[arg(short, long, value_name = "PATH")]
        events: PathBuf,

        /// Use predicate-based evaluation (faster for repeated evaluations)
        #[arg(short, long)]
        predicates: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logger
    let log_level = if cli.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();

    match cli.command {
        Commands::Compile { rules, output } => {
            compile_rules(&rules, &output)?;
        }
        Commands::Eval {
            rules,
            events,
            predicates,
        } => {
            eval_events(&rules, &events, predicates)?;
        }
    }

    Ok(())
}

/// Compile YAML rules to binary format
fn compile_rules(rules_path: &PathBuf, output_path: &PathBuf) -> Result<()> {
    log::info!("Compiling rules from: {}", rules_path.display());

    // Load rules from directory or file
    let rules = if rules_path.is_dir() {
        Rules::from_dir(rules_path)
            .map_err(|e| anyhow!("Failed to load rules from directory: {:?}", e))?
    } else if rules_path.is_file() {
        let content = std::fs::read_to_string(rules_path)
            .with_context(|| format!("Failed to read rule file: {}", rules_path.display()))?;
        let rule = Rule::from_yaml(&content)
            .map_err(|e| anyhow!("Failed to parse rule: {:?}", e))?;
        Rules::from(vec![rule])
    } else {
        return Err(anyhow!(
            "Rules path does not exist: {}",
            rules_path.display()
        ));
    };

    let rule_count = rules.iter().count();
    log::info!("Loaded {} rule(s)", rule_count);

    // Create RuleSet and serialize
    let rule_set = RuleSet::from(rules);

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
        }
    }

    let mut output_file = std::fs::File::create(output_path)
        .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;

    rule_set
        .serialize(&mut output_file)
        .map_err(|e| anyhow!("Failed to serialize ruleset: {:?}", e))?;

    log::info!(
        "Successfully compiled {} rule(s) to: {}",
        rule_count,
        output_path.display()
    );
    println!(
        "✓ Compiled {} rule(s) to {}",
        rule_count,
        output_path.display()
    );

    Ok(())
}

/// Evaluate events against compiled rules
fn eval_events(rules_path: &PathBuf, events_path: &PathBuf, use_predicates: bool) -> Result<()> {
    log::info!("Loading ruleset from: {}", rules_path.display());

    // Load compiled ruleset
    let mut rules_file = std::fs::File::open(rules_path)
        .with_context(|| format!("Failed to open ruleset file: {}", rules_path.display()))?;

    let rule_set = RuleSet::deserialize(&mut rules_file)
        .map_err(|e| anyhow!("Failed to deserialize ruleset: {:?}", e))?;

    let rule_count = rule_set.rules().iter().count();
    log::info!("Loaded {} rule(s)", rule_count);

    // Load events
    log::info!("Loading events from: {}", events_path.display());

    let events = Events::from_dir(events_path)
        .map_err(|e| anyhow!("Failed to load events: {:?}", e))?;

    let event_count = events.iter().count();
    log::info!("Loaded {} event(s)", event_count);

    // Create engine and evaluate
    let mut engine = RadeEngine::from_rules(rule_set.retain_rules());

    println!("Evaluating {} event(s) against {} rule(s)...", event_count, rule_count);

    let matches = if use_predicates {
        log::info!("Using predicate-based evaluation");
        engine.compile_rules();
        engine
            .eval_with_predicates(events)
            .map_err(|e| anyhow!("Evaluation failed: {:?}", e))?
    } else {
        log::info!("Using iterative evaluation");
        engine.eval_iterative(events)
    };

    // Display results
    println!("\n{}", matches);

    println!(
        "\n✓ Evaluation complete for {} event(s)",
        event_count
    );

    Ok(())
}
