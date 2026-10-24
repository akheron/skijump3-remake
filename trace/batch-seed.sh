#!/bin/sh
# Usage: batch-seed.sh <seed> <keys file> <maxframes> [extra data dir]
# Runs a scenario in the Pascal binary, replays it in Rust, and prints one PASS
# or FAIL line. The output goes to $RUNS/<keys name>-s<seed>, or with an extra
# data dir to $RUNS/<keys name>-s<seed>-<extra data dir name>. The files of the
# extra data dir go on top of the data files that the run starts with.
. "$(dirname "$0")/env.sh"
[ $# -ge 3 ] || { echo "Usage: batch-seed.sh <seed> <keys file> <maxframes> [extra data dir]" >&2; exit 2; }
seed=$1; keys=$2; max=$3
name=$(basename "$keys" .keys)-s$seed
if [ -n "$4" ]; then
  DATA_EXTRA=$(realpath "$4"); export DATA_EXTRA
  name=$name-$(basename "$4")
fi
mkdir -p "$RUNS"
SJ3_MAXFRAMES=$max SJ3_TIMEOUT=${SJ3_TIMEOUT:-900} "$TRACE_DIR/run-sdl.sh" "$keys" "$RUNS/$name" "$seed" > "$RUNS/$name.pascal.txt"
SJ3_TIMEOUT=${SJ3_TIMEOUT:-900} "$TRACE_DIR/run-rust.sh" "$RUNS/$name" "$RUNS/$name-rust" > "$RUNS/$name.rust.txt"
pascal=$(head -1 "$RUNS/$name.pascal.txt")
# The console text before it may end without a newline
rust=$(grep -o 'Trace replay complete: .*' "$RUNS/$name-rust/stdout.txt")
[ -n "$rust" ] || rust=$(tail -1 "$RUNS/$name-rust/stdout.txt")
jumps=$(grep -c ' JumpResult ' "$RUNS/$name/run/Trace.txt")
counts=$(echo "$pascal" | sed -n 's/^exit=\([0-9]*\) End frames=\([0-9]*\) records=\([0-9]*\) exitcode=\1$/\3 records and \2 frames/p')
files=$(grep '^files: ' "$RUNS/$name.rust.txt")
case "$rust" in
  "Trace replay complete: $counts match") [ -n "$counts" ] && [ "$files" = "files: same" ] && result=PASS || result=FAIL ;;
  *) result=FAIL ;;
esac
echo "$result $name: jumps=$jumps $pascal | $rust | $files"
[ $result = PASS ]
