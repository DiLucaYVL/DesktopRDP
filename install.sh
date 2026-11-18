#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$HOME/.local/bin"
APP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"

TARGET_BIN="$BIN_DIR/remote-desktop-linux"
TARGET_LINK="$BIN_DIR/desktop-rdp"
TARGET_DESKTOP="$APP_DIR/io.github.dilucayvl.RemoteDesktopLinux.desktop"
TARGET_ICON="$ICON_DIR/io.github.dilucayvl.RemoteDesktopLinux.svg"

mkdir -p "$BIN_DIR" "$APP_DIR" "$ICON_DIR"

install -m 0755 "$SCRIPT_DIR/src/remote_desktop_linux.py" "$TARGET_BIN"
ln -sfn "remote-desktop-linux" "$TARGET_LINK"

install -m 0644 "$SCRIPT_DIR/data/io.github.dilucayvl.RemoteDesktopLinux.svg" "$TARGET_ICON"

sed "s|^Exec=remote-desktop-linux$|Exec=$TARGET_BIN|" \
  "$SCRIPT_DIR/data/io.github.dilucayvl.RemoteDesktopLinux.desktop" > "$TARGET_DESKTOP"
chmod 0644 "$TARGET_DESKTOP"

if ! command -v sdl-freerdp >/dev/null 2>&1 && ! command -v xfreerdp >/dev/null 2>&1; then
  if command -v flatpak >/dev/null 2>&1; then
    if ! flatpak info com.freerdp.FreeRDP >/dev/null 2>&1; then
      echo "Instalando FreeRDP via Flatpak..."
      flatpak install --user -y flathub com.freerdp.FreeRDP || true
    fi
    if flatpak info com.freerdp.FreeRDP >/dev/null 2>&1; then
      cat > "$BIN_DIR/sdl-freerdp" <<'WRAPPER'
#!/usr/bin/env bash
if [ -x /usr/bin/sdl-freerdp ]; then
  exec /usr/bin/sdl-freerdp "$@"
fi
exec flatpak run com.freerdp.FreeRDP "$@"
WRAPPER
      chmod 0755 "$BIN_DIR/sdl-freerdp"
    fi
  fi
fi

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$APP_DIR" || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" || true
fi

echo "Remote Desktop Linux (Área de trabalho remota) instalado com sucesso!"
echo "Execute '$TARGET_BIN' ou procure por 'Remote Desktop Linux' no menu de aplicativos."
