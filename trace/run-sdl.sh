#!/bin/sh
# Usage: run-sdl.sh <keys file> <out dir> [seed]
# Runs the Pascal reference binary $SJ3_BIN headless in <out dir>/run, in a
# fresh copy of the data files. The trace has a CRC of each frame. If
# SJ3_FRAMES_FROM or SJ3_FRAMES_TO is set, the frames in that range go to
# <out dir>/frames. SJ3_MAXFRAMES stops the run after that many frames. The
# exit status is that of the game, or 137 when SJ3_TIMEOUT kills it.
. "$(dirname "$0")/env.sh"
[ $# -ge 2 ] || { echo "Usage: run-sdl.sh <keys file> <out dir> [seed]" >&2; exit 2; }
keys=$(realpath "$1"); out=$(realpath -m "$2"); seed=${3:-1}
bin=$(realpath "$SJ3_BIN")
rm -rf "$out"; mkdir -p "$out/run" "$out/frames"
init_data "$out/run"
frames=""; [ -n "$SJ3_FRAMES_FROM$SJ3_FRAMES_TO" ] && frames="$out/frames"
cd "$out/run"
SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy SJ3_FAST=1 SJ3_MAXFRAMES=${SJ3_MAXFRAMES:-} SJ3_SEED=$seed SJ3_KEYS="$keys" SJ3_FRAMES="$frames" \
  timeout -s KILL "${SJ3_TIMEOUT:-300}" "$bin" > "$out/stdout.txt" 2>&1
status=$?
echo "exit=$status $(tail -1 Trace.txt | cut -d ' ' -f 2-)"
exit $status
