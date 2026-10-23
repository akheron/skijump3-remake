#!/bin/sh
# Builds both binaries, runs every scenario in parallel and replays it in Rust.
# Prints one line for each scenario. JOBS sets the number of parallel runs.
. "$(dirname "$0")/env.sh"
mkdir -p "$RUNS"
"$TRACE_DIR/build-sdl.sh" || exit 1
"$TRACE_DIR/build-rust.sh" 2> "$RUNS/build-rust.log" || { cat "$RUNS/build-rust.log"; exit 1; }
sc=$TRACE_DIR/scenarios
{
  echo "1 $sc/w1_worldcup_qual.keys 25900"
  echo "1 $sc/h2_early_overrotate_r.keys 3000"
  echo "1 $sc/h3_late_crash.keys 3000"
  for seed in 2 3 4; do echo "$seed $sc/cpu_qual.keys 40000"; done
  echo "1 $sc/multi5.keys 49372"
} | xargs -P "${JOBS:-$(nproc)}" -L 1 "$TRACE_DIR/batch-seed.sh"
status=$?
[ $status = 0 ] && echo "All scenarios pass" || echo "Some scenarios fail"
exit $status
