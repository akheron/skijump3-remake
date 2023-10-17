#!/bin/bash

set -e
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen ../target/wasm32-unknown-unknown/release/skijump3_web.wasm --out-dir dist --out-name skijump3 --target web

assets=(LANGBASE.SKI ANIM.SKI HISCORE.SKI CONFIG.SKI PLAYERS.SKI NAMES0.SKI MOREHILL.SKI HILLBASE.SKI MAIN.PCX LOAD.PCX FRONT1.PCX BACK1.PCX GOALS.SKI)
for asset in ${assets[@]}; do
  cp ../assets/${asset} dist/
done
cp index.html index.js setImmediate.js dist/

#if type wasm-opt >/dev/null 2>&1; then
#  wasm-opt -Oz -o dist/utk-level-editor_bg.wasm dist/utk-level-editor_bg.wasm
#else
#  echo "warning: wasm-opt not found"
#fi
