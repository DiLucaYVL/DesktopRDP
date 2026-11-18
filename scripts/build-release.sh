#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="1.2.1"
PKG_NAME="remote-desktop-linux"
DIST_DIR="$ROOT_DIR/dist"
STAGE_DIR="$(mktemp -d)"
trap 'rm -rf "$STAGE_DIR"' EXIT

rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# 1. Build .deb package
DEB_ROOT="$STAGE_DIR/${PKG_NAME}_${VERSION}_all"
mkdir -p \
  "$DEB_ROOT/DEBIAN" \
  "$DEB_ROOT/usr/bin" \
  "$DEB_ROOT/usr/share/applications" \
  "$DEB_ROOT/usr/share/icons/hicolor/scalable/apps" \
  "$DEB_ROOT/usr/share/doc/$PKG_NAME"

cat > "$DEB_ROOT/DEBIAN/control" <<EOF
Package: $PKG_NAME
Version: $VERSION
Section: net
Priority: optional
Architecture: all
Maintainer: DiLucaYVL <brunodiluca10@outlook.com>
Depends: python3, python3-gi, gir1.2-gtk-4.0, gir1.2-adw-1, gir1.2-secret-1, freerdp-x11 | freerdp3-x11 | freerdp-sdl | freerdp3-sdl
Recommends: tailscale
Homepage: https://github.com/DiLucaYVL/DesktopRDP
Description: Remote Desktop Linux - GTK4/Libadwaita FreeRDP & Tailscale client
 Simple native GTK4/Libadwaita remote desktop client (Area de trabalho remota)
 to connect to Windows machines via FreeRDP, Tailscale, or local IP, with
 secure password storage in GNOME Keyring, unified single-app window grouping,
 and automatic EN / PT-BR localization.
EOF

install -m 0755 "$ROOT_DIR/src/remote_desktop_linux.py" "$DEB_ROOT/usr/bin/$PKG_NAME"
ln -s "$PKG_NAME" "$DEB_ROOT/usr/bin/desktop-rdp"

sed "s|^Exec=remote-desktop-linux$|Exec=/usr/bin/$PKG_NAME|" \
  "$ROOT_DIR/data/io.github.dilucayvl.RemoteDesktopLinux.desktop" \
  > "$DEB_ROOT/usr/share/applications/io.github.dilucayvl.RemoteDesktopLinux.desktop"
chmod 0644 "$DEB_ROOT/usr/share/applications/io.github.dilucayvl.RemoteDesktopLinux.desktop"

install -m 0644 \
  "$ROOT_DIR/data/io.github.dilucayvl.RemoteDesktopLinux.svg" \
  "$DEB_ROOT/usr/share/icons/hicolor/scalable/apps/io.github.dilucayvl.RemoteDesktopLinux.svg"

install -m 0644 "$ROOT_DIR/README.md" "$DEB_ROOT/usr/share/doc/$PKG_NAME/README.md"
install -m 0644 "$ROOT_DIR/LICENSE" "$DEB_ROOT/usr/share/doc/$PKG_NAME/LICENSE"

chmod -R u+rwX,go+rX,go-w "$DEB_ROOT"
chmod 0755 "$DEB_ROOT/DEBIAN" "$DEB_ROOT/usr/bin/$PKG_NAME"
chmod 0644 "$DEB_ROOT/DEBIAN/control"

dpkg-deb --build --root-owner-group "$DEB_ROOT" "$DIST_DIR/${PKG_NAME}_${VERSION}_all.deb"

# 2. Build portable .tar.gz archive
TAR_ROOT="$STAGE_DIR/DesktopRDP-${VERSION}"
mkdir -p "$TAR_ROOT/src" "$TAR_ROOT/data"
install -m 0755 "$ROOT_DIR/src/remote_desktop_linux.py" "$TAR_ROOT/src/remote_desktop_linux.py"
install -m 0644 "$ROOT_DIR/data/io.github.dilucayvl.RemoteDesktopLinux.desktop" "$TAR_ROOT/data/io.github.dilucayvl.RemoteDesktopLinux.desktop"
install -m 0644 "$ROOT_DIR/data/io.github.dilucayvl.RemoteDesktopLinux.svg" "$TAR_ROOT/data/io.github.dilucayvl.RemoteDesktopLinux.svg"
install -m 0755 "$ROOT_DIR/install.sh" "$TAR_ROOT/install.sh"
install -m 0755 "$ROOT_DIR/uninstall.sh" "$TAR_ROOT/uninstall.sh"
install -m 0644 "$ROOT_DIR/README.md" "$TAR_ROOT/README.md"
install -m 0644 "$ROOT_DIR/LICENSE" "$TAR_ROOT/LICENSE"

tar -C "$STAGE_DIR" -czf "$DIST_DIR/${PKG_NAME}-${VERSION}-linux.tar.gz" "DesktopRDP-${VERSION}"

# 3. Generate SHA256SUMS
(
  cd "$DIST_DIR"
  sha256sum "${PKG_NAME}_${VERSION}_all.deb" "${PKG_NAME}-${VERSION}-linux.tar.gz" > SHA256SUMS
)

echo "Release artifacts built in $DIST_DIR:"
ls -lh "$DIST_DIR"
