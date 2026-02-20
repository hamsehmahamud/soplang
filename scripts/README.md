# Soplang Scripts

Scripts have been cleaned up. The important ones live at the project root or in `benchmarks/`.

## Build and run

- **Root:** `./build.sh` — runs `cargo build --release`
- **Root:** `./soplang-docker.sh` — build Docker image (if needed) and run Soplang in Docker (shell or a file)
- **Benchmarks:** `benchmarks/run_benchmarks.sh` — run `cargo bench` and regenerate `benchmarks/RESULTS.md`

## Tests

Use the Makefile or Cargo directly:

```bash
make test
# or
cargo test
```

Example programs are tested automatically via `cargo test` (examples vs `.expected` files).

## Recreating scripts later

When you need them again, the most useful to add back are:

1. **Example runner** — run all `examples/*.sop` with `./target/release/soplang` (optionally non-interactive)
2. **Platform packaging** — macOS/Linux/Windows scripts that build the Rust binary and produce installers (e.g. .dmg, .deb, .exe) when you’re ready to ship
