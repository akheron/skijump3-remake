# Ski Jump International v3 (remake)

This repository will be a complete rewrite of the legendary DOS game Ski Jump International v3. It's a very early work
in progress, and if you'd like to play the game now, see [the original game's website](https://www.nomasi.com/sj3/).

## Goals

The main goal of this project is to port the game to mobile web browsers.

Why does it need a complete rewrite? The original game is written in Pascal and is DOS-only.
An [SDL2 port exists](https://github.com/suomipelit/skijump3-sdl), but the combination of Pascal and SDL is not possible
to compile to web browsers.

## How?

The remake will be written in Rust.

**Phase 1: Rewrite**

- Make an "almost line-by-line" rewrite of the SDL2 port, using the sdl2 rust crate.
- This will be mostly a translation of Pascal syntax to Rust syntax.
- Also requires handling of global mutable state (there's lots of it), and porting I/O and other platform-specific
  code.

**Phase 2: Refactor**

- Refactor the code to be more idiomatic Rust.
- Refactor game state and logic so that there's one main loop that can give up control after rendering each frame (a
  requirement for web browsers).

**Phase 3: Web**

- Port the game to web browsers. This requires some amount of JS glue code.
- Invent a touch UI that works well in mobile browsers.

The original game contains about 10k lines of Pascal code. Phase 1 about 25 % done.

## Development

1. Install SDL development library
2. Copy `*.PCX` and `*.SKI` from the [original game](https://github.com/suomipelit/skijump3) to the project root
3. Run `cargo run`
