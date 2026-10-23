#!/bin/sh
# Builds the Rust binary that replays traces.
set -e
. "$(dirname "$0")/env.sh"
cd "$REPO"
LIBRARY_PATH="$TRACE_DIR/deps/lib${LIBRARY_PATH:+:$LIBRARY_PATH}" cargo build --release -p desktop --features trace
