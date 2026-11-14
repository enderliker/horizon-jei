#!/usr/bin/env bash
# Format, lint, test, compile the game with the mod, and package it.
# HORIZON_MODS points at a clone of https://github.com/horizon-rust/horizon-mods.
set -euo pipefail

mod="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tools="${HORIZON_MODS:-/home/ender/JEI-Horizon/horizon-ref/horizon-mods}"
py="$tools/native/tools/native.py"

# A local ALSA sysroot (docs/dev.md), when the image has no libasound2-dev and no sudo.
sysroot="${HORIZON_SYSROOT:-/home/ender/horizon-tools/sysroot}"
if [ -d "$sysroot/usr/lib/x86_64-linux-gnu/pkgconfig" ]; then
  export PKG_CONFIG_PATH="$sysroot/usr/lib/x86_64-linux-gnu/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
fi

cd "$mod"

echo "== fmt"
cargo fmt --check
rustfmt --edition 2024 --check mixins/jei/*.rs tests/*.rs

echo "== clippy"
cargo clippy --all-targets -- -D warnings

echo "== test"
cargo test

echo "== check (game + mod)"
python3 "$py" check "$mod"

echo "== package"
out="$(python3 "$py" package "$mod")"
file="$(python3 -c 'import json,sys; print(json.load(sys.stdin)["file"])' <<<"$out")"
echo "$out"

if command -v cargo-horizon-mod >/dev/null 2>&1; then
  echo "== registry check"
  cargo horizon-mod check "$file"
else
  echo "cargo-horizon-mod not installed; skipping the registry check" >&2
  echo "  cargo install --git https://github.com/horizon-rust/horizon cargo-horizon-mod" >&2
fi