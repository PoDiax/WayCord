#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

cd "${REPO_DIR}"

VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
APPIMAGE_NAME="WayCord-v${VERSION}-x86_64.AppImage"
APPDIR="$(mktemp -d /tmp/waycord-appdir.XXXXXX)"

trap 'rm -rf "${APPDIR}"' EXIT

echo "Building WayCord release binary..."
cargo build --release --bin waycord

echo "Staging AppDir..."
mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/256x256/apps"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/scalable/apps"

cp "target/release/waycord" "${APPDIR}/usr/bin/"
chmod 755 "${APPDIR}/usr/bin/waycord"

if command -v strip >/dev/null 2>&1; then
    strip -s "${APPDIR}/usr/bin/waycord"
fi

cp "assets/waycord.desktop" "${APPDIR}/"
cp "assets/waycord.desktop" "${APPDIR}/usr/share/applications/"
cp "assets/waycord.png" "${APPDIR}/"
cp "assets/waycord.png" "${APPDIR}/usr/share/icons/hicolor/256x256/apps/"
cp "assets/waycord.svg" "${APPDIR}/usr/share/icons/hicolor/scalable/apps/"

cat << 'EOF' > "${APPDIR}/AppRun"
#!/bin/sh
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
exec "${HERE}/usr/bin/waycord" "$@"
EOF
chmod +x "${APPDIR}/AppRun"

TOOL=""
if command -v appimagetool >/dev/null 2>&1; then
    TOOL="appimagetool"
else
    CACHE_TOOL="/tmp/appimagetool"
    if [ ! -f "${CACHE_TOOL}" ]; then
        echo "⬇Downloading appimagetool..."
        curl -fsSL -o "${CACHE_TOOL}" https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
        chmod +x "${CACHE_TOOL}"
    fi
    TOOL="${CACHE_TOOL} --appimage-extract-and-run"
fi

echo "Generating ${APPIMAGE_NAME}..."
ARCH=x86_64 ${TOOL} "${APPDIR}" "${REPO_DIR}/${APPIMAGE_NAME}"

echo "Successfully built ${APPIMAGE_NAME}!"
