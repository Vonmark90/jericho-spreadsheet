#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Building optimized release binary..."
cd "$PROJECT_ROOT"
cargo build --release

APP_NAME="Jericho Spreadsheet"
APP_DIR="$PROJECT_ROOT/target/bundle/$APP_NAME.app"
DESKTOP_DIR="$HOME/Desktop/$APP_NAME.app"

echo "Creating application bundle structure..."
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

echo "Copying binary..."
cp "$PROJECT_ROOT/target/release/jericho_spreadsheet" "$APP_DIR/Contents/MacOS/jericho_spreadsheet"
chmod +x "$APP_DIR/Contents/MacOS/jericho_spreadsheet"

echo "Copying icon..."
if [ -f "$PROJECT_ROOT/assets/AppIcon.icns" ]; then
    cp "$PROJECT_ROOT/assets/AppIcon.icns" "$APP_DIR/Contents/Resources/AppIcon.icns"
elif [ -f "/tmp/AppIcon.icns" ]; then
    cp "/tmp/AppIcon.icns" "$APP_DIR/Contents/Resources/AppIcon.icns"
fi

echo "Writing Info.plist..."
cat << 'EOF' > "$APP_DIR/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleDisplayName</key>
    <string>Jericho Spreadsheet</string>
    <key>CFBundleExecutable</key>
    <string>jericho_spreadsheet</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>com.jericho.spreadsheet</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>Jericho Spreadsheet</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>Excel Workbook</string>
            <key>CFBundleTypeRole</key>
            <string>Editor</string>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>xlsx</string>
                <string>xls</string>
            </array>
        </dict>
        <dict>
            <key>CFBundleTypeName</key>
            <string>CSV Document</string>
            <key>CFBundleTypeRole</key>
            <string>Editor</string>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>csv</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
EOF

echo "Signing application bundle ad-hoc..."
codesign --force --deep --sign - "$APP_DIR" || true

echo "Deploying to Desktop..."
rm -rf "$DESKTOP_DIR"
cp -R "$APP_DIR" "$DESKTOP_DIR"

# Touch to notify Finder of the new app bundle
touch "$DESKTOP_DIR"

echo "Jericho Spreadsheet.app successfully installed to $DESKTOP_DIR!"
