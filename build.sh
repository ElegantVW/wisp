#!/usr/bin/env bash
# build.sh — release wisp; install --house → faeOS engine path
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"
cargo build --release
BIN="$ROOT/target/release/wisp"
echo "built: $BIN"
ls -la "$BIN"

if [[ "${1:-}" != "install" ]]; then
  exit 0
fi
shift || true
HOUSE=0
if [[ "${1:-}" == "--house" ]]; then
  HOUSE=1
fi

LIB="$HOME/.local/lib/faeos"
WRAP_SRC="$ROOT/scripts/wisp"
mkdir -p "$LIB" "$HOME/bin"
cp -f "$BIN" "$LIB/wisp"
chmod +x "$LIB/wisp"

install_launcher() {
  local dest="$1"
  mkdir -p "$(dirname "$dest")"
  cp -f "$WRAP_SRC" "$dest"
  chmod +x "$dest"
}

install_launcher "$HOME/bin/wisp"
if [[ "$HOUSE" -eq 1 ]]; then
  for kit in "$HOME/faeOS/bin" "$HOME/faeos/bin"; do
    if [[ -d "$kit" ]]; then
      install_launcher "$kit/wisp"
    fi
  done
fi

echo "installed engine → $LIB/wisp"
echo "launcher        → $HOME/bin/wisp"
