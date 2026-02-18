# Soplang

> The Somali Programming Language

Soplang is a programming language with syntax inspired by Somali, making programming more accessible to Somali speakers. It combines static and dynamic typing in one language with a focus on clarity and ease of use.

**The project is being rebuilt in Rust.** The repository layout separates the future Rust implementation from reference implementations and general project files.

## Project structure

| Path | Description |
|------|-------------|
| **Rust (coming)** | Primary implementation. The codebase is being migrated to Rust for performance and a single native binary. |
| **csrc/** | C reference implementation (lexer, parser, runtime, stdlib). |
| **psrc/** | Python reference implementation. All Python code (interpreter, tests, tooling) lives here. See [psrc/README.md](psrc/README.md). |
| **examples/** | Soplang example programs (`.sop` files). |
| **docs/** | Documentation (installation, language reference, contributing). |
| **windows/**, **linux/**, **macos/** | Platform-specific build and packaging scripts. |
| **scripts/** | Build, test, and benchmark scripts. |

Root-level files are general project files: `LICENSE`, `CHANGELOG.md`, `Makefile`, `build.sh`, `grammar.ebnf`, `main.py` (thin stub that runs the Python implementation), and `setup.py` (installs the Python package from `psrc/`).

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

## Running Soplang today

Until the Rust implementation is ready, you can run the **Python implementation** from the `psrc/` directory. From the project root:

```bash
# Install the Python package (installs psrc)
pip install -e .

# Run interactive shell
python -m psrc

# Run a file
python -m psrc examples/hello.sop

# Or use the stub at root (same as above)
python main.py examples/hello.sop
```

See **[psrc/README.md](psrc/README.md)** for Python-specific setup, testing, and building.

## Rust (planned)

The main interpreter will be reimplemented in Rust. The root of the repository will then provide:

- `cargo build` / `cargo run` for the primary Soplang binary
- `csrc/` and `psrc/` kept as reference or fallback implementations

## Documentation

- [Getting started](docs/index.md)
- [Language reference](docs/language/keywords.md)
- [Installation](docs/installation.md)
- [Contributing](docs/CONTRIBUTING.md)

## License

This project is licensed under the [MIT License](LICENSE).
