# Soplang

> The Somali Programming Language

Soplang is a programming language with syntax inspired by Somali, making programming more accessible to Somali speakers. It combines static and dynamic typing in one language with a focus on clarity and ease of use.

**The primary implementation is in Rust.** The Python→Rust migration is complete: you get a single native binary (`soplang`) that runs `.sop` files and provides an interactive REPL. We are now **building the compiler**: Cranelift JIT (for fast run) and LLVM AOT (for standalone binaries). See [COMPILER_PLAN.md](COMPILER_PLAN.md).

## Project structure

| Path | Description |
|------|-------------|
| **src/** | Rust implementation: lexer, parser, **interpreter** (current runtime), stdlib. **Compiler** (semantic, HIR, Cranelift, LLVM) in progress per [COMPILER_PLAN.md](COMPILER_PLAN.md). |
| **examples/** | Soplang example programs (`.sop` files). |
| **benchmarks/** | Benchmark programs and [RESULTS.md](benchmarks/RESULTS.md). |
| **psrc/** | Python reference implementation (legacy). See [psrc/README.md](psrc/README.md). |
| **csrc/** | C reference implementation. |
| **docs/** | Documentation (installation, language reference, contributing). |
| **windows/**, **linux/**, **macos/** | Platform-specific build and packaging scripts. |

## Features

- **Dual type system** — Static typing (`abn`, `qoraal`, etc.) and dynamic typing (`door`)
- **Somali-based syntax** — Keywords and concepts in Somali
- **Modern paradigms** — Functional, procedural, and object-oriented support
- **Interactive shell** — REPL for experimentation
- **Compiled language (in progress)** — Cranelift JIT + LLVM AOT per [COMPILER_PLAN.md](COMPILER_PLAN.md)

## Example

```sop
qor("Salaan, Adduunka!")   // Hello, World!

door magac = "Sharafdin"
abn age = 25

hawl salaam(qof) {
    celi "Salaan, " + qof + "!"
}
qor(salaam(magac))
```

## Running Soplang

**Current runtime:** tree-walking interpreter (Rust). Build and run:

```bash
cargo build --release

./target/release/soplang examples/hello.sop
./target/release/soplang -i    # REPL
./target/release/soplang -c 'qor("Salaan!")'
```

Or: `make build`, `make run FILE=examples/hello.sop`, `make shell`, `make test-rust`, `make bench`.

**Planned (compiler):** `soplang file.sop` will use Cranelift JIT; `soplang build file.sop` will produce a standalone binary via LLVM. See [COMPILER_PLAN.md](COMPILER_PLAN.md).

## Benchmark results

The Rust interpreter is benchmarked with [Criterion](https://github.com/bheisler/criterion.rs). Summary (release build):

| Benchmark | Mean time |
|-----------|-----------|
| fib(25) recursive | ~282 ms |
| loop sum 1..100k | ~25 ms |
| nested loops 200×200 | ~8.5 ms |
| string concat 1k | ~617 µs |
| list ops 5k elements | ~2.5 ms |
| object create 2k | ~1.8 ms |

Details and Rust vs Python comparison: [benchmarks/RESULTS.md](benchmarks/RESULTS.md) and [benchmarks/HYPERFINE.md](benchmarks/HYPERFINE.md).

## Legacy — psrc (Python)

The Python implementation in `psrc/` is kept for reference:

```bash
pip install -e .
python -m psrc examples/hello.sop
```

See [psrc/README.md](psrc/README.md).

## Documentation

- [Getting started](docs/index.md)
- [Language reference](docs/language/keywords.md)
- [Installation](docs/installation.md)
- [Contributing](docs/CONTRIBUTING.md)
- [Rust implementation plan (complete)](IMPLEMENTATION_PLAN.md) — Phases 1–7 done
- [Compiler plan (in progress)](COMPILER_PLAN.md) — Cranelift + LLVM

## License

This project is licensed under the [MIT License](LICENSE).
