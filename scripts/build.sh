#!/bin/sh
# Builds the whole static site into dist/:
#   dist/index.html        opens the Orbital dashboard
#   dist/commerce/         Commerce Center dashboard  + its sample reports
#   dist/orbital/          Orbital (DFR) dashboard    + its synthetic sample files
# Both Rust engines run in the browser as WebAssembly, so dist/ can be served by any static host.
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$HOME/.local/node/bin:$PATH"

echo "1/4  Rust engines -> WebAssembly"
cargo build -p recon_wasm --release --target wasm32-unknown-unknown --quiet
wasm-bindgen --target web --out-dir web/wasm-pkg target/wasm32-unknown-unknown/release/recon_wasm.wasm
for app in commerce orbital; do rm -rf "web/$app/src/engine/wasm"; cp -R web/wasm-pkg "web/$app/src/engine/wasm"; done

rm -rf dist && mkdir -p dist
for app in commerce orbital; do
  echo "2/4  $app dashboard"
  ( cd "web/$app" && { [ -d node_modules ] || npm ci --silent; } && npx rescript >/dev/null && npx vite build --logLevel warn --outDir "$ROOT/dist/$app" --emptyOutDir )
  rm -rf "dist/$app/samples" && cp -R "engines/$app/samples" "dist/$app/samples"
done

echo "3/4  start page"
cp web/index.html dist/index.html

echo "4/4  done: $(du -sh dist | cut -f1) in dist/"
