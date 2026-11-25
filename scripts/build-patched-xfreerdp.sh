#!/usr/bin/env bash
# Rebuilds xfreerdp with patches/freerdp/*.patch against the FreeRDP 3 libraries already
# installed on the system, and installs it where install.sh puts the bundled client.
#
# Why: FreeRDP's X11 client drops its pending clipboard requests when the remote clipboard
# changes, but still matches the late answer to the next request. A CF_TEXT answer read as
# CF_UNICODETEXT pastes as Chinese characters until the next copy. See
# patches/freerdp/xf_cliprdr-stale-response.patch.
#
# No root needed: the -dev packages are downloaded and unpacked in a temporary folder.
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LIB_DIR="$HOME/.local/lib/remote-desktop-linux/usr/bin"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# Build for the FreeRDP version the system libraries come from.
VERSION="$(dpkg-query -W -f='${Version}' libfreerdp3-3 | sed 's/[+~-].*//')"
echo "FreeRDP $VERSION"

mkdir -p "$WORK/debs" "$WORK/sysroot" "$WORK/lib"
(cd "$WORK/debs" && apt-get download freerdp3-dev libwinpr3-dev libx11-dev libxext-dev \
  libxinerama-dev libxcursor-dev libxi-dev libxrender-dev libxrandr-dev libxfixes-dev \
  libxkbfile-dev x11proto-dev libxcb1-dev libxau-dev libxdmcp-dev >/dev/null)
for deb in "$WORK"/debs/*.deb; do dpkg-deb -x "$deb" "$WORK/sysroot"; done

curl -sfL "https://github.com/FreeRDP/FreeRDP/archive/refs/tags/$VERSION.tar.gz" | tar xz -C "$WORK"
SRC="$WORK/FreeRDP-$VERSION"
for p in "$REPO_DIR"/patches/freerdp/*.patch; do patch -d "$SRC" -p1 < "$p"; done

# Link against the runtime libraries (the -dev .so symlinks are not installed).
for l in freerdp3 freerdp-client3 winpr3; do ln -sf "/usr/lib/x86_64-linux-gnu/lib$l.so.3" "$WORK/lib/lib$l.so"; done
for l in X11 Xext Xinerama Xcursor Xi Xrender Xrandr Xfixes xkbfile; do
  ln -sf "$(ls /usr/lib/x86_64-linux-gnu/lib$l.so.* | head -1)" "$WORK/lib/lib$l.so"
done

R="$WORK/sysroot/usr"
cd "$SRC/client/X11"
gcc -O2 -fPIE -pie -std=gnu11 -w -DNDEBUG -D_FILE_OFFSET_BITS=64 -fstack-protector-strong \
  -D_FORTIFY_SOURCE=2 -Wl,-z,relro,-z,now -s \
  -DWITH_XINERAMA -DWITH_XEXT -DWITH_XCURSOR -DWITH_XI -DWITH_XRENDER -DWITH_XRANDR -DWITH_XFIXES \
  -I. -I"$SRC/resources" -I"$R/include/freerdp3" -I"$R/include/winpr3" -I"$R/include" \
  xf_utils.c xf_x11_utils.c xf_gfx.c xf_rail.c xf_input.c xf_event.c xf_floatbar.c \
  xf_channels.c xf_cliprdr.c xf_reconnect.c xf_monitor.c xf_disp.c xf_graphics.c \
  xf_keyboard.c keyboard_x11.c xkb_layout_ids.c xf_video.c xf_window.c xf_client.c cli/xfreerdp.c \
  -o "$WORK/xfreerdp" -L"$WORK/lib" -lfreerdp-client3 -lfreerdp3 -lwinpr3 -lX11 -lXext \
  -lXinerama -lXcursor -lXi -lXrender -lXrandr -lXfixes -lxkbfile -lm -lpthread

"$WORK/xfreerdp" --version
mkdir -p "$LIB_DIR"
install -m 0755 "$WORK/xfreerdp" "$LIB_DIR/xfreerdp.new"
mv -f "$LIB_DIR/xfreerdp.new" "$LIB_DIR/xfreerdp"
ln -sfn "$LIB_DIR/xfreerdp" "$HOME/.local/bin/xfreerdp"
echo "Patched xfreerdp installed in $LIB_DIR"
