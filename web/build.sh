#!/bin/bash

set -e
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen ../target/wasm32-unknown-unknown/release/skijump3_web.wasm --out-dir dist --out-name skijump3 --target web

cp index.html index.js setImmediate.js dist/

(cd .. && cargo run --bin pack)
mv ../assets.pack dist/

#if type wasm-opt >/dev/null 2>&1; then
#  wasm-opt -Oz -o dist/utk-level-editor_bg.wasm dist/utk-level-editor_bg.wasm
#else
#  echo "warning: wasm-opt not found"
#fi
