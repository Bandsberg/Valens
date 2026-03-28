#!/usr/bin/env bash
set -euo pipefail

# Build, bundle, and install Valens.app to /Applications/.
# Usage: bash scripts/bundle.sh [--no-build]
#   --no-build   skip cargo build (use existing target/release/valens)

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

NO_BUILD=false
if [[ "${1:-}" == "--no-build" ]]; then
  NO_BUILD=true
fi

# ── Read version from Cargo.toml ──────────────────────────────────────────────
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
echo "→ Version: $VERSION"

# ── Build ─────────────────────────────────────────────────────────────────────
if [[ "$NO_BUILD" == false ]]; then
  echo "→ Building release binary..."
  cargo build --release
fi

BINARY="target/release/valens"
if [[ ! -f "$BINARY" ]]; then
  echo "Error: binary not found at $BINARY" >&2
  exit 1
fi

# ── Create .app structure ─────────────────────────────────────────────────────
APP="build/Valens.app"
CONTENTS="$APP/Contents"
MACOS="$CONTENTS/MacOS"
RESOURCES="$CONTENTS/Resources"

rm -rf "$APP"
mkdir -p "$MACOS" "$RESOURCES"
cp "$BINARY" "$MACOS/valens"

# ── Generate AppIcon.icns from assets/icon-1024.png ──────────────────────────
ICONSET="build/AppIcon.iconset"
rm -rf "$ICONSET"
mkdir -p "$ICONSET"

sips -z 16   16   assets/icon-1024.png --out "$ICONSET/icon_16x16.png"      >/dev/null
sips -z 32   32   assets/icon-1024.png --out "$ICONSET/icon_16x16@2x.png"   >/dev/null
sips -z 32   32   assets/icon-1024.png --out "$ICONSET/icon_32x32.png"      >/dev/null
sips -z 64   64   assets/icon-1024.png --out "$ICONSET/icon_32x32@2x.png"   >/dev/null
sips -z 128  128  assets/icon-1024.png --out "$ICONSET/icon_128x128.png"    >/dev/null
sips -z 256  256  assets/icon-1024.png --out "$ICONSET/icon_128x128@2x.png" >/dev/null
sips -z 256  256  assets/icon-1024.png --out "$ICONSET/icon_256x256.png"    >/dev/null
sips -z 512  512  assets/icon-1024.png --out "$ICONSET/icon_256x256@2x.png" >/dev/null
sips -z 512  512  assets/icon-1024.png --out "$ICONSET/icon_512x512.png"    >/dev/null
cp assets/icon-1024.png "$ICONSET/icon_512x512@2x.png"

iconutil -c icns "$ICONSET" -o "$RESOURCES/AppIcon.icns"
rm -rf "$ICONSET"

# ── Write Info.plist ──────────────────────────────────────────────────────────
cat > "$CONTENTS/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>com.jesbp.valens</string>
    <key>CFBundleName</key>
    <string>Valens</string>
    <key>CFBundleDisplayName</key>
    <string>Valens</string>
    <key>CFBundleExecutable</key>
    <string>valens</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
</dict>
</plist>
EOF

# ── Ad-hoc code sign ──────────────────────────────────────────────────────────
echo "→ Signing (ad-hoc)..."
codesign --force --deep --sign - "$APP"

# ── Install to /Applications ──────────────────────────────────────────────────
echo "→ Installing to /Applications/Valens.app..."
if [[ -d "/Applications/Valens.app" ]]; then
  BACKUP="/Applications/Valens.app.bak"
  rm -rf "$BACKUP"
  mv "/Applications/Valens.app" "$BACKUP"
  echo "  (Previous version backed up to /Applications/Valens.app.bak)"
fi
cp -r "$APP" "/Applications/Valens.app"

echo "✓ Valens $VERSION installed."
echo "  Data is safe at: ~/Library/Application Support/valens/"
