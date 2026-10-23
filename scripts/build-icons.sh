#!/bin/sh
# Deterministic size conversion of the supplied artwork; original pixels retained.
set -eu
cd "$(dirname "$0")/.."
source_icon=assets/brand/moon-original.png
iconset_dir=.local/moonitor.iconset
mkdir -p "$iconset_dir"
for size in 16 32 128 256 512; do
  sips -z "$size" "$size" "$source_icon" --out "$iconset_dir/icon_${size}x${size}.png" >/dev/null
  retina=$((size * 2))
  sips -z "$retina" "$retina" "$source_icon" --out "$iconset_dir/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$iconset_dir" -o .local/moonitor.icns
