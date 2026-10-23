# Paths shared by the trace scripts. Each one can be overridden in the environment.
TRACE_DIR=$(cd "$(dirname "$0")" && pwd)
REPO=$(dirname "$TRACE_DIR")
: "${SJ3_SDL:=$(dirname "$REPO")/skijump3-sdl-trace}"
: "${SJ3_BIN:=$TRACE_DIR/build/sj3}"
: "${RUST_BIN:=$REPO/target/release/desktop}"
: "${RUNS:=$TRACE_DIR/runs}"
export SJ3_SDL SJ3_BIN RUST_BIN RUNS

# Copies the data files that a run starts with to $1.
init_data() {
  if [ -n "$DATA" ]; then
    cp "$DATA"/* "$1/"
  else
    cp "$REPO"/assets/* "$1/"
    cp "$TRACE_DIR"/data-init/* "$1/"
  fi
}
