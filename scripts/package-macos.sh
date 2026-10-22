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
if [ ! -f .local/fleet.toml ]; then cp config/fleet.example.toml .local/fleet.toml; fi
cp .local/fleet.toml "$app_dir/Contents/Resources/fleet.toml"
cat > "$app_dir/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>moonitor</string>
<key>CFBundleIdentifier</key><string>dev.maiomascio.moonitor</string>
<key>CFBundleName</key><string>moonitor</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.1.0</string>
<key>CFBundleVersion</key><string>1</string>
<key>NSHighResolutionCapable</key><true/>
<key>NSSupportsAutomaticGraphicsSwitching</key><true/>
</dict></plist>
PLIST
codesign --force --sign - "$app_dir"
printf '%s\n' "$app_dir"
