#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"
PROJECT_ROOT="$PWD"
if ! command -v cargo >/dev/null 2>&1 && [ -x ../.toolchain/cargo/bin/cargo ]; then
  export CARGO_HOME="$(cd ../.toolchain/cargo && pwd)"
  export RUSTUP_HOME="$(cd ../.toolchain/rustup && pwd)"
  export PATH="$CARGO_HOME/bin:$PATH"
fi
# Prefer the SDK selected by Command Line Tools instead of an unrelated beta SDK.
if [ -z "${SDKROOT:-}" ]; then
  if [ -d "$(xcode-select -p)/SDKs/MacOSX.sdk" ]; then
    export SDKROOT="$(xcode-select -p)/SDKs/MacOSX.sdk"
  else
    export SDKROOT="$(xcrun --sdk macosx --show-sdk-path)"
  fi
fi
export MACOSX_DEPLOYMENT_TARGET=11.0
ARCH="$(uname -m)"
case "$ARCH" in arm64) TARGET=aarch64-apple-darwin ;; x86_64) TARGET=x86_64-apple-darwin ;; *) exit 1 ;; esac
cargo build --manifest-path src-tauri/Cargo.toml --release --locked
OUT="$PROJECT_ROOT/../dist"
APP="$OUT/PhuketBuddy.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp src-tauri/target/release/phuketbuddy "$APP/Contents/MacOS/PhuketBuddy"
cp src-tauri/icons/icon.icns "$APP/Contents/Resources/icon.icns"
cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>PhuketBuddy</string>
<key>CFBundleIdentifier</key><string>com.phuketbuddy.desktop</string>
<key>CFBundleName</key><string>PhuketBuddy</string>
<key>CFBundleDisplayName</key><string>PhuketBuddy</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.1.0</string>
<key>CFBundleVersion</key><string>1</string>
<key>CFBundleIconFile</key><string>icon.icns</string>
<key>LSMinimumSystemVersion</key><string>11.0</string>
<key>LSUIElement</key><true/>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
codesign --force --deep --sign "${MACOS_SIGNING_IDENTITY:--}" "$APP"
codesign --verify --deep --strict "$APP"
cp README-macOS.md "$OUT/使用说明.md"
ditto -c -k --sequesterRsrc --keepParent "$APP" "$OUT/PhuketBuddy-v0.1.0-macOS-$ARCH.zip"
printf 'Built: %s\n' "$APP"
