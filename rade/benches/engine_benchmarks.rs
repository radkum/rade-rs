//! Benchmarks for the Rade Engine
//!
//! Run with: `cargo bench -p rade`

use std::path::PathBuf;

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use rade::{Events, RadeEngine, Rules};

fn get_test_data_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir).join("test_data")
}

fn load_test_rules() -> Rules {
    let rules_path = get_test_data_path().join("rules");
    Rules::from_dir(&rules_path).expect("Failed to load rules")
}

fn load_test_events() -> Events {
    let events_path = get_test_data_path().join("events");
    Events::from_dir(&events_path).expect("Failed to load events")
}

/// Benchmark loading rules from YAML files
fn bench_load_rules(c: &mut Criterion) {
    let rules_path = get_test_data_path().join("rules");

    c.bench_function("load_rules_from_yaml", |b| {
        b.iter(|| {
            let rules = Rules::from_dir(black_box(&rules_path)).unwrap();
            black_box(rules)
        })
    });
}

/// Benchmark loading events from YAML files
fn bench_load_events(c: &mut Criterion) {
    let events_path = get_test_data_path().join("events");

    c.bench_function("load_events_from_yaml", |b| {
        b.iter(|| {
            let events = Events::from_dir(black_box(&events_path)).unwrap();
            black_box(events)
        })
    });
}

/// Benchmark iterative evaluation (no predicate compilation)
fn bench_eval_iterative(c: &mut Criterion) {
    let rules = load_test_rules();
    let events = load_test_events();

    let mut group = c.benchmark_group("eval_iterative");

    group.throughput(Throughput::Elements(1));

    group.bench_function("all_events", |b| {
        b.iter(|| {
            let mut engine = RadeEngine::from_rules(rules.clone());
            let matches = engine.eval_iterative(black_box(events.clone()));
            black_box(matches)
        })
    });

    group.finish();
}

/// Benchmark evaluation with predicate compilation
fn bench_eval_with_predicates(c: &mut Criterion) {
    let rules = load_test_rules();
    let events = load_test_events();

    let mut group = c.benchmark_group("eval_with_predicates");

    group.throughput(Throughput::Elements(1));

    // Benchmark compilation + evaluation
    group.bench_function("compile_and_eval", |b| {
        b.iter(|| {
            let mut engine = RadeEngine::from_rules(rules.clone());
            engine.compile_rules();
            let matches = engine.eval_with_predicates(black_box(events.clone())).unwrap();
            black_box(matches)
        })
    });

    // Benchmark just evaluation (pre-compiled)
    group.bench_function("eval_only_precompiled", |b| {
        let mut engine = RadeEngine::from_rules(rules.clone());
        engine.compile_rules();

        b.iter(|| {
            let matches = engine.eval_with_predicates(black_box(events.clone())).unwrap();
            black_box(matches)
        })
    });

    group.finish();
}

/// Benchmark predicate compilation
fn bench_compile_rules(c: &mut Criterion) {
    let rules = load_test_rules();

    c.bench_function("compile_predicates", |b| {
        b.iter(|| {
            let mut engine = RadeEngine::from_rules(black_box(rules.clone()));
            engine.compile_rules();
            black_box(engine)
        })
    });
}

/// Benchmark engine creation
fn bench_engine_creation(c: &mut Criterion) {
    let rules = load_test_rules();

    c.bench_function("engine_from_rules", |b| {
        b.iter(|| {
            let engine = RadeEngine::from_rules(black_box(rules.clone()));
            black_box(engine)
        })
    });
}

/// Benchmark with varying number of events
fn bench_scaling_events(c: &mut Criterion) {
    let rules = load_test_rules();
    let base_events = load_test_events();

    let mut group = c.benchmark_group("scaling_events");

    // Test with 1x, 2x, 4x, 8x events
    for multiplier in [1, 2, 4, 8] {
        let mut events_vec = Vec::new();
        for _ in 0..multiplier {
            for event in base_events.iter() {
                events_vec.push(event.clone());
            }
        }
        let scaled_events = Events::new(events_vec);
        let event_count = base_events.iter().count() * multiplier;

        group.throughput(Throughput::Elements(event_count as u64));

        // Iterative evaluation
        group.bench_with_input(
            BenchmarkId::new("iterative", event_count),
            &scaled_events,
            |b, events| {
                b.iter(|| {
                    let mut engine = RadeEngine::from_rules(rules.clone());
                    let matches = engine.eval_iterative(black_box(events.clone()));
                    black_box(matches)
                })
            },
        );

        // Predicate-based evaluation
        group.bench_with_input(
            BenchmarkId::new("predicates", event_count),
            &scaled_events,
            |b, events| {
                let mut engine = RadeEngine::from_rules(rules.clone());
                engine.compile_rules();

                b.iter(|| {
                    let matches = engine.eval_with_predicates(black_box(events.clone())).unwrap();
                    black_box(matches)
                })
            },
        );
    }

    group.finish();
}

/// Compare iterative vs predicate-based evaluation
fn bench_comparison(c: &mut Criterion) {
    let rules = load_test_rules();
    let events = load_test_events();

    let mut group = c.benchmark_group("comparison");

    // Iterative
    group.bench_function("iterative", |b| {
        b.iter(|| {
            let mut engine = RadeEngine::from_rules(rules.clone());
            let matches = engine.eval_iterative(black_box(events.clone()));
            black_box(matches)
        })
    });

    // Predicate (compile each time)
    group.bench_function("predicate_with_compile", |b| {
        b.iter(|| {
            let mut engine = RadeEngine::from_rules(rules.clone());
            engine.compile_rules();
            let matches = engine.eval_with_predicates(black_box(events.clone())).unwrap();
            black_box(matches)
        })
    });

    // Predicate (pre-compiled)
    let mut precompiled_engine = RadeEngine::from_rules(rules.clone());
    precompiled_engine.compile_rules();

    group.bench_function("predicate_precompiled", |b| {
        b.iter(|| {
            let matches = precompiled_engine
                .eval_with_predicates(black_box(events.clone()))
                .unwrap();
            black_box(matches)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_load_rules,
    bench_load_events,
    bench_engine_creation,
    bench_compile_rules,
    bench_eval_iterative,
    bench_eval_with_predicates,
    bench_scaling_events,
    bench_comparison,
);

criterion_main!(benches);
