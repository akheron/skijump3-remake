# Trace tools

These scripts check the remake against the Pascal SDL port. The Pascal
reference build runs a scenario from a key script and writes a trace of the
input, the random numbers, the clock, the files that it finds, the game state,
the files that it writes, its console text and a CRC of every frame. The Rust
desktop build replays the trace and stops at the first line that differs. See the "Tracing" section of the README of the `sp4-trace`
branch of the SDL port for the trace format.

## Setup

You need x86_64 Linux: the reference build computes floats in 80-bit x87
precision, which other targets do not have. Install Free Pascal 3.2.2
(Ubuntu: `fp-compiler-3.2.2`), the SDL2 and X11 libraries (`libsdl2-2.0-0`,
`libx11-6`), `curl`, `dpkg-deb`, `git` and `python3`. Then:

```
trace/fetch-deps.sh
git -C ../skijump3-sdl worktree add ../skijump3-sdl-trace sp4-trace
```

`fetch-deps.sh` puts the X11 units of Free Pascal, the Pascal SDL2 headers
and links to the SDL2 and X11 libraries in `trace/deps`. The scripts expect
the `sp4-trace` branch of the SDL port in `../skijump3-sdl-trace`, next to
this repository. `sp4-trace` is a local branch of the clone of the SDL port
in `../skijump3-sdl`. It starts from SP4 (`c485d0e`), and it is not on a
remote.

The scripts read these environment variables:

| Variable | Default | Description |
| --- | --- | --- |
| `SJ3_SDL` | `../skijump3-sdl-trace` | Source of the reference build |
| `SJ3_BIN` | `trace/build/sj3` | Reference binary |
| `RUST_BIN` | `target/release/desktop` | Rust binary |
| `RUNS` | `trace/runs` | Output of `batch-seed.sh`, `regress.sh`, `explore.sh` and `multicomp.py` |
| `DATA` | | Directory of the data files that a run starts with. The default is `assets/` with `trace/data-init/` on top: its `CONFIG.SKI` has `reg=1` and `languagenumber=2` (Finnish). |
| `DATA_EXTRA` | | Directory of files that go on top of the data files that a run starts with |
| `SJ3_MAXFRAMES` | | Stop the Pascal run after this many frames. -1 does not stop it. |
| `SJ3_FRAMES_FROM` | | Dump the frames of a Pascal run from this frame on. A frame dump is 64000 bytes of pixels and 768 bytes of 6-bit palette. |
| `SJ3_FRAMES_TO` | | Dump the frames of a Pascal run up to this frame |
| `SJ3_TIMEOUT` | 300, 900 in `batch-seed.sh`, 4 in `explore.sh` | Kill a run after this many seconds |
| `SJ3_TRACE_ENDTEXT` | | Trace the end text that the game writes to the console when it exits. Set it for both runs, or for neither. |
| `JOBS` | number of CPUs | Parallel scenarios in `regress.sh` |
| `FULL` | | Run `cpu_qual` with the seeds 2 to 10 in `regress.sh` |

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

`run-rust.sh` runs the Rust binary in `trace/runs/w1-rust`, with the same data
files in its data directory `trace/runs/w1-rust/run`. The replay must reach
the `End` line in a key wait, in a frame or at the end of the game, with the
same numbers of frames and lines. Then the last line of
`trace/runs/w1-rust/stdout.txt` is `Trace replay complete: N records and M
frames match`. Otherwise the run panics, and `run-rust.sh` prints the panic.
When the CRC of a frame differs, the Rust frame goes to
`trace/runs/w1-rust/actual.dat`, and the panic message gives the commands that
dump the Pascal frame and replay again. With the last argument `frames`,
`run-rust.sh` gives the frame dumps of the Pascal run to the Rust binary, and
the panic message tells how many pixels and which palette entries differ. At
the end, `run-rust.sh` compares `trace/runs/w1-rust/run` with
`trace/runs/w1/run` byte by byte, without `Trace.txt`, and prints `files:
same`, or `files: differ` and the files that differ.

`run-sdl.sh` exits with the exit status of the game, or 137 when it kills the
game after `SJ3_TIMEOUT` seconds.

`batch-seed.sh <seed> <keys file> <maxframes> [extra data dir]` does both
steps in `$RUNS/<keys name>-s<seed>` and prints one `PASS` or `FAIL` line.
With an extra data dir, it sets `DATA_EXTRA` to it, and the output goes to
`$RUNS/<keys name>-s<seed>-<extra data dir name>`.

## View a frame

```
SJ3_FRAMES_FROM=500 SJ3_FRAMES_TO=500 SJ3_MAXFRAMES=501 trace/run-sdl.sh trace/scenarios/w1_worldcup_qual.keys trace/runs/w1 1
python3 trace/frame2png.py trace/runs/w1/frames/000500.dat trace/runs/frame.png
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
scenario passes when the Pascal run exits with exit code 0, the Rust replay
completes, and the data files of the two runs are the same.

| Scenario | Seeds | Frames | Description |
| --- | --- | --- | --- |
| `w1_worldcup_qual` | 1 | 25900 | World Cup. The human jumper jumps 122 m in the qualification and lands with telemark. |
| `h2_early_overrotate_r` | 1 | 3000 | The human jumper rotates forward too early, lands with `R` and falls. |
| `h3_late_crash` | 1 | 3000 | The human jumper does not land and falls. |
| `cpu_qual` | 2, 3, 4 (2 to 10 with `FULL=1`) | 40000 | World Cup. The human jumper does not start, so the computer jumpers do the first competition. |
| `multi5` | 1 | 49372 | World Cup through 5 competitions, made with `multicomp.py`. |
| `worldcup_full` | 1 | -1 | World Cup through all 20 competitions to the end of the cup, made with `multicomp.py` and `MARGIN=4000`. The human jumper does not start. After the last competition, the keys 999999 go through the result lists, and the end of the cup writes `PLAYERS.SKI`, `HISCORE.SKI` and `SENDME.TXT`. The run stops in the jump menu. The replay takes about 3 minutes. |
| `setup_menu` | 1 | 5000 | Each item of the setup menu that works in the Rust port: the language, beeps and graphics detail; the name file with a set that does not exist and Esc; every item of the jumping settings; the computer jumpers and the place of the wind meter; the first two hiscore settings, and the two questions that reset the hiscores, answered with `E`; one key and the default keys; the length goals; and the list of the hill maker. Then it chooses the last language (Slovak) with item 0, the name file `NAMES1.SKI`, answers `S` (yes in Slovak) to both questions that reset the hiscores, resets the settings and leaves the setup menu, which writes `CONFIG.SKI`. The run ends in the main menu. The `hillmaker` scenario makes new hills. |
| `k4_start` | 1 | 2000 | World Cup. The human jumper starts with `T` (K[4]) on the bar and jumps 120.5 m with telemark. The run ends after the comment screen. |
| `k4_start` with `wind11`, `wind12`, `wind13` | 1 | 2000 | `k4_start` with the wind meter at the place 11, 12 or 13: the extra data dirs `trace/scenarios/wind11` to `wind13` have a `CONFIG.SKI` with that place. |
| `listkeys` | 1 | -1 | World Cup. The qualification list is paged with PgDn, Right, Left, Home, End (no effect), PgDn and PgUp, then F10 goes to the jump menu. |
| `hiscore_error` with `hiscore_error` | 1 | 100 | The extra data dir `trace/scenarios/hiscore_error` has a `HISCORE.SKI` with a changed name, so the checksum is wrong. The game writes new `HISCORE.SKI` and `CONFIG.SKI` files, waits for a key and exits. |
| `hiscore_error` with `hiscore_short` | 1 | 100 | The extra data dir `trace/scenarios/hiscore_short` has a `HISCORE.SKI` with only its first 150 lines. At the end of the file, the strings that the game reads are empty and the numbers are 0, so the checksum is wrong, and the game does the same as in `hiscore_error`. |
| `fourhills` | 1 | 3900 | Four Hills to the end of the cup, made with `multicomp.py` from the start keys `10 13 0`, `20 51 0`, `30 13 0` and `40 13 0`. The human jumper does not start. The end of the cup writes `PLAYERS.SKI`, `HISCORE.SKI` and `SENDME.TXT`. The run stops in the jump menu. |
| `quit` | 1 | -1 | Turns the beeps on in the setup, answers `E` to the quit question, then `k`, so `Loppu` writes the files and the end text. The last key, 999999, is never read: without it, `putsaa` in `Loppu` polls after the last key and the run stops before `MakeSendMe`. |
| `training` | 1 | -1 | Training: Kuopio with the start position changed with `+`, `+`, `-`, a 121.5 m telemark jump, `-` and a crash, Esc in the inrun; Lahti with Esc before the jump; Vikersund with Esc on the bar; then `0` to the jump menu. |
| `a_training_kuopio` | 1 | -1 | Training in Kuopio: one 121 m jump that lands with `T`. The run ends after the comment screen. |
| `tops` | 1 | -1 | Main menu item 4 through its 3 pages with PgDn, Home, Right, Space, Left, PgUp, PgDn, PgDn, Enter; then item 5 (one page) and Esc. |
| `fourhills_hiscore` | 1 | -1 | The setup menu turns off the hill records of the computer jumpers and resets the hiscores to zero (`K` to the question). Four Hills, made with `multicomp.py` after the first jumps: the human jumper jumps 113 m in the qualification in Oberstdorf, which is a hill record, and 109 m in the first round, and does not start after that. At the end of the cup the human jumper gets onto the Four Hills list (`sorthighs`, `dolist`). In the main menu, item 4 shows the 3 pages of lists, and item 5 the hill records with the A.H.I. The run ends in the main menu. |
| `koth` | 1 | -1 | King of the Hill: challenge 1 chosen with item 2 and played to the end (the human jumper presses Esc before the jump and drops out in the first round); then `choosekoth` with 4 jumpers, item 4 (Lahti), wind on, one round, played to the end; then `0` (writes `CONFIG.SKI`). The run ends in the jump menu. |
| `koth_win` | 1 | -1 | King of the Hill challenge 6 (7 jumpers, one round, wind). The human jumper jumps with telemark in each round and wins the challenge, so `kothlista` adds a win to the challenge on the top list and sets the King of the Hill level of the profile to 6. Then `0` leaves the menu (writes `CONFIG.SKI`), and the game quits from the main menu, which writes `HISCORE.SKI`, `CONFIG.SKI`, `PLAYERS.SKI` and `SENDME.TXT`. |
| `profiles` | 1 | -1 | Main menu item 2 makes 8 profiles as in `teamcup` (they all go into the order). Profile 1 is edited: the name with Del, letters and Backspace (`getstr`), the real name typed and cancelled with Esc, then typed again, the suit and ski colors, the replaced jumper with Down, PgDn and Up (`selectreplace`), the coach style and the qualification setting. Then Del takes profile 1 out of the order and Enter puts it at the end. Del takes profile 2 out of the order, and a second Del asks to delete it (`question`): `E` keeps it, `K` deletes it. Edit item 8 resets the next profile with `K`. F10 goes to the main menu, and `0`, Enter and `K` quit, so `Loppu` writes the files, also `PLAYERS.SKI`. |
| `profile_worldcup` | 1 | -1 | World Cup with profile 1 as the human jumper, made with `multicomp.py` from the first keys of `w1_worldcup_qual`: the 122 m jump in the qualification of the first competition; after that the human jumper does not start. In the qualification list of the second competition, F10 leaves the cup. `0` goes to the main menu, and item 2 shows the stats of the profile (`writeprofile`): 2 jumps and the longest jump, 122.0 m in Kuopio. Then F10, and `0`, Enter and `K` quit, so `Loppu` writes the files. |
| `teamcup` | 1 | -1 | Main menu item 2 makes 8 jumpers, then a full team cup. The team names `SUOMI` and `NORJA` are typed with Del, letters and Backspace. The human jumpers press Esc before each jump. |
| `teamcup_hiscore` | 1 | -1 | Main menu item 2 makes 8 jumpers as in `teamcup`. The setup menu turns on the stats after each cup list and resets the hiscores to zero (`K` to the question). Team cup with the default team names: the 4 jumpers of team 1 jump with telemark in both rounds of the first competition (Vikersund), and the team is third and gets 300 points (`jcupluettelo`). The jumpers of team 2, and of team 1 after the first competition, press Esc. After each cup list `showstats` shows the stats of both teams. At the end of the cup team 1 gets onto the team top list (`sorthighs`). `0` goes to the main menu, and `0`, Enter and `K` quit, so `Loppu` writes the files. |
| `teamwarning` | 1 | -1 | Turns the beeps on in the setup. With one jumper, jump menu item 4 beeps and shows `teamwarning`. Then `0` goes to the main menu. |
| `hillmaker` | 1 | -1 | Setup menu item 6, the hill maker. A new hill: each item of `edithill` is edited (`getstr`, `getint`, `getnum` and `getstr2`): a K point below the range, front and back indexes of files that do not exist, a value that is not a number, an empty value, values above the range, Esc in the author and in the file name. Exit and save writes `MAKI1.SJH`. The hill is edited again, and the question to write over the file is answered `N`, then `Y`. It is opened once more and closed with item 12. A second new hill is saved as `NEW2.SJH`, opened and closed with item 12. Then the game quits, and `Loppu` writes the hills again (`WriteExtraHillHR`). |
| `extrahill` with `extrahill` | 1 | -1 | The extra data dir `trace/scenarios/extrahill` has the `.SJH` files that `hillmaker` writes. Training: `selecthill` goes to the second page of hills, back to the first page, and again to the second page. One 68 m telemark jump on `MAKI1`, then Esc and `0`. King of the Hill challenge 6 on the hill `NEW2` (item 4 and the second page of `selecthill`). Esc twice during the first jump of a computer jumper, and Esc in the turn of the human jumper, which ends the first round. The run ends after the first round. The `customcup` scenario has a competition on an extra hill. |
| `extrahill_error` with `extrahill_error` | 1 | -1 | The extra data dir `trace/scenarios/extrahill_error` has `MAKI1.SJH` and `BAD.SJH`, a copy of `NEW2.SJH` with a changed name, so the checksum is wrong. `LoadInfo` writes a warning, waits for a key and uses the default hill, at the start (twice) and when training starts on `BAD`. The default hill keeps the profile checksum of the file, which is not the checksum of `FRONT1.PCX`, so `LoadHill` shows the alert box. Then Esc and `0`. |
| `customcup` with `customcup` | 1 | -1 | The extra data dir `trace/scenarios/customcup` has the `.SJH` files of `extrahill` and a `TEMP.SJC` with the hill `MAKI1`. `selectcustomhills` loads it first, and it sets `sortby`, which `cup` does not set. The setup menu turns off the hill records of the computer jumpers. In jump menu item 2, `selectcustomhills` removes the hill with Backspace, chooses `MAKI1` with End and Left, adds it with Space and saves the set as `CUSTOM` (`saveset`). It adds Kuopio and saves the set as `B` (Del clears the name), then tries to save it as `CUSTOM` and answers `N` to the question to write over the file. `L` (`selectload`) deletes `B` with Del and `K`, and loads `CUSTOM` with Enter. Enter starts the cup, one competition on `MAKI1`. The human jumper jumps 108.5 m and 110.0 m with telemark, sets the hill record of `MAKI1` and is 22nd, which is the first custom cup record of `CUSTOM` (`newcrecordscreen`). In the main menu, item 4 goes through its 4 pages with PgDn, the last one with the custom sets, and Esc. Then `0`, Enter and `k` quit, so `Loppu` writes the files, also `MAKI1.SJH` with the new hill record (`WriteExtraHillHR`). |
| `replay_save` | 1 | -1 | The setup menu turns on the automatic hill record replays and resets the hiscores to zero (`K` to the question). Training in Kuopio: after a 122.5 m telemark jump, `T` on the comment screen opens `replayinfo`. The author (`PL`) and the name (`TEST`) are typed after Del, and the replay is saved as `JUMP1` (`writereplay`). After the second jump, `T` again, the file name `JUMP1` and save: the question to write over the file is answered `E`, then `K`. Esc and `0` leave the training. World Cup: the human jumper jumps 121.0 m in the qualification, which is a hill record. `T` saves the replay with the default name `TEMP`, and the game saves the hill record replay `HR-KUO.SJR` with the author `HILLBASE`, because `writereplay` changes the string that the save block gives to it as the author. F10 in the qualification list leaves the cup, and `0`, Enter and `k` quit, so `Loppu` writes the files. The extra data dir `trace/scenarios/replays` has the files that this run writes: the 3 `.SJR` files, `CONFIG.SKI`, `HISCORE.SKI` and `PLAYERS.SKI`. |
| `replays` with `replays` | 1 | -1 | Main menu item 6 with the replays of `replay_save` and `INTRO.SJR`. The list is paged with Right, Down, Space, `+`, Left, Up and `-`, and Enter plays `JUMP1`. The playback uses Right (play and pause), `P`, Space (one step), `+` and Down (faster, also when paused and at the highest speed), `-` and Up (slower, down to the custom speed 0 and below it), Left (rewind and pause), plays to the end at 200% and stops with Esc. Then Enter plays `INTRO` in intro mode: Enter closes 4 tutorial boxes and Esc stops it. Del deletes `TEMP.SJR` (`mightdelete`, `K`), and Esc goes to the main menu. |
| `replay_check` with `replay_check` | 1 | -1 | Main menu item 6 with replays that do not pass the check: `NOHILL.SJR` is on the extra hill `NOHILL`, which does not exist, and has a checksum that is correct with the profile of Kuopio; `BAD.SJR` has a changed author, so the checksum is wrong; `LONGREPLAYNAME.SJR` has a name with more than 12 characters, so the game does not find the file `LONGREPLAYNA.SJR`. Right goes through the list to `NOHILL` again, so that each check result (0 to 3) shows. Then Esc. |
| `intro` with `intro` | 1 | -1 | The extra data dir `trace/scenarios/intro` has a `CONFIG.SKI` with `languagenumber=255`, so the game starts with `WelcomeScreen`. Enter chooses English, and the intro replay plays to the end with its 11 tutorial boxes, each closed with Enter. The run ends in the main menu. |

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
- The game exits when it polls or waits for a key after the last one. Then
  `frames=` in the `End` line of the trace is the number of the frame in
  which it polled, and the next key can come some frames after it.

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
