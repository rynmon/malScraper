#!/bin/bash
# Launcher shim for malScraper.app
#
# A double-clicked .app on macOS has no controlling terminal, so this shim
# opens a fresh Terminal window and execs the real malscraper binary inside it.
# When you launch malScraper from `open` or Finder/Launchpad/Spotlight, you
# get a real terminal session; when you run the binary directly from a
# terminal you'd already-running shell, just call `Contents/MacOS/malscraper`.

set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
BIN="${DIR}/malscraper-bin"

if [ ! -x "${BIN}" ]; then
    /usr/bin/osascript -e "display alert \"malScraper\" message \"Bundled binary not found at:\n${BIN}\" as critical"
    exit 1
fi

# Escape the path for embedding in an AppleScript string literal.
ESCAPED_BIN=$(printf '%s' "${BIN}" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g')

/usr/bin/osascript <<EOF
tell application "Terminal"
    activate
    do script "clear; exec \"${ESCAPED_BIN}\""
end tell
EOF
