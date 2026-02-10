#!/usr/bin/env bash
# Launch popTB.exe in Wine with cnc-ddraw
set -euo pipefail
export WINEDEBUG=-all

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
GAME_DIR="$PROJECT_DIR/data/original_game"

cd "$GAME_DIR"
wine popTB.exe "$@"
