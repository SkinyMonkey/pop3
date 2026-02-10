#!/usr/bin/env bash
# Launch popTB.exe under Wine and inject a Frida script.
#
# Usage:
#   ./tmp/run-game-frida.sh <script.js>
#   ./tmp/run-game-frida.sh tmp/frida-pathfinding.js
#
# The script is injected via frida-inject.exe (Windows x86) running inside
# Wine — this is required on Apple Silicon where the native macOS Frida
# cannot inject into Rosetta 2 translated processes.
#
# Prerequisites:
#   - Wine installed (brew install --cask wine-stable)
#   - tmp/frida-inject.exe downloaded (see setup-wine.sh)
#   - cnc-ddraw installed in data/original_game/ (see setup-wine.sh)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
GAME_DIR="$PROJECT_DIR/data/original_game"
FRIDA_INJECT="$SCRIPT_DIR/frida-inject.exe"

if [ $# -lt 1 ]; then
    echo "Usage: $0 <frida-script.js>"
    exit 1
fi

FRIDA_SCRIPT="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"

if [ ! -f "$FRIDA_SCRIPT" ]; then
    echo "ERROR: Script not found: $FRIDA_SCRIPT"
    exit 1
fi

if [ ! -f "$FRIDA_INJECT" ]; then
    echo "ERROR: frida-inject.exe not found at $FRIDA_INJECT"
    echo "       Download it: curl -sL https://github.com/frida/frida/releases/download/17.4.4/frida-inject-17.4.4-windows-x86.exe.xz | xz -d > $FRIDA_INJECT"
    exit 1
fi

# Convert to Wine Z: drive path
WIN_SCRIPT="Z:${FRIDA_SCRIPT//\//\\}"

export WINEDEBUG=-all

is_game_running() {
    if command -v pgrep >/dev/null 2>&1; then
        pgrep -f "popTB\\.exe" >/dev/null 2>&1
    else
        # Avoid grep|pipefail false negatives when a downstream command exits early.
        ps auxx | awk '/[p]opTB\\.exe/ { found=1 } END { exit found ? 0 : 1 }'
    fi
}

# Kill any existing game
pkill -f popTB.exe 2>/dev/null && sleep 2 || true

# Launch the game
echo "[*] Launching popTB.exe..."
cd "$GAME_DIR"
wine popTB.exe &
WINE_PID=$!

# Wait for the game process to appear
echo "[*] Waiting for game to start..."
for attempt in $(seq 1 10); do
    sleep 1
    if is_game_running; then
        break
    fi
    if [ "$attempt" -eq 10 ]; then
        echo "ERROR: Game did not start within 10 seconds"
        exit 1
    fi
done
sleep 2

# Find the Wine internal PID for popTB.exe
echo "[*] Finding Wine PID..."
WPID_HEX=$(winedbg --command "info proc" 2>&1 | grep "popTB.exe" | awk '{print $1}')
if [ -z "$WPID_HEX" ]; then
    echo "ERROR: Could not find popTB.exe in Wine process list"
    exit 1
fi
WPID_DEC=$((16#${WPID_HEX}))
echo "[*] popTB.exe Wine PID: $WPID_HEX ($WPID_DEC)"

# Inject the Frida script
echo "[*] Injecting: $FRIDA_SCRIPT"
wine "$FRIDA_INJECT" -p "$WPID_DEC" -s "$WIN_SCRIPT" --eternalize 2>&1

echo "[*] Script injected and eternalized. Game is running with hooks active."
echo "[*] Press Ctrl+C to kill the game when done."

# Wait for the game to exit
wait $WINE_PID 2>/dev/null || true
echo "[*] Game exited."
