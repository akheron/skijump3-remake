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
status=$?
# The game polls for the sentinel key until the timeout kills it (137), or it
# exits (0) when the script runs out in a key wait that does not poll.
case $status in
  0|137) ;;
  *) echo "The game exited with status $status, see $RUNS/$2/stdout.txt" >&2 ;;
esac
# The timeout can kill the game while it writes the last frame.
last=$(find "$RUNS/$2/frames" -name '*.dat' -size 64768c | sort | tail -1)
python3 -I "$TRACE_DIR/frame2png.py" "$last" "$RUNS/$2.png"
echo "last frame $(basename "$last"); keys consumed: $(grep -c ' Keypress ' "$RUNS/$2/run/Trace.txt")"
case $status in 0|137) ;; *) exit $status ;; esac
