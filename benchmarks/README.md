# Soplang Benchmarks

This directory contains benchmark programs and tooling for Soplang.

## Contents

| File / directory | Description |
|------------------|-------------|
| **\*.sop** | Benchmark programs (fibonacci, loops, strings, lists, objects). |
| **RESULTS.md** | Criterion results, pipeline breakdown, and how to run. |
| **run_benchmarks.sh** | Optional script to regenerate RESULTS.md from `cargo bench`. |

## Quick start

From the **project root**:

```bash
make build
cargo bench
```

Results: **RESULTS.md**.

## Benchmark programs

| Program | What it measures |
|---------|------------------|
| `fib_recursive.sop` | Recursive function calls — `fib(25)`. |
| `loop_sum.sop` | Tight for-loop + arithmetic — sum 1..100,000. |
| `nested_loops.sop` | Nested loop dispatch — 200×200 iterations. |
| `string_concat.sop` | String concatenation — 1,000 appends. |
| `list_ops.sop` | List push + index traversal — 5,000 elements. |
| `object_create.sop` | Object allocation + property access — 2,000 objects. |

## See also

- **[RESULTS.md](RESULTS.md)** — Criterion timings, pipeline (lex/parse/execute), and architecture notes.
