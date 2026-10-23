#!/bin/sh
# Usage: build-sdl.sh [source dir] [output dir]
# Builds the Pascal reference binary. The defaults are $SJ3_SDL and the
# directory of $SJ3_BIN.
set -e
. "$(dirname "$0")/env.sh"
src=$(realpath "${1:-$SJ3_SDL}")
out=$(realpath -m "${2:-$(dirname "$SJ3_BIN")}")
deps=$TRACE_DIR/deps
x11=$deps/base/usr/lib/x86_64-linux-gnu/fpc/3.2.2/units/x86_64-linux/x11
if [ ! -d "$x11" ] || [ ! -d "$deps/Pascal-SDL-2-Headers" ]; then
  echo "Missing $deps, run fetch-deps.sh" >&2
  exit 1
fi
mkdir -p "$out"
cd "$src"
fpc -B -Xs- -Mtp -Fu"$deps/Pascal-SDL-2-Headers" -Fu"$x11" -Fl"$deps/lib" -FU"$out" -o"$out/sj3" SJ3.PAS > "$out/build.log" 2>&1 || {
  grep -i -B2 -A2 'error\|fatal' "$out/build.log"
  exit 1
}
