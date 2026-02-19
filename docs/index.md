# Soplang Documentation

Soplang is the Somali programming language, designed to make programming accessible to Somali speakers worldwide.

**Implementation:** Rust (at project root). The **compiler** (Cranelift JIT + LLVM AOT) is in progress; see [COMPILER_PLAN.md](../COMPILER_PLAN.md) in the repo root.

## Documentation Sections

### Getting Started
- [Installation Guide](installation.md) - Detailed instructions for installing Soplang on Windows, Linux, and macOS

### Language Reference
- [Keywords and Grammar](language/keywords.md) - Complete reference of Soplang keywords and language structure
- [Expressions](language/expressions.md) - Detailed explanation of expressions and operator usage
- [Grammar Specification](language/grammar.md) - Formal grammar specification in EBNF

### Examples
- [Examples Guide](examples/EXAMPLES.md) - Guide to the example programs
- [Test Examples](examples/TEST_EXAMPLES_README.md) - Documentation for test examples

### Building and Performance
- [Build Guide](build/BUILD.md) - How to build Soplang from source
- [Performance](build/PERFORMANCE.md) - Performance benchmarks and optimization techniques
- [C Implementation](build/README_C.md) - Information about the C implementation

### Architecture & Internals
- [Compiler Plan](../COMPILER_PLAN.md) - Cranelift JIT + LLVM AOT (Rust, in progress)

### Testing
- [Testing Guide](testing/TESTING.md) - How to test Soplang
- [Test README](testing/README-TESTS.md) - Additional test documentation

## Getting Started

### Installation

To install Soplang, see the [Installation Guide](installation.md) which covers all platforms:
- Windows installation (using installer or building from source)
- Linux installation (using package manager or building from source)
- macOS installation (using DMG or building from source)

### Running Soplang

The primary way to run Soplang is the **Rust binary** (from project root):

```bash
# Build
cargo build --release

# Run a Soplang program
./target/release/soplang examples/hello.sop

# Start the interactive shell (REPL)
./target/release/soplang -i
```

Or use the Makefile: `make build`, `make run FILE=examples/hello.sop`, `make shell`.

## Further Resources

- Website: [https://www.soplang.org/](https://www.soplang.org/)
- GitHub Repository: [https://github.com/sharafdin/soplang](https://github.com/sharafdin/soplang)
