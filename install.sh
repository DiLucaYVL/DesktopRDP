#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$HOME/.local/bin"
LIB_DIR="$HOME/.local/lib/remote-desktop-linux"
APP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"

TARGET_BIN="$BIN_DIR/remote-desktop-linux"
TARGET_LINK="$BIN_DIR/desktop-rdp"
TARGET_DESKTOP="$APP_DIR/io.github.dilucayvl.RemoteDesktopLinux.desktop"
TARGET_ICON="$ICON_DIR/io.github.dilucayvl.RemoteDesktopLinux.svg"

LOCALE_STR="${LC_ALL:-${LC_MESSAGES:-${LANGUAGE:-${LANG:-}}}}"
IS_PT=false
if [[ "${LOCALE_STR,,}" == pt* ]]; then
  IS_PT=true
fi

mkdir -p "$BIN_DIR" "$APP_DIR" "$ICON_DIR"

install -m 0755 "$SCRIPT_DIR/src/remote_desktop_linux.py" "$TARGET_BIN"
ln -sfn "remote-desktop-linux" "$TARGET_LINK"

install -m 0644 "$SCRIPT_DIR/data/io.github.dilucayvl.RemoteDesktopLinux.svg" "$TARGET_ICON"

sed "s|^Exec=remote-desktop-linux$|Exec=$TARGET_BIN|" \
  "$SCRIPT_DIR/data/io.github.dilucayvl.RemoteDesktopLinux.desktop" > "$TARGET_DESKTOP"
chmod 0644 "$TARGET_DESKTOP"

# Prefer native xfreerdp (avoids Flatpak creating a separate Dock/Alt-Tab app)
if [ ! -x /usr/bin/xfreerdp ] && [ ! -x /usr/bin/xfreerdp3 ]; then
  if [ ! -x "$LIB_DIR/usr/bin/xfreerdp" ] && [ ! -x "$LIB_DIR/usr/bin/xfreerdp3" ]; then
    if command -v apt-get >/dev/null 2>&1 && command -v dpkg-deb >/dev/null 2>&1; then
      if [ "$IS_PT" = true ]; then
        echo "Baixando freerdp-x11 nativo para integração em janela única..."
      else
        echo "Downloading native freerdp-x11 for single-app window integration..."
      fi
      TMP_DEB_DIR="$(mktemp -d)"
      if (cd "$TMP_DEB_DIR" && apt-get download freerdp-x11 >/dev/null 2>&1); then
        mkdir -p "$LIB_DIR"
        for deb in "$TMP_DEB_DIR"/freerdp-x11*.deb; do
          [ -f "$deb" ] && dpkg-deb -x "$deb" "$LIB_DIR"
        done
      fi
      rm -rf "$TMP_DEB_DIR"
    fi
  fi
  if [ -x "$LIB_DIR/usr/bin/xfreerdp" ]; then
    ln -sfn "$LIB_DIR/usr/bin/xfreerdp" "$BIN_DIR/xfreerdp"
  elif [ -x "$LIB_DIR/usr/bin/xfreerdp3" ]; then
    ln -sfn "$LIB_DIR/usr/bin/xfreerdp3" "$BIN_DIR/xfreerdp"
  fi
fi

# Fallback to Flatpak only if no native xfreerdp/sdl-freerdp is available
if [ ! -x "$BIN_DIR/xfreerdp" ] && ! command -v xfreerdp >/dev/null 2>&1 && ! command -v xfreerdp3 >/dev/null 2>&1 && ! command -v sdl-freerdp >/dev/null 2>&1; then
  if command -v flatpak >/dev/null 2>&1; then
    if ! flatpak info com.freerdp.FreeRDP >/dev/null 2>&1; then
      if [ "$IS_PT" = true ]; then
        echo "Instalando FreeRDP via Flatpak..."
      else
        echo "Installing FreeRDP via Flatpak..."
      fi
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

if [ "$IS_PT" = true ]; then
  echo "Área de trabalho remota instalado com sucesso!"
  echo "Execute '$TARGET_BIN' ou procure por 'Área de trabalho remota' no menu de aplicativos."
else
  echo "Remote Desktop Linux installed successfully!"
  echo "Run '$TARGET_BIN' or search for 'Remote Desktop Linux' in your application menu."
fi
