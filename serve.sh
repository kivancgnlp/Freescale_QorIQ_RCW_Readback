#!/usr/bin/env bash
# Serve the RCW Reader app locally.
# Builds a local staging copy in _deploy/ then serves it.
# Usage: ./serve.sh [port]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$SCRIPT_DIR/Rust project"
DEPLOY_DIR="$SCRIPT_DIR/_deploy"
PORT="${1:-8765}"

echo "==> Building WASM..."
cd "$RUST_DIR"
wasm-pack build --target web

echo "==> Staging files..."
rm -rf "$DEPLOY_DIR"
mkdir -p "$DEPLOY_DIR/pkg"

cp "$RUST_DIR/web/index.html" "$DEPLOY_DIR/"
cp "$RUST_DIR/web/style.css"  "$DEPLOY_DIR/"
sed 's|from "\.\./pkg/|from "./pkg/|g' "$RUST_DIR/web/index.js" > "$DEPLOY_DIR/index.js"
cp "$RUST_DIR/pkg/rcw_reader.js"      "$DEPLOY_DIR/pkg/"
cp "$RUST_DIR/pkg/rcw_reader_bg.wasm" "$DEPLOY_DIR/pkg/"

echo ""
echo "Serving at http://localhost:$PORT/"
echo "Press Ctrl+C to stop."
echo ""

cd "$DEPLOY_DIR"

if command -v python3 &>/dev/null; then
  python3 -m http.server "$PORT"
elif command -v python &>/dev/null; then
  python -m SimpleHTTPServer "$PORT"
elif command -v npx &>/dev/null; then
  npx serve -l "$PORT" .
else
  echo "No suitable server found. Install Python 3 or Node.js."
  exit 1
fi
