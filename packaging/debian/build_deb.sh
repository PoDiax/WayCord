#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

cd "${REPO_DIR}"

VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
DEB_NAME="waycord_${VERSION}_amd64.deb"
STAGE_DIR="$(mktemp -d /tmp/waycord-deb-stage.XXXXXX)"

trap 'rm -rf "${STAGE_DIR}"' EXIT

echo "Building WayCord release binary..."
cargo build --release --bin waycord

echo "Staging Debian package layout..."
mkdir -p "${STAGE_DIR}/DEBIAN"
mkdir -p "${STAGE_DIR}/usr/bin"
mkdir -p "${STAGE_DIR}/usr/share/applications"
mkdir -p "${STAGE_DIR}/usr/share/icons/hicolor/256x256/apps"
mkdir -p "${STAGE_DIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${STAGE_DIR}/usr/share/doc/waycord"

cp "target/release/waycord" "${STAGE_DIR}/usr/bin/"
chmod 755 "${STAGE_DIR}/usr/bin/waycord"

if command -v strip >/dev/null 2>&1; then
    strip -s "${STAGE_DIR}/usr/bin/waycord"
fi

cp "assets/waycord.desktop" "${STAGE_DIR}/usr/share/applications/"
chmod 644 "${STAGE_DIR}/usr/share/applications/waycord.desktop"

cp "assets/waycord.png" "${STAGE_DIR}/usr/share/icons/hicolor/256x256/apps/"
chmod 644 "${STAGE_DIR}/usr/share/icons/hicolor/256x256/apps/waycord.png"

cp "assets/waycord.svg" "${STAGE_DIR}/usr/share/icons/hicolor/scalable/apps/"
chmod 644 "${STAGE_DIR}/usr/share/icons/hicolor/scalable/apps/waycord.svg"

cp "README.md" "${STAGE_DIR}/usr/share/doc/waycord/"
cp "LICENSE" "${STAGE_DIR}/usr/share/doc/waycord/copyright"

cat <<EOF > "${STAGE_DIR}/DEBIAN/control"
Package: waycord
Version: ${VERSION}
Section: net
Priority: optional
Architecture: amd64
Maintainer: PoDiax <pd@pdx.ovh>
Depends: libx11-6, libfontconfig1, libssl3 | libssl1.1
Description: Lightweight Discord voice overlay for Linux
 WayCord is a clean, GPU-accelerated Discord voice overlay for Linux and
 Wayland compositors (Hyprland, Sway, KDE, GNOME, etc.).
EOF

echo "Creating ${DEB_NAME}..."
if command -v dpkg-deb >/dev/null 2>&1; then
    dpkg-deb --build --root-owner-group "${STAGE_DIR}" "${REPO_DIR}/${DEB_NAME}"
else
    ARCH_DIR="$(mktemp -d /tmp/waycord-deb-arch.XXXXXX)"
    trap 'rm -rf "${STAGE_DIR}" "${ARCH_DIR}"' EXIT
    echo "2.0" > "${ARCH_DIR}/debian-binary"
    tar --owner=0 --group=0 -czf "${ARCH_DIR}/control.tar.gz" -C "${STAGE_DIR}/DEBIAN" control
    tar --owner=0 --group=0 -czf "${ARCH_DIR}/data.tar.gz" -C "${STAGE_DIR}" usr
    ar rcs "${REPO_DIR}/${DEB_NAME}" "${ARCH_DIR}/debian-binary" "${ARCH_DIR}/control.tar.gz" "${ARCH_DIR}/data.tar.gz"
fi

echo "Successfully built ${DEB_NAME}!"
