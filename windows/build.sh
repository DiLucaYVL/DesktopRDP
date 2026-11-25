#!/usr/bin/env bash
# Builds the Windows client (DesktopRDP.exe, Rust + egui) and its MSI installer on Linux.
#
# Needs:
#   - Rust (rustup target add x86_64-pc-windows-gnu), zig and cargo-zigbuild (cross linker)
#   - wixl from msitools (sudo apt install wixl), or WIXL=/path/to/wixl
#   - go-winres for the exe icon/manifest (go install github.com/tc-hib/go-winres@v0.3.3), or a
#     committed winres/rsrc_windows_amd64.syso
# Output: windows/dist/DesktopRDP-<version>-x64.msi and SHA256SUMS-windows
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
TARGET=x86_64-pc-windows-gnu
WIXL="${WIXL:-wixl}"
mkdir -p dist

# Windows resources: icon, version info, manifest (visual styles, per-monitor DPI).
if command -v go-winres >/dev/null 2>&1; then
  go-winres simply --arch amd64 --icon winres/icon.png --manifest gui \
    --product-name DesktopRDP --file-description "Área de trabalho remota / Remote Desktop" \
    --product-version "$VERSION.0" --file-version "$VERSION.0" \
    --original-filename DesktopRDP.exe --copyright "MIT License" --out winres/rsrc
fi

cargo zigbuild --release --target "$TARGET"
EXE="target/$TARGET/release/DesktopRDP.exe"

MSI="DesktopRDP-$VERSION-x64.msi"
(cd installer && "$WIXL" -a x64 -D "Version=$VERSION" -D "ExePath=../$EXE" -o "../dist/$MSI" DesktopRDP.wxs)

(cd dist && sha256sum "$MSI" > SHA256SUMS-windows)
ls -la dist
