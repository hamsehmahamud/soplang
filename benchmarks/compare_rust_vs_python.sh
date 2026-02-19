#!/usr/bin/env bash
# Compare Rust vs Python (psrc/) using Hyperfine.
# Requires: hyperfine (cargo install hyperfine), release binary, Python 3 with psrc/ in repo.
# Uses PYTHONPATH so psrc is loaded from ./psrc/ (no pip install needed).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
PYTHON="${PYTHON:-python3}"
RUST_BIN="$PROJECT_DIR/target/release/soplang"
export PYTHONPATH="$PROJECT_DIR"

cd "$PROJECT_DIR"

if ! command -v hyperfine &>/dev/null; then
    echo "hyperfine not found. Install with: cargo install hyperfine"
    exit 1
fi

if [[ ! -x "$RUST_BIN" ]]; then
    echo "Release binary not found. Run: make build  # or cargo build --release"
    exit 1
fi

if ! "$PYTHON" -c "import psrc" 2>/dev/null; then
    echo "Python package psrc not found. PYTHONPATH=$PROJECT_DIR (psrc/ must be here)."
    exit 1
fi

BENCHMARKS=(
    "fib_recursive.sop"
    "loop_sum.sop"
    "nested_loops.sop"
    "string_concat.sop"
    "list_ops.sop"
    "object_create.sop"
)

echo "=== Rust vs Python (hyperfine) ==="
echo "Rust:  $RUST_BIN"
echo "Python: $PYTHON -m psrc"
echo ""

for name in "${BENCHMARKS[@]}"; do
    path="$SCRIPT_DIR/$name"
    [[ -f "$path" ]] || continue
    echo "--- $name ---"
    hyperfine \
        --warmup 2 \
        "$RUST_BIN $path" \
        "$PYTHON -m psrc $path" \
        2>&1 || true
    echo ""
done

echo "Done. Summary and detailed tables: benchmarks/HYPERFINE.md (update from this output if needed)."
