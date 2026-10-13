#!/usr/bin/env bash
# Builds the WebAssembly module and its JavaScript bindings into
# web/src/generated/. Requires the wasm32-unknown-unknown target,
# wasm-bindgen-cli (the exact version in Cargo.lock) and wasm-opt (binaryen).
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="$root/web/src/generated"

# wasm-bindgen-cli must match the wasm-bindgen crate exactly.
locked="$(grep -A1 '^name = "wasm-bindgen"$' "$root/Cargo.lock" | sed -n 's/^version = "\(.*\)"$/\1/p')"
installed="$(wasm-bindgen --version | awk '{print $2}')"
if [[ "$locked" != "$installed" ]]; then
  echo "error: wasm-bindgen-cli $installed does not match wasm-bindgen $locked in Cargo.lock" >&2
  echo "install it with: cargo install wasm-bindgen-cli --version '=$locked' --locked" >&2
  exit 1
fi

cargo build \
  --manifest-path "$root/Cargo.toml" \
  --package cheme-unit-converter-wasm \
  --target wasm32-unknown-unknown \
  --profile wasm-release

target_dir="$(cargo metadata --manifest-path "$root/Cargo.toml" --format-version 1 --no-deps |
  sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')"

rm -rf "$out"
wasm-bindgen \
  --target web \
  --out-dir "$out" \
  --out-name converter \
  "$target_dir/wasm32-unknown-unknown/wasm-release/cheme_unit_converter_wasm.wasm"

wasm-opt -Oz \
  --enable-bulk-memory \
  --enable-nontrapping-float-to-int \
  --enable-sign-ext \
  --enable-mutable-globals \
  --enable-reference-types \
  --enable-multivalue \
  "$out/converter_bg.wasm" -o "$out/converter_bg.wasm"

echo "built $out/converter_bg.wasm ($(wc -c <"$out/converter_bg.wasm" | tr -d ' ') bytes)"
