# Soplang Python implementation (psrc)

This directory contains the **entire Python implementation** of Soplang: interpreter, standard library, tests, and Python-specific tooling. The root project is being rebuilt in Rust; this folder is the reference Python version and is self-contained.

## Layout

```
psrc/
├── README.md           # This file
├── __init__.py         # Package root; re-exports main types
├── __main__.py         # CLI entry point (python -m psrc)
├── core/               # Lexer, parser, tokens, AST, version
├── runtime/            # Interpreter, shell, main module
├── stdlib/             # Builtins (qor, gelin, nooc, list/object methods, etc.)
├── utils/              # Error types and Somali error messages
├── tests/              # Unit tests (lexer, parser, interpreter)
├── tests/runners/      # Test runner scripts
├── run_tests.py        # Run all tests
├── check_examples.py   # Run each .sop example under ../examples
├── requirements.txt    # Runtime dependencies
├── requirements-dev.txt # Dev dependencies (pytest, flake8, etc.)
├── pytest.ini          # Pytest config
├── soplang.spec        # PyInstaller spec (build from repo root)
├── .flake8             # Flake8 lint config
└── .pylintrc           # Pylint config
```

## Requirements

- Python 3.6+
- **Runtime:** `colorama`, `prompt_toolkit` (see `requirements.txt`)
- **Dev:** pytest, flake8, black, etc. (see `requirements-dev.txt`)

## Installation

From the **project root** (parent of `psrc/`):

```bash
pip install -e .
```

This installs the `soplang` command and the `psrc` package. Requirements are read from `psrc/requirements.txt`.

## Running the interpreter

From the **project root** (so that `examples/` and imports resolve):

```bash
# Interactive shell
python -m psrc

# Run a file
python -m psrc path/to/file.sop

# Run an example by number
python -m psrc -e 1

# One-off code
python -m psrc -c 'qor("Hello")'

# Version
python -m psrc -v
```

Or use the root stub:

```bash
python main.py
python main.py examples/hello.sop
```

## Testing

From the **project root**:

```bash
# Run all Python tests (unittest)
python psrc/run_tests.py

# Or call the runner directly
python psrc/tests/runners/run_all_tests.py

# Pytest (with project root on PYTHONPATH)
PYTHONPATH=. pytest psrc/tests -v

# Check that all example files run without unexpected errors
python psrc/check_examples.py
```

## Building a standalone executable (PyInstaller)

From the **project root**:

```bash
pip install pyinstaller
pyinstaller psrc/soplang.spec
```

The spec resolves paths relative to the project root; the resulting executable is produced in `dist/`.

## Linting and formatting

From the **project root**, with `psrc` on the path:

```bash
cd psrc
flake8 . --config .flake8
pylint core runtime stdlib utils
black .
```

## Relation to the rest of the project

- **Root** holds general files: docs, examples, platform scripts, `main.py` stub, `setup.py`, and (when added) the Rust crate.
- **csrc/** is the C implementation (separate from Python).
- **psrc/** is this Python implementation. All Python code for Soplang lives here so the root can stay language-agnostic and the Rust migration can proceed independently.
