#!/bin/sh
# =============================================================================
#  Video Saver - macOS / Linux launcher.
#  Run it with:  ./start.sh
#
#  1. Checks that Python 3, yt-dlp, ffmpeg and deno are installed.
#  2. If something is missing, prints how to install it.
#  3. Starts the app and opens it in your browser.
# =============================================================================
cd "$(dirname "$0")" || exit 1

missing=""
command -v python3 >/dev/null 2>&1 || missing="$missing python"
for tool in yt-dlp ffmpeg deno; do
    command -v "$tool" >/dev/null 2>&1 || missing="$missing $tool"
done

if [ -n "$missing" ]; then
    echo "Video Saver needs:$missing"
    echo
    echo "  macOS (Homebrew):  brew install python yt-dlp ffmpeg deno"
    echo "  Linux:             install python3 and ffmpeg from your package manager,"
    echo "                     yt-dlp: https://github.com/yt-dlp/yt-dlp#installation"
    echo "                     deno:   https://docs.deno.com/runtime/getting_started/installation/"
    echo
    echo "Then run ./start.sh again."
    exit 1
fi

exec python3 app.py
