#!/bin/sh
# Fetches to deps/ what build-sdl.sh and build-rust.sh need besides the Free
# Pascal 3.2.2 compiler (Ubuntu package fp-compiler-3.2.2) and the SDL2 and
# X11 shared libraries (libsdl2-2.0-0, libx11-6):
# - the X11 units of Free Pascal (Ubuntu package fp-units-base-3.2.2)
# - the Pascal SDL2 headers
# - libSDL2.so and libX11.so links for the linker, so that the -dev packages
#   are not needed
set -e
. "$(dirname "$0")/env.sh"
deps=$TRACE_DIR/deps
mkdir -p "$deps/debs" "$deps/lib"

deb=fp-units-base-3.2.2_3.2.2+dfsg-32_amd64.deb
sha256=d52594af32a89b2a80da59d90e2b621e98f567bfb94d4e39cb35215f569c4dbd
if [ ! -f "$deps/debs/$deb" ]; then
  curl -fsSL -o "$deps/debs/$deb.part" "http://archive.ubuntu.com/ubuntu/pool/universe/f/fpc/$deb"
  mv "$deps/debs/$deb.part" "$deps/debs/$deb"
fi
echo "$sha256  $deps/debs/$deb" | sha256sum -c --quiet
rm -rf "$deps/base"
dpkg-deb -x "$deps/debs/$deb" "$deps/base"

headers=$deps/Pascal-SDL-2-Headers
if [ ! -d "$headers" ]; then
  git clone -q https://github.com/ev1313/Pascal-SDL-2-Headers "$headers"
fi
git -C "$headers" checkout -q 201a767368020183b9964983d1c57ac4bd6a16fd

for lib in libSDL2-2.0.so.0:libSDL2.so libX11.so.6:libX11.so; do
  path=$(ldconfig -p | awk -v name="${lib%%:*}" '$1 == name && /x86-64/ { print $NF; exit }')
  if [ -z "$path" ]; then
    echo "${lib%%:*} not found" >&2
    exit 1
  fi
  ln -sf "$path" "$deps/lib/${lib#*:}"
done
