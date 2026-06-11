#!/usr/bin/env bash
# Builds the server binary and copies it where Tauri expects sidecars:
# desktop/src-tauri/binaries/app-starter-<target-triple>
# On Windows, append .exe to both source and destination.
set -euo pipefail
cd "$(dirname "$0")/.."

MODE=release
FLAGS=(--release)
if [[ "${1:-}" == "--debug" ]]; then
  MODE=debug
  FLAGS=()
fi

SKIP_FRONTEND_BUILD=1 cargo build "${FLAGS[@]}" --bin app-starter
TRIPLE=$(rustc -vV | sed -n 's/^host: //p')
DEST=desktop/src-tauri/binaries
mkdir -p "$DEST"
cp "target/$MODE/app-starter" "$DEST/app-starter-$TRIPLE"
echo "sidecar ready: $DEST/app-starter-$TRIPLE"
