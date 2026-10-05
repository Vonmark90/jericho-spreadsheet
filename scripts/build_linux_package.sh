#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

VERSION=$(grep -m1 '^version = ' "$PROJECT_ROOT/Cargo.toml" | cut -d '"' -f2)
ARCH="x86_64"
PKG_NAME="jericho-spreadsheet-${VERSION}-linux-${ARCH}"
DIST_DIR="$PROJECT_ROOT/target/dist/$PKG_NAME"
TARBALL="$PROJECT_ROOT/target/dist/${PKG_NAME}.tar.gz"

echo "=== Building Jericho Spreadsheet Linux Distribution Package v${VERSION} (${ARCH}) ==="

# Check binary existence
BIN_PATH="$PROJECT_ROOT/target/release/jericho_spreadsheet"
if [ ! -f "$BIN_PATH" ]; then
    echo "Release binary not found at $BIN_PATH. Building..."
    cd "$PROJECT_ROOT"
    cargo build --release
fi

echo "Creating distribution directory structure..."
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR/bin"
mkdir -p "$DIST_DIR/share/applications"
mkdir -p "$DIST_DIR/share/metainfo"
mkdir -p "$DIST_DIR/share/icons/hicolor/256x256/apps"
mkdir -p "$DIST_DIR/share/icons/hicolor/512x512/apps"

echo "Copying files..."
cp "$BIN_PATH" "$DIST_DIR/bin/jericho_spreadsheet"
chmod +x "$DIST_DIR/bin/jericho_spreadsheet"

cp "$PROJECT_ROOT/assets/jericho-spreadsheet.desktop" "$DIST_DIR/share/applications/"
cp "$PROJECT_ROOT/assets/com.jericho.spreadsheet.metainfo.xml" "$DIST_DIR/share/metainfo/"
cp "$PROJECT_ROOT/assets/icon-256.png" "$DIST_DIR/share/icons/hicolor/256x256/apps/jericho-spreadsheet.png"
cp "$PROJECT_ROOT/assets/icon.png" "$DIST_DIR/share/icons/hicolor/512x512/apps/jericho-spreadsheet.png"
cp "$PROJECT_ROOT/README.md" "$DIST_DIR/"

# Generate self-contained installer script inside package
cat << 'EOF' > "$DIST_DIR/install.sh"
#!/usr/bin/env bash
set -e

PREFIX="/usr/local"
if [ "${1:-}" = "--user" ] || [ "$(id -u)" -ne 0 ]; then
    PREFIX="${HOME}/.local"
    echo "Installing to user-local directory: $PREFIX"
else
    echo "Installing system-wide to: $PREFIX (requires root)"
fi

BIN_DIR="$PREFIX/bin"
APP_DIR="$PREFIX/share/applications"
META_DIR="$PREFIX/share/metainfo"
ICON256_DIR="$PREFIX/share/icons/hicolor/256x256/apps"
ICON512_DIR="$PREFIX/share/icons/hicolor/512x512/apps"

mkdir -p "$BIN_DIR" "$APP_DIR" "$META_DIR" "$ICON256_DIR" "$ICON512_DIR"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Installing binary..."
cp "$SCRIPT_DIR/bin/jericho_spreadsheet" "$BIN_DIR/"
chmod +x "$BIN_DIR/jericho_spreadsheet"

echo "Installing desktop launcher..."
cp "$SCRIPT_DIR/share/applications/jericho-spreadsheet.desktop" "$APP_DIR/"
cp "$SCRIPT_DIR/share/metainfo/com.jericho.spreadsheet.metainfo.xml" "$META_DIR/"

echo "Installing application icons..."
cp "$SCRIPT_DIR/share/icons/hicolor/256x256/apps/jericho-spreadsheet.png" "$ICON256_DIR/"
cp "$SCRIPT_DIR/share/icons/hicolor/512x512/apps/jericho-spreadsheet.png" "$ICON512_DIR/"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APP_DIR" || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "$PREFIX/share/icons/hicolor" 2>/dev/null || true
fi

echo ""
echo "Jericho Spreadsheet successfully installed!"
echo "Run 'jericho_spreadsheet' from your terminal or launch from your application menu."
EOF
chmod +x "$DIST_DIR/install.sh"

# Generate uninstaller
cat << 'EOF' > "$DIST_DIR/uninstall.sh"
#!/usr/bin/env bash
set -e

PREFIX="/usr/local"
if [ "${1:-}" = "--user" ] || [ "$(id -u)" -ne 0 ]; then
    PREFIX="${HOME}/.local"
fi

echo "Uninstalling Jericho Spreadsheet from $PREFIX..."
rm -f "$PREFIX/bin/jericho_spreadsheet"
rm -f "$PREFIX/share/applications/jericho-spreadsheet.desktop"
rm -f "$PREFIX/share/metainfo/com.jericho.spreadsheet.metainfo.xml"
rm -f "$PREFIX/share/icons/hicolor/256x256/apps/jericho-spreadsheet.png"
rm -f "$PREFIX/share/icons/hicolor/512x512/apps/jericho-spreadsheet.png"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$PREFIX/share/applications" 2>/dev/null || true
fi

echo "Jericho Spreadsheet successfully uninstalled."
EOF
chmod +x "$DIST_DIR/uninstall.sh"

echo "Compressing tarball..."
mkdir -p "$PROJECT_ROOT/target/dist"
tar -czf "$TARBALL" -C "$PROJECT_ROOT/target/dist" "$PKG_NAME"

echo "Tarball created: $TARBALL"
ls -lh "$TARBALL"
