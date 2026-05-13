#!/usr/bin/env bash
# Bundle a built malscraper binary into a macOS .app.
#
# Usage:
#   ./scripts/bundle-macos.sh <path/to/malscraper-binary> [output-dir]
#
# If output-dir is omitted, the bundle is written to ./dist/malScraper.app
# next to the rust crate root.
#
# Produces:
#   <output-dir>/malScraper.app/
#       Contents/Info.plist
#       Contents/MacOS/malScraper        (launcher shim, CFBundleExecutable)
#       Contents/MacOS/malscraper-bin    (the actual rust binary)
#       Contents/Resources/AppIcon.icns
#
# (The inner binary is named malscraper-bin to avoid colliding with the
# launcher shim on macOS's default case-insensitive filesystem.)
#
# The bundle is ad-hoc codesigned (`codesign -s -`) so it launches on
# Apple Silicon without "killed" errors. It is NOT notarized; first launch
# from Finder will show a Gatekeeper warning that the user can dismiss with
# right-click → Open.

set -euo pipefail

CYAN='\033[1;36m'
GREEN='\033[1;32m'
YELLOW='\033[1;33m'
RED='\033[1;31m'
RESET='\033[0m'

if [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ] || [ -z "${1:-}" ]; then
    sed -n '2,21p' "$0"
    exit 0
fi

if [ "$(uname -s)" != "Darwin" ]; then
    echo -e "${RED}Error: bundle-macos.sh must run on macOS (uname is $(uname -s))${RESET}" >&2
    exit 1
fi

BINARY="$1"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RUST_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUTPUT_DIR="${2:-${RUST_DIR}/dist}"

if [ ! -f "${BINARY}" ]; then
    echo -e "${RED}Error: binary not found: ${BINARY}${RESET}" >&2
    exit 1
fi
if [ ! -x "${BINARY}" ]; then
    chmod +x "${BINARY}"
fi

VERSION="$(grep -m1 '^version' "${RUST_DIR}/Cargo.toml" | sed -E 's/version = "(.*)"/\1/' | tr -d '"' | xargs)"
if [ -z "${VERSION}" ]; then VERSION="0.0.0"; fi

APP_NAME="malScraper"
APP_DIR="${OUTPUT_DIR}/${APP_NAME}.app"
CONTENTS="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS}/MacOS"
RES_DIR="${CONTENTS}/Resources"

echo -e "${CYAN}========================================${RESET}"
echo -e "${CYAN}  Bundling ${APP_NAME}.app v${VERSION}${RESET}"
echo -e "${CYAN}========================================${RESET}"
echo "  Source binary : ${BINARY}"
echo "  Output bundle : ${APP_DIR}"
echo

rm -rf "${APP_DIR}"
mkdir -p "${MACOS_DIR}" "${RES_DIR}"

# 1. Copy the binary in as `malscraper-bin`, the launcher shim as `malScraper`.
#    Distinct names avoid collisions on macOS's case-insensitive filesystem.
cp "${BINARY}" "${MACOS_DIR}/malscraper-bin"
chmod +x "${MACOS_DIR}/malscraper-bin"

cp "${SCRIPT_DIR}/launcher.sh" "${MACOS_DIR}/${APP_NAME}"
chmod +x "${MACOS_DIR}/${APP_NAME}"

# 2. Render Info.plist with the current version.
sed "s/__VERSION__/${VERSION}/g" \
    "${SCRIPT_DIR}/Info.plist.template" > "${CONTENTS}/Info.plist"

# 3. Build AppIcon.icns from the available source icon.
ICON_SRC=""
if [ -f "${RUST_DIR}/assets/icon.png" ]; then
    ICON_SRC="${RUST_DIR}/assets/icon.png"
elif [ -f "${RUST_DIR}/assets/icon.ico" ]; then
    ICON_SRC="${RUST_DIR}/assets/icon.ico"
fi

if [ -n "${ICON_SRC}" ]; then
    echo -e "${CYAN}Generating AppIcon.icns from ${ICON_SRC}...${RESET}"
    TMP_ICONSET="$(mktemp -d)/AppIcon.iconset"
    mkdir -p "${TMP_ICONSET}"

    # Normalize source to PNG first.
    SRC_PNG="${TMP_ICONSET}/_source.png"
    sips -s format png "${ICON_SRC}" --out "${SRC_PNG}" >/dev/null

    for spec in \
        "16:icon_16x16.png" \
        "32:icon_16x16@2x.png" \
        "32:icon_32x32.png" \
        "64:icon_32x32@2x.png" \
        "128:icon_128x128.png" \
        "256:icon_128x128@2x.png" \
        "256:icon_256x256.png" \
        "512:icon_256x256@2x.png" \
        "512:icon_512x512.png" \
        "1024:icon_512x512@2x.png"
    do
        size="${spec%%:*}"
        name="${spec##*:}"
        sips -z "${size}" "${size}" "${SRC_PNG}" --out "${TMP_ICONSET}/${name}" >/dev/null
    done
    rm "${SRC_PNG}"

    iconutil -c icns "${TMP_ICONSET}" -o "${RES_DIR}/AppIcon.icns"
    rm -rf "$(dirname "${TMP_ICONSET}")"
    echo -e "${GREEN}  Icon written: ${RES_DIR}/AppIcon.icns${RESET}"
else
    echo -e "${YELLOW}No icon source found at assets/icon.png or assets/icon.ico — bundle will use the generic Mac icon.${RESET}"
    # Drop the CFBundleIconFile key so Finder falls back to the generic icon.
    /usr/libexec/PlistBuddy -c "Delete :CFBundleIconFile" "${CONTENTS}/Info.plist" 2>/dev/null || true
fi

# 4. Ad-hoc codesign so Gatekeeper / Apple Silicon will let it run.
echo -e "${CYAN}Ad-hoc codesigning bundle...${RESET}"
codesign --force --deep --sign - "${APP_DIR}"
codesign --verify --deep --strict "${APP_DIR}" && \
    echo -e "${GREEN}  Codesign OK${RESET}"

# 5. Report.
ARCH_INFO="$(file "${MACOS_DIR}/malscraper-bin" | sed 's/.*: //')"
SIZE="$(du -sh "${APP_DIR}" | awk '{print $1}')"

echo
echo -e "${GREEN}========================================${RESET}"
echo -e "${GREEN}  Bundle built successfully!${RESET}"
echo -e "${GREEN}========================================${RESET}"
echo -e "${YELLOW}Bundle :${RESET} ${APP_DIR}"
echo -e "${YELLOW}Arch   :${RESET} ${ARCH_INFO}"
echo -e "${YELLOW}Size   :${RESET} ${SIZE}"
echo
echo "Try it:"
echo "  open \"${APP_DIR}\""
echo
echo "Install:"
echo "  cp -R \"${APP_DIR}\" /Applications/"
