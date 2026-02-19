# Soplang Interpreter Benchmarks

## Benchmark Programs

| Benchmark | Description | Workload |
|-----------|-------------|----------|
| `fib_recursive` | Recursive Fibonacci | `fib(25)` — 242,785 function calls |
| `loop_sum` | Tight for-loop with arithmetic | Sum of 1..100,000 |
| `nested_loops` | Nested loop dispatch | 200 x 200 = 40,804 iterations |
| `string_concat` | String concatenation in loop | 1,000 appends |
| `list_ops` | List push + index traversal | 5,000 elements |
| `object_create` | Object allocation + property access | 2,000 objects |

## Criterion Results (in-process, statistical)

| Benchmark | Mean | Low | High |
|-----------|------|-----|------|
| **fibonacci/fib_25_full** | **282.15 ms** | 269.72 ms | 297.19 ms |
| **loops/loop_sum_100k** | **25.10 ms** | 23.41 ms | 27.22 ms |
| **loops/nested_loops_200x200** | **8.54 ms** | 8.35 ms | 8.84 ms |
| **strings/string_concat_1k** | **616.93 us** | 601.98 us | 632.76 us |
| **lists/list_ops_5k** | **2.48 ms** | 2.44 ms | 2.55 ms |
| **objects/object_create_2k** | **1.79 ms** | 1.61 ms | 2.00 ms |

### Pipeline Stage Breakdown

| Stage | Mean | Notes |
|-------|------|-------|
| Lex only (fib program) | **3.76 us** | Tokenization is near-instant |
| Parse only (fib program) | **5.98 us** | Parsing is near-instant |
| Full execution (fib_25) | **282.15 ms** | ~99.99% time is in interpretation |

> The lexer and parser are extremely fast. Virtually all time is spent in the
> tree-walking interpreter executing the AST, which is expected for this
> architecture.

## How to Run

```bash
# Full criterion benchmarks (with HTML reports)
cargo bench

# Run a specific benchmark group
cargo bench -- fibonacci
cargo bench -- loops
cargo bench -- strings
cargo bench -- pipeline_stages
cargo bench -- comparison

# Quick wall-clock timing of a single benchmark file
time ./target/release/soplang benchmarks/fib_recursive.sop

# Auto-generate this file with fresh results
bash benchmarks/run_benchmarks.sh

# Rust vs Python (Hyperfine) — full process comparison
# Requires: cargo install hyperfine, pip install -e ., make build
bash benchmarks/compare_rust_vs_python.sh
```

## Rust vs Python (Hyperfine)

[Hyperfine](https://github.com/sharkdp/hyperfine) compares **full process** runtimes (Rust binary vs `python -m psrc`) on the same `.sop` benchmarks. This measures real-world “run this script once” performance including startup.

**Run the comparison:**

```bash
cargo install hyperfine   # one-time
make build               # ensure release binary exists
# Python: use psrc/ from repo. Either pip install -e . or:
#   python3 -m venv .venv && .venv/bin/pip install -r psrc/requirements.txt
#   PYTHON=.venv/bin/python bash benchmarks/compare_rust_vs_python.sh
bash benchmarks/compare_rust_vs_python.sh
```

The script sets `PYTHONPATH` to the project root so `python -m psrc` loads the `psrc/` package without installing it. If your system Python can’t import psrc (e.g. missing `colorama`), use a venv as above.

**Sample results (Rust vs Python, full process):**

| Benchmark       | Rust (mean) | Python (mean) | Speedup (Rust vs Python) |
|----------------|-------------|---------------|---------------------------|
| fib_recursive  | ~314 ms     | ~1.71 s       | **~5.5×**                 |
| loop_sum       | ~23 ms      | ~480 ms       | **~21×**                  |
| nested_loops   | ~11 ms      | ~325 ms       | **~29×**                  |
| string_concat  | ~1.1 ms     | ~191 ms       | **~170×**                 |
| list_ops       | ~3.4 ms     | ~234 ms       | **~69×**                  |
| object_create  | ~2.2 ms     | ~212 ms       | **~97×**                  |

A combined report with summary and detailed tables is in **benchmarks/HYPERFINE.md**. Results are machine-dependent; use them for relative speedup.

## Interpreting Results

- **Criterion** runs each benchmark many times, computing mean, median, standard
  deviation, and confidence intervals. It also detects regressions/improvements
  between consecutive runs.
- **HTML reports** are generated in `target/criterion/` — open
  `target/criterion/report/index.html` in a browser for violin plots, PDFs, and
  regression analysis.
- The `comparison` benchmark group runs all programs side by side under identical
  criterion settings for fair cross-benchmark comparison.
- These are **in-process** measurements (no process startup overhead), so they are
  more accurate than wall-clock `time` measurements for comparing interpreter
  performance across changes.

## Architecture Notes

The interpreter is a **tree-walking interpreter** — it traverses the AST directly
without compiling to bytecode. This means:

- **Function calls are expensive**: each call creates a new scope, clones the
  environment, and recursively walks the body AST. This is why `fib(25)` (~242k
  calls) takes ~282ms.
- **Loops are moderate**: each iteration dispatches through the AST matcher, which
  adds overhead compared to a bytecode loop.
- **String concat is O(n^2)**: each append creates a new string. A rope or buffer
  strategy would improve this.
- **Object/list allocation** is relatively fast thanks to Rust's efficient HashMap
  and Vec implementations under the hood.

Future optimizations (bytecode compilation, constant folding, tail-call optimization)
would show up clearly in these benchmarks.
