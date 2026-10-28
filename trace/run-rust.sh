#!/bin/sh
# Usage: run-rust.sh <pascal out dir> <out dir> [frames]
# Replays the trace of a run-sdl.sh run in the Rust binary $RUST_BIN. Its data
# directory is <out dir>/run, a fresh copy of the data files that the Pascal
# run started with. With "frames", it compares the frames that differ from the
# trace with the frame dumps of the Pascal run. At the end, it compares the data
# directory with the run directory of the Pascal run byte by byte.
. "$(dirname "$0")/env.sh"
[ $# -ge 2 ] || { echo "Usage: run-rust.sh <pascal out dir> <out dir> [frames]" >&2; exit 2; }
ref=$(realpath "$1"); out=$(realpath -m "$2")
bin=$(realpath "$RUST_BIN")
rm -rf "$out"; mkdir -p "$out/run"
init_data "$out/run"
cd "$out"
if [ "$3" = frames ]; then export SJ3_FRAMES="$ref/frames"; fi
SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy SJ3_DATA_DIR="$out/run" SJ3_TRACE="$ref/run/Trace.txt" \
  timeout -s KILL "${SJ3_TIMEOUT:-300}" "$bin" > "$out/stdout.txt" 2>&1
echo "exit=$?"
grep -A30 panicked "$out/stdout.txt" | head -40
if diff -rq -x Trace.txt "$ref/run" "$out/run" > "$out/files.txt"; then
  echo "files: same"
else
  echo "files: differ"
  cat "$out/files.txt"
fi
