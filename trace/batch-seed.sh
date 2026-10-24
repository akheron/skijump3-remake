#!/bin/sh
# Usage: batch-seed.sh <seed> <keys file> <maxframes>
# Runs a scenario in the Pascal binary, replays it in Rust, and prints one PASS
# or FAIL line. The output goes to $RUNS/<keys name>-s<seed>.
. "$(dirname "$0")/env.sh"
[ $# -ge 3 ] || { echo "Usage: batch-seed.sh <seed> <keys file> <maxframes>" >&2; exit 2; }
seed=$1; keys=$2; max=$3
name=$(basename "$keys" .keys)-s$seed
mkdir -p "$RUNS"
SJ3_MAXFRAMES=$max SJ3_TIMEOUT=${SJ3_TIMEOUT:-900} "$TRACE_DIR/run-sdl.sh" "$keys" "$RUNS/$name" "$seed" > "$RUNS/$name.pascal.txt"
SJ3_TIMEOUT=${SJ3_TIMEOUT:-900} "$TRACE_DIR/run-rust.sh" "$RUNS/$name" "$RUNS/$name-rust" > "$RUNS/$name.rust.txt"
pascal=$(head -1 "$RUNS/$name.pascal.txt")
# The console text before it may end without a newline
rust=$(grep -o 'Trace replay complete: .*' "$RUNS/$name-rust/stdout.txt")
[ -n "$rust" ] || rust=$(tail -1 "$RUNS/$name-rust/stdout.txt")
jumps=$(grep -c ' JumpResult ' "$RUNS/$name/run/Trace.txt")
counts=$(echo "$pascal" | sed -n 's/^exit=0 End frames=\([0-9]*\) records=\([0-9]*\) exitcode=0$/\2 records and \1 frames/p')
files=$(grep '^files: ' "$RUNS/$name.rust.txt")
case "$rust" in
  "Trace replay complete: $counts match") [ -n "$counts" ] && [ "$files" = "files: same" ] && result=PASS || result=FAIL ;;
  *) result=FAIL ;;
esac
echo "$result $name: jumps=$jumps $pascal | $rust | $files"
[ $result = PASS ]
