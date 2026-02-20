# Soplang Benchmarks

This directory contains benchmark programs and tooling for measuring Soplang’s **compiler** (Cranelift JIT) performance.

## Contents

| File / directory | Description |
|------------------|-------------|
| **\*.sop** | Benchmark programs (fibonacci, loops, strings, lists, objects). |
| **RESULTS.md** | Criterion results, pipeline breakdown, and how to run. |
| **run_benchmarks.sh** | Script to run benchmarks and regenerate RESULTS.md. |

## Quick start

From the **project root**:

```bash
make build
cargo bench
```

Or run benchmarks and regenerate the results file:

```bash
bash benchmarks/run_benchmarks.sh
```

Results are written to **RESULTS.md**.

---

## What the benchmark names mean

The Criterion output uses short IDs; here is what each one measures:

| Criterion ID | Meaning |
|--------------|---------|
| **fibonacci/fib_25_full** | Full run of the recursive Fibonacci benchmark with **n = 25**. So we call `fib(25)` once; the result is 75,025 (the 25th Fibonacci number). This triggers about **242,785** recursive calls and stresses **function-call and return** overhead. |
| **loops/loop_sum_100k** | A single tight loop that sums integers from 1 to **100,000** (100k iterations). Measures **loop + arithmetic** codegen and runtime. |
| **loops/nested_loops_200x200** | **Nested** loops: outer 200 iterations, inner 200, so **40,000** iterations total. Measures **loop dispatch and inner-loop** performance. |
| **strings/string_concat_1k** | **1,000** string concatenations in a loop (e.g. repeatedly `s = s + "x"`). Measures **string allocation and copy** cost. |
| **lists/list_ops_5k** | **5,000** list operations: push elements and index traversal. Measures **list allocation and indexing**. |
| **objects/object_create_2k** | Create **2,000** objects and access properties. Measures **object allocation and property access**. |
| **pipeline_stages/lex_only** | Only **lex** the fib program (tokenize). Measures front-end tokenizer speed. |
| **pipeline_stages/parse_only** | **Lex + parse** the fib program (no execution). Measures parser speed. |

So: **fib_25** = Fibonacci with argument 25; **100k** = 100,000 iterations; **1k** = 1,000; **5k** = 5,000; **2k** = 2,000; **200x200** = 200×200 nested iterations.

---

## Performance achievements

Soplang’s **compiler** (Cranelift JIT) compiles `.sop` to native code in-process. Current results (see RESULTS.md for up-to-date numbers) show:

- **Fibonacci fib(25)** — ~25 ms for ~242k function calls: **direct calls** from compiled code to compiled code with minimal ABI overhead.
- **Loops** — **loop_sum_100k** ~10 ms (100k iterations), **nested_loops_200x200** ~4 ms (40k iterations): efficient loop codegen and arithmetic.
- **Strings / lists / objects** — **string_concat_1k** &lt;1 ms, **list_ops_5k** ~1.3 ms, **object_create_2k** ~0.25 ms: fast built-in types and runtime helpers.
- **Pipeline** — **lex_only** and **parse_only** in **microseconds**: front-end (lex + parse) is a tiny fraction of total time; almost all time is in **compile + execution**.

So the runtime is **compiler-driven**: we JIT-compile to native code and run it, rather than interpreting the AST. The benchmarks show that function calls, loops, and built-in operations all run at compiled speed.

---

## Benchmark programs (files)

| Program | What it measures |
|---------|------------------|
| `fib_recursive.sop` | Recursive function calls — `fib(25)` → 75,025. |
| `loop_sum.sop` | Tight for-loop — sum 1..100,000. |
| `nested_loops.sop` | Nested loop dispatch — 200×200 iterations. |
| `string_concat.sop` | String concatenation — 1,000 appends. |
| `list_ops.sop` | List push + index traversal — 5,000 elements. |
| `object_create.sop` | Object allocation + property access — 2,000 objects. |

---

## See also

- **[RESULTS.md](RESULTS.md)** — Criterion timings (Mean / Low / High), pipeline breakdown, and how to run.
