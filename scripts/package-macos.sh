#!/bin/sh
set -eu
repo_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_dir"
if [ "$(uname -s)" != Darwin ]; then
  echo 'This packaging script requires macOS.' >&2
  exit 1
fi
if [ "${1:-}" != --no-build ]; then
  cargo build --release --locked -p moonitor-desktop -p moonitor-agent
fi
app_dir="$repo_dir/.local/moonitor.app"
mkdir -p "$app_dir/Contents/MacOS" "$app_dir/Contents/Resources"
cp target/release/moonitor "$app_dir/Contents/MacOS/moonitor"
scripts/build-icons.sh
cp .local/moonitor.icns "$app_dir/Contents/Resources/moonitor.icns"
if [ ! -f .local/fleet.toml ]; then cp config/fleet.example.toml .local/fleet.toml; fi
config_dir="$HOME/Library/Application Support/moonitor"
mkdir -p "$config_dir"
# Preserve an existing user configuration. Never embed private fleet data in a bundle.
if [ ! -e "$config_dir/fleet.toml" ]; then
  cp .local/fleet.toml "$config_dir/fleet.toml"
  chmod 600 "$config_dir/fleet.toml"
fi
if [ -f "$app_dir/Contents/Resources/fleet.toml" ]; then
  backup_dir="$repo_dir/.local/checkpoints/$(date -u +%Y%m%dT%H%M%SZ)-bundle-config"
  mkdir -p "$backup_dir"
  cp "$app_dir/Contents/Resources/fleet.toml" "$backup_dir/fleet.toml"
  rm "$app_dir/Contents/Resources/fleet.toml"
fi
python3 scripts/build-manifest.py "$app_dir/Contents/Resources/build-manifest.json"
cat > "$app_dir/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>moonitor</string>
<key>CFBundleIdentifier</key><string>dev.maiomascio.moonitor</string>
<key>CFBundleName</key><string>moonitor</string>
<key>CFBundleIconFile</key><string>moonitor.icns</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.1.0</string>
<key>CFBundleVersion</key><string>1</string>
<key>NSHighResolutionCapable</key><true/>
<key>NSSupportsAutomaticGraphicsSwitching</key><true/>
</dict></plist>
PLIST
codesign --force --sign - "$app_dir"
codesign --verify --deep --strict "$app_dir"
python3 scripts/build-receipt.py "$app_dir" .local/moonitor-build-receipt.json
printf '%s\n' "$app_dir"
