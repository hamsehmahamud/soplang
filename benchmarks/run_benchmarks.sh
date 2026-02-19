#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
RESULTS_FILE="$SCRIPT_DIR/RESULTS.md"

cd "$PROJECT_DIR"

echo "Building release binary..."
cargo build --release 2>&1

echo ""
echo "Running criterion benchmarks..."
BENCH_OUTPUT=$(cargo bench 2>&1) || true

echo ""
echo "Generating $RESULTS_FILE ..."

TIMESTAMP=$(date -u "+%Y-%m-%d %H:%M:%S UTC")
RUSTC_VERSION=$(rustc --version)
CARGO_VERSION=$(cargo --version)
UNAME_INFO=$(uname -srm)

cat > "$RESULTS_FILE" <<HEADER
# Soplang Interpreter Benchmarks

> Auto-generated on **$TIMESTAMP**
> System: \`$UNAME_INFO\` | \`$RUSTC_VERSION\` | \`$CARGO_VERSION\`

## Benchmark Programs

| Benchmark | Description | Workload |
|-----------|-------------|----------|
| fib_recursive | Recursive Fibonacci | fib(25) = 75025 |
| loop_sum | Tight for-loop | Sum 1..100,000 |
| nested_loops | Nested loop dispatch | 200 x 200 iterations |
| string_concat | String concatenation | 1,000 appends |
| list_ops | List push + traversal | 5,000 elements |
| object_create | Object allocation | 2,000 objects |

## Criterion Results

\`\`\`
HEADER

echo "$BENCH_OUTPUT" | grep -E '(time:|Benchmarking|found|change|Performance)' >> "$RESULTS_FILE" 2>/dev/null || true

if echo "$BENCH_OUTPUT" | grep -q 'time:'; then
    echo "$BENCH_OUTPUT" | grep -B1 'time:' >> "$RESULTS_FILE" 2>/dev/null || true
else
    echo "$BENCH_OUTPUT" >> "$RESULTS_FILE"
fi

cat >> "$RESULTS_FILE" <<'FOOTER'
```

## Quick Timings (wall-clock)

| Benchmark | Time |
|-----------|------|
FOOTER

BINARY="$PROJECT_DIR/target/release/soplang"
for bench_file in "$SCRIPT_DIR"/*.sop; do
    name=$(basename "$bench_file" .sop)
    elapsed=$( { time "$BINARY" "$bench_file" > /dev/null 2>&1; } 2>&1 )
    real_time=$(echo "$elapsed" | grep real | awk '{print $2}')
    if [ -z "$real_time" ]; then
        real_time=$(echo "$elapsed" | grep -oP '[\d.]+s' | head -1)
    fi
    if [ -z "$real_time" ]; then
        real_time="$elapsed"
    fi
    echo "| $name | $real_time |" >> "$RESULTS_FILE"
done

cat >> "$RESULTS_FILE" <<'NOTES'

## How to Run

```bash
# Full criterion benchmarks (with HTML reports in target/criterion/)
cargo bench

# Quick wall-clock timings
bash benchmarks/run_benchmarks.sh

# Single benchmark group
cargo bench -- fibonacci
```

## Interpreting Results

- **Criterion** measures statistical performance: mean, median, standard deviation,
  and detects regressions/improvements between runs.
- **HTML reports** are generated in `target/criterion/` — open `report/index.html`
  in a browser for detailed graphs.
- **Wall-clock timings** include process startup overhead; criterion timings are
  pure in-process measurements and more accurate for comparison.
NOTES

echo ""
echo "Done! Results written to $RESULTS_FILE"
