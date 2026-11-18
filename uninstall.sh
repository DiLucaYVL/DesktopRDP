#!/usr/bin/env bash
set -euo pipefail

BIN_DIR="$HOME/.local/bin"
APP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"

LOCALE_STR="${LC_ALL:-${LC_MESSAGES:-${LANGUAGE:-${LANG:-}}}}"
IS_PT=false
if [[ "${LOCALE_STR,,}" == pt* ]]; then
  IS_PT=true
fi

rm -f \
  "$BIN_DIR/remote-desktop-linux" \
  "$BIN_DIR/desktop-rdp" \
  "$APP_DIR/io.github.dilucayvl.RemoteDesktopLinux.desktop" \
  "$ICON_DIR/io.github.dilucayvl.RemoteDesktopLinux.svg"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$APP_DIR" || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" || true
fi

if [ "$IS_PT" = true ]; then
  echo "Área de trabalho remota removido com sucesso."
else
  echo "Remote Desktop Linux uninstalled successfully."
fi
