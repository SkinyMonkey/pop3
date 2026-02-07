#!/usr/bin/env bash
# Run all camera test scripts and validate output.
# Usage: bash scripts/run_all_tests.sh '<path_to_pop_base>'

BASE="$1"
if [ -z "$BASE" ]; then
    echo "Usage: $0 '<path_to_populous_base_dir>'"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LOG="/tmp/faithful_debug.jsonl"
FAILED_LIST=""
FAILED=0
PASSED=0

for script in "$SCRIPT_DIR"/test_*.txt; do
    name="$(basename "$script")"
    echo "=== Running $name ==="

    # Clear previous log
    > "$LOG"

    if ! cargo run --release -- --base "$BASE" --level 1 --script "$script" 2>&1; then
        echo "FAIL: $name - cargo run failed"
        FAILED=$((FAILED + 1))
        FAILED_LIST="$FAILED_LIST  - $name (cargo run failed)\n"
        continue
    fi

    if python3 "$SCRIPT_DIR/check_camera.py" "$LOG"; then
        echo "PASS: $name"
        PASSED=$((PASSED + 1))
    else
        echo "FAIL: $name"
        FAILED=$((FAILED + 1))
        FAILED_LIST="$FAILED_LIST  - $name\n"
    fi
    echo ""
done

echo "=== Summary ==="
echo "Passed: $PASSED"
echo "Failed: $FAILED"

if [ "$FAILED" -gt 0 ]; then
    echo ""
    echo "Failed tests:"
    echo -e "$FAILED_LIST"
    exit 1
fi
echo "All tests passed."
