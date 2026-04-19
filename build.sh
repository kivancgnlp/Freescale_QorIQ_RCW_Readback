#!/usr/bin/env bash
# Build the RCW Reader WASM app.
# Output is placed in _deploy/ — copy its contents to your github.io repository.
# Usage: ./build.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$SCRIPT_DIR/Rust project"
DEPLOY_DIR="$SCRIPT_DIR/_deploy"

echo "==> Building WASM..."
cd "$RUST_DIR"
wasm-pack build --target web

echo "==> Preparing _deploy/ folder..."
rm -rf "$DEPLOY_DIR"
mkdir -p "$DEPLOY_DIR/pkg"

cp "$RUST_DIR/web/index.html" "$DEPLOY_DIR/"
cp "$RUST_DIR/web/style.css"  "$DEPLOY_DIR/"
sed 's|from "\.\./pkg/|from "./pkg/|g' "$RUST_DIR/web/index.js" > "$DEPLOY_DIR/index.js"
cp "$RUST_DIR/pkg/rcw_reader.js"      "$DEPLOY_DIR/pkg/"
cp "$RUST_DIR/pkg/rcw_reader_bg.wasm" "$DEPLOY_DIR/pkg/"

echo ""
echo "Done. Copy the contents of _deploy/ to your github.io repository:"
echo ""
echo "  _deploy/"
echo "  ├── index.html"
echo "  ├── index.js"
echo "  ├── style.css"
echo "  └── pkg/"
echo "      ├── rcw_reader.js"
echo "      └── rcw_reader_bg.wasm"
