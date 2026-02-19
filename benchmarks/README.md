# Soplang Benchmarks

This directory contains benchmark programs and tooling for the Soplang interpreter (Rust and Python/psrc).

## Contents

| File / directory | Description |
|------------------|-------------|
| **\*.sop** | Benchmark programs (fibonacci, loops, strings, lists, objects). |
| **RESULTS.md** | Criterion (Rust-only) results, pipeline breakdown, and how to run. |
| **HYPERFINE.md** | Rust vs Python full-process comparison (one combined report). |
| **compare_rust_vs_python.sh** | Script that runs [Hyperfine](https://github.com/sharkdp/hyperfine) on Rust and Python for each benchmark; writes **HYPERFINE.md**. |
| **run_benchmarks.sh** | Optional script to regenerate RESULTS.md from `cargo bench`. |

## Quick start

From the **project root**:

```bash
# Rust-only (Criterion, in-process)
make build
cargo bench

# Rust vs Python (Hyperfine, full process)
cargo install hyperfine   # one-time
# Python: pip install -e .  OR  .venv with psrc/requirements.txt + PYTHON=.venv/bin/python
bash benchmarks/compare_rust_vs_python.sh
```

Results: **RESULTS.md** (Criterion), **HYPERFINE.md** (Rust vs Python).

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
- **[HYPERFINE.md](HYPERFINE.md)** — Rust vs Python comparison (full process, one file).
