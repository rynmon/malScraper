#!/usr/bin/env bash
# Build script for malScraper (macOS / Linux)
# Usage:
#   ./build.sh              # native build (auto-detects host architecture)
#   ./build.sh --arm        # macOS ARM (aarch64-apple-darwin)
#   ./build.sh --x86_64     # Intel macOS (x86_64-apple-darwin)
#   ./build.sh --universal  # macOS universal binary (ARM + x86_64 lipo'd)
#   ./build.sh --linux      # x86_64 Linux
#   ./build.sh --linux-arm  # aarch64 Linux
#
# macOS .app bundles (build + wrap into a double-clickable app):
#   ./build.sh --app            # native architecture, bundled as malScraper.app
#   ./build.sh --app-arm        # aarch64-apple-darwin, bundled
#   ./build.sh --app-x86_64     # x86_64-apple-darwin, bundled
#   ./build.sh --app-universal  # universal binary, bundled

set -euo pipefail

CYAN='\033[1;36m'
GREEN='\033[1;32m'
YELLOW='\033[1;33m'
RED='\033[1;31m'
RESET='\033[0m'

cd "$(dirname "$0")"

echo -e "${CYAN}========================================${RESET}"
echo -e "${CYAN}  malScraper Build Script${RESET}"
echo -e "${CYAN}========================================${RESET}"
echo

if ! command -v cargo >/dev/null 2>&1; then
    echo -e "${RED}ERROR: cargo not found in PATH${RESET}"
    echo
    echo -e "${YELLOW}Install Rust first:${RESET}"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "  source \"\$HOME/.cargo/env\""
    exit 1
fi
echo -e "${GREEN}Found:${RESET} $(cargo --version)"

HOST_OS="$(uname -s)"
HOST_ARCH="$(uname -m)"

MODE="${1:-native}"

resolve_native_target() {
    case "${HOST_OS}-${HOST_ARCH}" in
        Darwin-arm64)  echo "aarch64-apple-darwin" ;;
        Darwin-x86_64) echo "x86_64-apple-darwin" ;;
        Linux-x86_64)  echo "x86_64-unknown-linux-gnu" ;;
        Linux-aarch64) echo "aarch64-unknown-linux-gnu" ;;
        *) echo -e "${RED}Unsupported host: ${HOST_OS}-${HOST_ARCH}${RESET}" >&2; exit 1 ;;
    esac
}

ensure_target() {
    local target="$1"
    if ! rustup target list --installed 2>/dev/null | grep -q "^${target}\$"; then
        echo -e "${YELLOW}Installing rustup target: ${target}${RESET}"
        rustup target add "${target}"
    fi
}

build_target() {
    local target="$1"
    ensure_target "${target}"
    echo
    echo -e "${CYAN}Building target: ${target}${RESET}"
    cargo build --release --target "${target}"
}

report_binary() {
    local path="$1"
    if [ -f "${path}" ]; then
        local size
        size=$(du -h "${path}" | awk '{print $1}')
        echo
        echo -e "${YELLOW}Binary:${RESET} ${path}"
        echo -e "${YELLOW}Size:${RESET}   ${size}"
    fi
}

case "${MODE}" in
    native)
        TARGET="$(resolve_native_target)"
        build_target "${TARGET}"
        report_binary "target/${TARGET}/release/malscraper"
        ;;
    --arm|--aarch64|--apple-silicon)
        if [ "${HOST_OS}" != "Darwin" ]; then
            echo -e "${RED}--arm requires macOS host${RESET}" >&2; exit 1
        fi
        build_target "aarch64-apple-darwin"
        report_binary "target/aarch64-apple-darwin/release/malscraper"
        ;;
    --x86_64|--intel)
        if [ "${HOST_OS}" != "Darwin" ]; then
            echo -e "${RED}--x86_64 requires macOS host (use --linux for Linux x86_64)${RESET}" >&2; exit 1
        fi
        build_target "x86_64-apple-darwin"
        report_binary "target/x86_64-apple-darwin/release/malscraper"
        ;;
    --universal)
        if [ "${HOST_OS}" != "Darwin" ]; then
            echo -e "${RED}--universal requires macOS host${RESET}" >&2; exit 1
        fi
        build_target "aarch64-apple-darwin"
        build_target "x86_64-apple-darwin"
        mkdir -p target/universal-apple-darwin/release
        OUT="target/universal-apple-darwin/release/malscraper"
        echo
        echo -e "${CYAN}Creating universal binary via lipo...${RESET}"
        lipo -create \
            "target/aarch64-apple-darwin/release/malscraper" \
            "target/x86_64-apple-darwin/release/malscraper" \
            -output "${OUT}"
        echo -e "${GREEN}lipo info:${RESET} $(lipo -info "${OUT}")"
        report_binary "${OUT}"
        ;;
    --linux)
        build_target "x86_64-unknown-linux-gnu"
        report_binary "target/x86_64-unknown-linux-gnu/release/malscraper"
        ;;
    --linux-arm|--linux-aarch64)
        build_target "aarch64-unknown-linux-gnu"
        report_binary "target/aarch64-unknown-linux-gnu/release/malscraper"
        ;;
    --app)
        if [ "${HOST_OS}" != "Darwin" ]; then
            echo -e "${RED}--app requires macOS host${RESET}" >&2; exit 1
        fi
        TARGET="$(resolve_native_target)"
        build_target "${TARGET}"
        report_binary "target/${TARGET}/release/malscraper"
        ./scripts/bundle-macos.sh "target/${TARGET}/release/malscraper"
        ;;
    --app-arm|--app-aarch64|--app-apple-silicon)
        if [ "${HOST_OS}" != "Darwin" ]; then
            echo -e "${RED}--app-arm requires macOS host${RESET}" >&2; exit 1
        fi
        build_target "aarch64-apple-darwin"
        report_binary "target/aarch64-apple-darwin/release/malscraper"
        ./scripts/bundle-macos.sh "target/aarch64-apple-darwin/release/malscraper"
        ;;
    --app-x86_64|--app-intel)
        if [ "${HOST_OS}" != "Darwin" ]; then
            echo -e "${RED}--app-x86_64 requires macOS host${RESET}" >&2; exit 1
        fi
        build_target "x86_64-apple-darwin"
        report_binary "target/x86_64-apple-darwin/release/malscraper"
        ./scripts/bundle-macos.sh "target/x86_64-apple-darwin/release/malscraper"
        ;;
    --app-universal)
        if [ "${HOST_OS}" != "Darwin" ]; then
            echo -e "${RED}--app-universal requires macOS host${RESET}" >&2; exit 1
        fi
        build_target "aarch64-apple-darwin"
        build_target "x86_64-apple-darwin"
        mkdir -p target/universal-apple-darwin/release
        OUT="target/universal-apple-darwin/release/malscraper"
        echo
        echo -e "${CYAN}Creating universal binary via lipo...${RESET}"
        lipo -create \
            "target/aarch64-apple-darwin/release/malscraper" \
            "target/x86_64-apple-darwin/release/malscraper" \
            -output "${OUT}"
        echo -e "${GREEN}lipo info:${RESET} $(lipo -info "${OUT}")"
        report_binary "${OUT}"
        ./scripts/bundle-macos.sh "${OUT}"
        ;;
    -h|--help)
        sed -n '2,15p' "$0"
        exit 0
        ;;
    *)
        echo -e "${RED}Unknown option: ${MODE}${RESET}" >&2
        echo "Run './build.sh --help' for usage." >&2
        exit 1
        ;;
esac

echo
echo -e "${GREEN}========================================${RESET}"
echo -e "${GREEN}  Build Successful!${RESET}"
echo -e "${GREEN}========================================${RESET}"
