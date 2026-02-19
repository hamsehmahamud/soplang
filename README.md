# Soplang

> The Somali Programming Language

Soplang is a programming language with syntax inspired by Somali, making programming more accessible to Somali speakers. It combines static and dynamic typing in one language with a focus on clarity and ease of use.

The **primary implementation is in Rust** (tree-walking interpreter). The repo also includes reference implementations in Python and C.

## Project structure

| Path | Description |
|------|-------------|
| **src/** | Rust implementation (lexer, parser, interpreter, stdlib). Build with `cargo build --release`. |
| **examples/** | Soplang example programs (`.sop` files). |
| **benchmarks/** | Benchmark programs and [RESULTS.md](benchmarks/RESULTS.md) with timing data. |
| **psrc/** | Python reference implementation. See [psrc/README.md](psrc/README.md). |
| **csrc/** | C reference implementation. |
| **docs/** | Documentation (installation, language reference, contributing). |
| **windows/**, **linux/**, **macos/** | Platform-specific build and packaging scripts. |
| **scripts/** | Build, test, and benchmark scripts. |

## Features

- **Dual type system** — Static typing (`abn`, `qoraal`, etc.) and dynamic typing (`door`) in one language
- **Somali-based syntax** — Keywords and concepts in Somali
- **Modern paradigms** — Functional, procedural, and object-oriented support
- **Interactive shell** — REPL for experimentation
- **Cross-platform** — Installers and guides for Windows, Linux, and macOS

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

From the project root, build and run the **Rust implementation**:

```bash
# Build release binary
cargo build --release

# Run a .sop file
./target/release/soplang examples/hello.sop

# Interactive REPL
./target/release/soplang -i

# One-liner
./target/release/soplang -c 'qor("Salaan!")'
```

Or use the Makefile: `make build`, `make run FILE=examples/hello.sop`, `make shell`, `make test-rust`, `make bench`.

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

Full details, pipeline breakdown (lex/parse vs execution), and how to run: **[benchmarks/RESULTS.md](benchmarks/RESULTS.md)**.

## Legacy / reference — psrc (Python)

The **Python implementation** in `psrc/` remains available for reference and comparison:

```bash
pip install -e .
python -m psrc examples/hello.sop
python -m psrc   # interactive shell
```

See **[psrc/README.md](psrc/README.md)** for Python setup and tests.

## Documentation

- [Getting started](docs/index.md)
- [Language reference](docs/language/keywords.md)
- [Installation](docs/installation.md)
- [Contributing](docs/CONTRIBUTING.md)

## License

This project is licensed under the [MIT License](LICENSE).
