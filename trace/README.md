# Trace tools

These scripts check the remake against the Pascal SDL port. The Pascal
reference build runs a scenario from a key script and writes a trace of the
input, the random numbers, the game state and a CRC of every frame. The Rust
desktop build replays the trace and stops at the first record or frame that
differs. See the "Tracing" section of the README of the `sp4-trace`
branch of the SDL port for the trace format.

## Setup

You need x86_64 Linux: the reference build computes floats in 80-bit x87
precision, which other targets do not have. Install Free Pascal 3.2.2
(Ubuntu: `fp-compiler-3.2.2`), the SDL2 and X11 libraries (`libsdl2-2.0-0`,
`libx11-6`), `curl`, `dpkg-deb` and `git`. Then:

```
trace/fetch-deps.sh
git -C ../skijump3-sdl worktree add ../skijump3-sdl-trace sp4-trace
```

`fetch-deps.sh` puts the X11 units of Free Pascal, the Pascal SDL2 headers
and links to the SDL2 and X11 libraries in `trace/deps`. The scripts expect
the `sp4-trace` branch of the SDL port in `../skijump3-sdl-trace`, next to
this repository.

The scripts read these environment variables:

| Variable | Default | Description |
| --- | --- | --- |
| `SJ3_SDL` | `../skijump3-sdl-trace` | Source of the reference build |
| `SJ3_BIN` | `trace/build/sj3` | Reference binary |
| `RUST_BIN` | `target/release/desktop` | Rust binary |
| `RUNS` | `trace/runs` | Output of `batch-seed.sh`, `regress.sh`, `explore.sh` and `multicomp.py` |
| `DATA` | | Directory of the data files that a run starts with. The default is `assets/` with `trace/data-init/` on top: its `CONFIG.SKI` has `reg=1` and `languagenumber=2` (Finnish). |
| `SJ3_MAXFRAMES` | | Stop the Pascal run after this many frames |
| `SJ3_FRAMES_FROM` | | Dump the frames of a Pascal run from this frame on. A frame dump is 64000 bytes of pixels and 768 bytes of 6-bit palette. |
| `SJ3_FRAMES_TO` | | Dump the frames of a Pascal run up to this frame |
| `SJ3_TIMEOUT` | 300 | Kill a run after this many seconds |
| `JOBS` | number of CPUs | Parallel scenarios in `regress.sh` |

## Build

```
trace/build-sdl.sh
trace/build-rust.sh
```

`build-sdl.sh` builds the reference binary `trace/build/sj3` from the SDL
worktree. Run it again after a change in the SDL branch. `build-rust.sh`
builds `target/release/desktop` with the `trace` feature.

## Run a scenario and replay it

```
SJ3_MAXFRAMES=25900 trace/run-sdl.sh trace/scenarios/w1_worldcup_qual.keys trace/runs/w1 1
trace/run-rust.sh trace/runs/w1 trace/runs/w1-rust
```

The last argument of `run-sdl.sh` is the random seed. The Pascal run writes:

- `trace/runs/w1/run/Trace.txt`: the trace, with a `Frame` line for each
  frame: the frame number (from 0) and the CRC-32 of the frame dump. The
  last line is `End frames=N records=N exitcode=N`: the number of frames,
  the number of lines before it and the exit code. The run writes it when it
  exits.
- `trace/runs/w1/run/`: the data files, also the ones that the game wrote
- `trace/runs/w1/frames/NNNNNN.dat`: the frames from `SJ3_FRAMES_FROM` to
  `SJ3_FRAMES_TO`, if one of them is set. 40000 frames take 2.6 GB.
- `trace/runs/w1/stdout.txt`

`run-rust.sh` runs the Rust binary in `trace/runs/w1-rust`, with the same
data files in `assets/`. The replay must reach the `End` line in a key wait,
in a frame or at the end of the game, with the same numbers of frames and
lines. Then the last line of `trace/runs/w1-rust/stdout.txt` is `Trace replay
complete: N records and M frames match`. Otherwise the run panics, and
`run-rust.sh` prints the panic. When the CRC of a
frame differs, the Rust frame goes to `trace/runs/w1-rust/actual.dat`, and the
panic message gives the commands that dump the Pascal frame and replay again.
With the last argument `frames`, `run-rust.sh` gives the frame dumps of the
Pascal run to the Rust binary, and the panic message tells how many pixels and
which palette entries differ.

`batch-seed.sh <seed> <keys file> <maxframes>` does both steps in
`$RUNS/<keys name>-s<seed>` and prints one `PASS` or `FAIL` line.

## View a frame

```
SJ3_FRAMES_FROM=500 SJ3_FRAMES_TO=500 SJ3_MAXFRAMES=501 trace/run-sdl.sh trace/scenarios/w1_worldcup_qual.keys trace/runs/w1 1
python3 trace/frame2png.py trace/runs/w1/frames/000500.dat frame.png
python3 trace/states.py trace/runs/w1/run/Trace.txt 10
```

`states.py <trace> [step] [fields...]` decodes the floats of the `JumpState`,
`LandState` and `JumpResult` records. It prints every `step`th state and every
result.

## Regression set

```
trace/regress.sh
```

It builds both binaries, runs every scenario in parallel, and prints one line
for each scenario and a summary line. The exit status is 0 if all pass. A
scenario passes when the Pascal run exits with exit code 0, and the Rust
replay completes.

| Scenario | Seeds | Frames | Description |
| --- | --- | --- | --- |
| `w1_worldcup_qual` | 1 | 25900 | World Cup. The human jumper jumps 122 m in the qualification and lands with telemark. |
| `h2_early_overrotate_r` | 1 | 3000 | The human jumper rotates forward too early, lands with `R` and falls. |
| `h3_late_crash` | 1 | 3000 | The human jumper does not land and falls. |
| `cpu_qual` | 2, 3, 4 | 40000 | World Cup. The human jumper does not start, so the computer jumpers do the first competition. |
| `multi5` | 1 | 49372 | World Cup through 5 competitions, made with `multicomp.py`. |
| `fourhills` | 1 | 3900 | Four Hills to the end of the cup, made with `multicomp.py` from the start keys `10 13 0`, `20 51 0`, `30 13 0` and `40 13 0`. The human jumper does not start. The run stops in the jump menu. Not in the regression set: at the end of `wcup`, the Rust port does not call `WriteProfiles`, `WriteRecords` and `MakeSendMe`, so the replay fails after the last jump. |
| `a_training_kuopio` | | | Training in Kuopio. Not in the regression set: the Rust replay gives a black frame 33, after training is chosen in the menu. |

## Write a key script

A key script has one key on each line: `frame ch1 ch2`. `ch1` and `ch2` are
the two bytes that `readkey` gives in the DOS version: `ch1` is the character
code, and for a special key `ch1` is 0 and `ch2` is the scan code. Examples:
Enter `13 0`, `3` `51 0`, `T` `84 20`, up `0 72`, left `0 75`, right `0 77`.

- When the game polls the keyboard (`KeyPressed`), the next key is available
  when the frame with number `frame` is rendered. The frame numbers are the
  numbers of the frame dumps.
- When the game waits for a key without polling, like on the result screens,
  it gets the next key at once.
- A key with frame 999999 is never available to polling, so it goes only to
  the waits that do not poll.
- The game exits when it polls or waits for a key after the last one.

To find the frames, run the keys that you have with `explore.sh`:

```
trace/explore.sh my.keys my
```

It adds a 999999 key, runs the game until it waits for a key that it polls for
(or for `SJ3_TIMEOUT`, default 4 seconds) and writes the last frame to
`trace/runs/my.png`. Then add the next key at a frame after that one. During a
jump, the `frame` field of the `JumpState` records tells the frame numbers.

`multicomp.py <seed> <waits> <out keys file> [start keys file]` makes a key
script for a cup in which the human jumper does not start. It runs the game,
finds where it waits for a key by polling, adds an Enter there, and does this
`waits` times. The start keys file has the keys that start the cup and the
first jump (the default starts a World Cup). It prints the frame count for
`SJ3_MAXFRAMES`. Each run goes on for `MARGIN` frames (default 45000) after
the last start key or the last wait. A smaller value makes the runs faster,
but it must be more than the number of frames between two waits.

## Rebase check

```
trace/check-rebase.sh
```

The `sp4-trace` branch starts from SP4 (`c485d0e`) of the SDL port. This
script rebases it onto SDL `master` in a temporary worktree and prints the
conflicts of each commit. It resolves each conflict with the version of the
trace branch to continue, so a later commit that changes the same lines can
have a conflict that the script does not show. Run it after each new commit in
the SDL branch.
