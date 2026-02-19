# Soplang

> The Somali Programming Language

Soplang is a programming language with syntax inspired by Somali, making programming more accessible to Somali speakers. It combines static and dynamic typing in one language with a focus on clarity and ease of use.

**The primary implementation is in Rust.** The Python→Rust migration is complete. Soplang now runs through a compiled pipeline by default: Cranelift JIT for `run` and an AOT build path for standalone binaries. See [COMPILER_PLAN.md](COMPILER_PLAN.md).

## Project structure

| Path | Description |
|------|-------------|
| **src/** | Rust implementation: lexer, parser, semantic, HIR, runtime, Cranelift JIT, and AOT build backend. |
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
- **Interactive shell** — REPL powered by the compiled JIT pipeline
- **Compiled execution** — Cranelift JIT for running files, AOT build for standalone binaries

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

Build and run:

```bash
cargo build --release

./target/release/soplang examples/hello.sop        # JIT run
./target/release/soplang -i    # REPL
./target/release/soplang -c 'qor("Salaan!")'
./target/release/soplang --build examples/hello.sop -o hello_aot
./hello_aot
```

Or: `make build`, `make run FILE=examples/hello.sop`, `make shell`, `make test-rust`, `make bench`.

### CLI notes

- `soplang <file.sop>`: run via Cranelift JIT.
- `soplang --build <file.sop> -o <out>`: build a standalone native binary.
- `soplang --build ... --opt-level 0..3`: tune AOT optimization level.
- `soplang --strict`: enable stricter static typing checks.

### AOT backend note

Current AOT backend is implemented by generating a temporary Rust runner and compiling it to a native executable. This is the supported strategy for now; it can be replaced later with a direct LLVM IR (`inkwell`) backend.

## Benchmark results

Runtime performance is benchmarked with [Criterion](https://github.com/bheisler/criterion.rs). Summary (release build):

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
