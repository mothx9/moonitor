#!/bin/sh
set -eu
repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
if [ "$(uname -s)" = Darwin ]; then
  "$repo_dir/scripts/package-macos.sh"
  open "$repo_dir/.local/moonitor.app"
else
  cd "$repo_dir"
  exec cargo run --release --locked -p moonitor-desktop -- --config config/fleet.example.toml
fi
