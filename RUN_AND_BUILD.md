# Running examples and building binaries

This guide describes two ways to run Soplang example programs: **run with Soplang** (JIT) and **build a standalone binary** (AOT). Built binaries go in the **`barnaamij/`** folder at the project root (barnaamij = program(s) in Somali).

## Prerequisites

```bash
cargo build --release
```

You need the release binary: `./target/release/soplang`.

---

## 1. Run with Soplang (JIT)

Run a `.sop` file directly with the Soplang binary. The program is compiled and executed on the fly (Cranelift JIT).

### Single file

```bash
./target/release/soplang examples/01_hello.sop
```

### Examples that need stdin

`13_input.sop` uses `gelin()` (user input). Pipe input when running:

```bash
echo "YourName" | ./target/release/soplang examples/13_input.sop
```

### Run all examples (no stdin)

```bash
for f in examples/*.sop; do
  [ "$(basename "$f")" = "14_random.sop" ] && continue
  echo "=== $f ==="
  ./target/release/soplang "$f" 2>&1
  echo ""
done
```

### REPL and one-liner

With no arguments, Soplang starts the **interactive REPL** (history in `~/.soplang_history`). In the REPL you can:

- Type statements or a **single expression** (e.g. `1+2`); expressions are printed automatically.
- Use **multi-line** input: continue on the next line when you have unclosed `{`, `(`, `[`, or a trailing `\`.
- Commands: `/caawii` or `/help`, `/bixi` or `/exit`, `/akhrifayl <fayl>` or `/load <file>`, `/ast <weedh>`, `/hir <weedh>`.

```bash
./target/release/soplang
./target/release/soplang -i                    # run a file then open REPL
./target/release/soplang -c '1+2'              # run snippet; expressions print result (3)
./target/release/soplang -c 'qor("Salaan!")'   # run snippet
```

**CLI options:** `-q` / `--quiet` (no build message), `--no-color` (plain errors), `--strict` (static types).

---

## 2. Build a standalone binary (AOT)

Build a native executable from a `.sop` file. Just run `--build`; the binary is written to **`barnaamij/`** (created automatically). Use `-o <path>` to override.

AOT uses a fixed workspace under `target/soplang_aot_runner/`, so the **first** `--build` compiles Soplang and its dependencies there; **later** builds only recompile the small runner and are much faster.

### Build one example

```bash
./target/release/soplang --build examples/01_hello.sop
./barnaamij/01_hello
```

### Build with optimization level

```bash
./target/release/soplang --build examples/01_hello.sop --opt-level 3
```

`--opt-level` can be 0, 1, 2, or 3 (default is 2).

### Build all examples

```bash
for f in examples/*.sop; do
  echo "Building $(basename "$f" .sop) ..."
  ./target/release/soplang --build "$f" 2>&1 || true
done
```

Then run any of them:

```bash
./barnaamij/01_hello
./barnaamij/02_variables
./barnaamij/13_input   # will read from stdin when you run it
```

**Note:** `13_input` and `14_random` use stdin / randomness; run them manually (e.g. `echo "Name" | ./barnaamij/13_input`).

---

## Summary

| Goal | Command |
|------|--------|
| Run one file (JIT) | `./target/release/soplang examples/01_hello.sop` |
| Run with stdin | `echo "Input" \| ./target/release/soplang examples/13_input.sop` |
| Build one binary (default: barnaamij/) | `./target/release/soplang --build examples/01_hello.sop` |
| Run built binary | `./barnaamij/01_hello` |
| Build all to barnaamij/ | Loop over `examples/*.sop` with `--build` (output goes to barnaamij/ by default) |
| REPL | `./target/release/soplang` (no args); `/caawii` = help, `/bixi` = exit |
| One-liner (with result) | `./target/release/soplang -c '1+2'` → prints `3` |
| Quiet build | `--build ... -q` to skip "Waa la dhisay" message |
| No color | `--no-color` or set `NO_COLOR=1` |

Built executables go to **`barnaamij/`**; the folder is created automatically. Use `-o <path>` to write elsewhere.
