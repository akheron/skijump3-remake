# Scenarios for the later phases

All use seed 1 and run to the end without `SJ3_MAXFRAMES`. "Frames" is the
`frames` value of the `End` line of the Pascal trace. Each run ends at the
first key wait after the last key, except `quit`, which ends when the game
exits.

| Scenario | Frames | Description |
| --- | --- | --- |
| `training` | 2315 | Training: Kuopio with start position `+`, `+`, `-`, a 121.5 m telemark jump, `-` and a crash, Esc in the inrun; Lahti with Esc before the jump; Vikersund with Esc on the bar; then `0` to the jump menu. |
| `listkeys` | 975 | World Cup qualification list paged with PgDn, Right, Left, Home, End (no effect), PgDn, PgUp, then F10 to the jump menu. |
| `tops` | 78 | Main menu item 4 through its 3 pages with PgDn, Home, Right, Space, Left, PgUp, PgDn, PgDn, Enter; then item 5 (one page) and Esc. |
| `koth` | 845 | King of the Hill: challenge 1 chosen with item 2 and played to the end (the human jumper presses Esc before each jump); then `choosekoth` with 4 jumpers, item 4 (Lahti), wind on, one round, played to the end; then `0` (writes `CONFIG.SKI`). |
| `teamcup` | 2352 | Main menu item 2 makes 8 jumpers, then a full team cup. The team names `SUOMI` and `NORJA` are typed with Del, letters and Backspace. The human jumpers press Esc before each jump. |
| `quit` | 162 | Turns the beeps on in the setup, answers `E` to the quit question, then `k`, so `Loppu` writes the files and the end text. |

A key with frame 999999 goes to a wait that does not poll: a result list, the
comment screen after a jump, the pages of `showtops`, and `choosekoth`.
`quit` ends with an extra 999999 key that the game never reads: without it,
`putsaa` in `Loppu` polls after the last key and the run stops before
`MakeSendMe`.
