use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use soplang::{Interpreter, Lexer, Parser};
use std::path::Path;
use std::time::Duration;

fn run_file(path: &str) {
    let source = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {path}: {e}"));
    let tokens = Lexer::new(&source).tokenize().expect("lex error");
    let stmts = Parser::new(tokens).parse().expect("parse error");
    let mut interp = Interpreter::new();
    interp
        .run_with_path(stmts, Some(Path::new(path)))
        .expect("runtime error");
}

fn lex_only(source: &str) {
    Lexer::new(source).tokenize().expect("lex error");
}

fn parse_only(source: &str) {
    let tokens = Lexer::new(source).tokenize().expect("lex error");
    Parser::new(tokens).parse().expect("parse error");
}

// ---------- Individual benchmark groups ----------

fn bench_fibonacci(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(20);

    group.bench_function("fib_25_full", |b| {
        b.iter(|| run_file("benchmarks/fib_recursive.sop"))
    });
    group.finish();
}

fn bench_loops(c: &mut Criterion) {
    let mut group = c.benchmark_group("loops");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(30);

    group.bench_function("loop_sum_100k", |b| {
        b.iter(|| run_file("benchmarks/loop_sum.sop"))
    });
    group.bench_function("nested_loops_200x200", |b| {
        b.iter(|| run_file("benchmarks/nested_loops.sop"))
    });
    group.finish();
}

fn bench_strings(c: &mut Criterion) {
    let mut group = c.benchmark_group("strings");
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(30);

    group.bench_function("string_concat_1k", |b| {
        b.iter(|| run_file("benchmarks/string_concat.sop"))
    });
    group.finish();
}

fn bench_lists(c: &mut Criterion) {
    let mut group = c.benchmark_group("lists");
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(30);

    group.bench_function("list_ops_5k", |b| {
        b.iter(|| run_file("benchmarks/list_ops.sop"))
    });
    group.finish();
}

fn bench_objects(c: &mut Criterion) {
    let mut group = c.benchmark_group("objects");
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(30);

    group.bench_function("object_create_2k", |b| {
        b.iter(|| run_file("benchmarks/object_create.sop"))
    });
    group.finish();
}

fn bench_pipeline(c: &mut Criterion) {
    let source = std::fs::read_to_string("benchmarks/fib_recursive.sop").unwrap();
    let mut group = c.benchmark_group("pipeline_stages");
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));

    group.bench_function("lex_only", |b| b.iter(|| lex_only(&source)));
    group.bench_function("parse_only", |b| b.iter(|| parse_only(&source)));

    group.finish();
}

fn bench_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison");
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(20);

    let benchmarks = [
        ("fib_recursive", "benchmarks/fib_recursive.sop"),
        ("loop_sum", "benchmarks/loop_sum.sop"),
        ("nested_loops", "benchmarks/nested_loops.sop"),
        ("string_concat", "benchmarks/string_concat.sop"),
        ("list_ops", "benchmarks/list_ops.sop"),
        ("object_create", "benchmarks/object_create.sop"),
    ];

    for (name, path) in &benchmarks {
        group.bench_with_input(
            BenchmarkId::new("soplang", name),
            path,
            |b, path| b.iter(|| run_file(path)),
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_fibonacci,
    bench_loops,
    bench_strings,
    bench_lists,
    bench_objects,
    bench_pipeline,
    bench_comparison,
);

criterion_main!(benches);
