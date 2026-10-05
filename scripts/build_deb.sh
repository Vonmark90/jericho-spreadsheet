#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

VERSION=$(grep -m1 '^version = ' "$PROJECT_ROOT/Cargo.toml" | cut -d '"' -f2)
ARCH="amd64"
DEB_NAME="jericho-spreadsheet_${VERSION}_${ARCH}"
DEB_DIR="$PROJECT_ROOT/target/dist/$DEB_NAME"
DEB_FILE="$PROJECT_ROOT/target/dist/${DEB_NAME}.deb"

echo "=== Building Debian/Ubuntu (.deb) Package v${VERSION} (${ARCH}) ==="

BIN_PATH="$PROJECT_ROOT/target/release/jericho_spreadsheet"
if [ ! -f "$BIN_PATH" ]; then
    echo "Release binary not found at $BIN_PATH. Building..."
    cd "$PROJECT_ROOT"
    cargo build --release
fi

echo "Creating Debian package structure..."
rm -rf "$DEB_DIR"
mkdir -p "$DEB_DIR/DEBIAN"
mkdir -p "$DEB_DIR/usr/bin"
mkdir -p "$DEB_DIR/usr/share/applications"
mkdir -p "$DEB_DIR/usr/share/metainfo"
mkdir -p "$DEB_DIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$DEB_DIR/usr/share/icons/hicolor/512x512/apps"
mkdir -p "$DEB_DIR/usr/share/doc/jericho-spreadsheet"

# Copy binary & assets
cp "$BIN_PATH" "$DEB_DIR/usr/bin/jericho_spreadsheet"
chmod 755 "$DEB_DIR/usr/bin/jericho_spreadsheet"

cp "$PROJECT_ROOT/assets/jericho-spreadsheet.desktop" "$DEB_DIR/usr/share/applications/"
cp "$PROJECT_ROOT/assets/com.jericho.spreadsheet.metainfo.xml" "$DEB_DIR/usr/share/metainfo/"
cp "$PROJECT_ROOT/assets/icon-256.png" "$DEB_DIR/usr/share/icons/hicolor/256x256/apps/jericho-spreadsheet.png"
cp "$PROJECT_ROOT/assets/icon.png" "$DEB_DIR/usr/share/icons/hicolor/512x512/apps/jericho-spreadsheet.png"
cp "$PROJECT_ROOT/README.md" "$DEB_DIR/usr/share/doc/jericho-spreadsheet/"

# Calculate installed size in KB
INSTALLED_SIZE=$(du -sk "$DEB_DIR" | cut -f1)

# Write DEBIAN/control
cat << EOF > "$DEB_DIR/DEBIAN/control"
Package: jericho-spreadsheet
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Depends: libfontconfig1, libx11-6, libxcursor1, libxrandr2, libxi6, libxinerama1, libxkbcommon0
Maintainer: Jericho Spreadsheet Team <team@jericho-spreadsheet.org>
Installed-Size: ${INSTALLED_SIZE}
Description: Fast, lightweight desktop spreadsheet with native Excel compatibility
 Jericho Spreadsheet is an original, ultra-optimized spreadsheet application
 built from scratch in Rust. It features a 125+ Excel function formula evaluation
 engine (financial modeling, statistics, multi-criteria lookup, math), sparse memory
 model (< 35 MB RAM), 60 FPS virtualized grid, and native .xlsx and .csv compatibility.
EOF

# Post-install & Post-remove hooks
cat << 'EOF' > "$DEB_DIR/DEBIAN/postinst"
#!/bin/sh
set -e
if [ "$1" = "configure" ]; then
    if which update-desktop-database >/dev/null 2>&1; then
        update-desktop-database -q /usr/share/applications || true
    fi
    if which gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor 2>/dev/null || true
    fi
fi
exit 0
EOF
chmod 755 "$DEB_DIR/DEBIAN/postinst"

cat << 'EOF' > "$DEB_DIR/DEBIAN/postrm"
#!/bin/sh
set -e
if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
    if which update-desktop-database >/dev/null 2>&1; then
        update-desktop-database -q /usr/share/applications || true
    fi
    if which gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor 2>/dev/null || true
    fi
fi
exit 0
EOF
chmod 755 "$DEB_DIR/DEBIAN/postrm"

if command -v dpkg-deb >/dev/null 2>&1; then
    echo "Building Debian package with dpkg-deb..."
    dpkg-deb --build "$DEB_DIR" "$DEB_FILE"
    echo "Debian package created: $DEB_FILE"
    ls -lh "$DEB_FILE"
else
    echo "Note: 'dpkg-deb' not installed locally. Debian tree created at: $DEB_DIR"
    echo "Run 'dpkg-deb --build \"$DEB_DIR\" \"$DEB_FILE\"' on Debian/Ubuntu or in Docker."
fi
