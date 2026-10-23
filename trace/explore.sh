#!/bin/sh
# Usage: explore.sh <keys file> <name>
# Runs the Pascal binary with a key script that ends in a 999999 sentinel key,
# so the game stops at the first blocking key wait after the script. Writes
# the last frame to $RUNS/<name>.png. SJ3_TIMEOUT sets the timeout in
# seconds (default 4).
. "$(dirname "$0")/env.sh"
[ $# -ge 2 ] || { echo "Usage: explore.sh <keys file> <name>" >&2; exit 2; }
mkdir -p "$RUNS"
keys=$RUNS/$2.explore.keys
cp "$1" "$keys"; echo '999999 0 0' >> "$keys"
SJ3_FRAMES_FROM=0 SJ3_TIMEOUT=${SJ3_TIMEOUT:-4} "$TRACE_DIR/run-sdl.sh" "$keys" "$RUNS/$2" >/dev/null
# The timeout can kill the game while it writes the last frame.
last=$(find "$RUNS/$2/frames" -name '*.dat' -size 64768c | sort | tail -1)
python3 -I "$TRACE_DIR/frame2png.py" "$last" "$RUNS/$2.png"
echo "last frame $(basename "$last"); keys consumed: $(grep -c ' Keypress ' "$RUNS/$2/run/Trace.txt")"
