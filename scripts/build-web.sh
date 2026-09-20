#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

bindgen="${WASM_BINDGEN:-}"
if [[ -z "$bindgen" ]]; then
    if [[ -x .tools/bin/wasm-bindgen ]]; then
        bindgen="$PWD/.tools/bin/wasm-bindgen"
    else
        bindgen=wasm-bindgen
    fi
fi
expected=$(awk '/^name = "wasm-bindgen"$/ { getline; gsub(/"/, "", $3); print $3; exit }' Cargo.lock)
if ! actual=$("$bindgen" --version 2>/dev/null) || [[ "$actual" != "wasm-bindgen $expected" ]]; then
    echo "Install the matching build tool: cargo install wasm-bindgen-cli --version $expected --locked" >&2
    exit 1
fi
if ! rustup target list --installed | grep -qx wasm32-unknown-unknown; then
    echo 'Install the Rust target: rustup target add wasm32-unknown-unknown' >&2
    exit 1
fi

# Panic locations survive stripping debug info. Remap local paths before compiling
# so publishing a local build cannot reveal the builder's username or checkout.
if [[ ! -v CARGO_ENCODED_RUSTFLAGS ]]; then
    read -r -a flags <<< "${RUSTFLAGS:-}"
    printf -v CARGO_ENCODED_RUSTFLAGS '%s\x1f' "${flags[@]}"
    CARGO_ENCODED_RUSTFLAGS=${CARGO_ENCODED_RUSTFLAGS%$'\x1f'}
fi
for mapping in "$HOME=/build-home" "${CARGO_HOME:-$HOME/.cargo}=/cargo" "$PWD=/source"; do
    CARGO_ENCODED_RUSTFLAGS+="${CARGO_ENCODED_RUSTFLAGS:+$'\x1f'}--remap-path-prefix=$mapping"
done
export CARGO_ENCODED_RUSTFLAGS
cargo build --locked --profile web --target wasm32-unknown-unknown --no-default-features --features web
mkdir -p dist
"$bindgen" --target web --remove-name-section --remove-producers-section --out-dir dist --out-name airy \
    target/wasm32-unknown-unknown/web/airys-experiment.wasm
# Keep the JS glue and Wasm on the same build when browsers cache local previews.
version=$(sha256sum dist/airy.js dist/airy_bg.wasm | sha256sum | cut -c1-16)
wasm_bytes=$(wc -c < dist/airy_bg.wasm)
sed -e "s/__BUILD_VERSION__/$version/g" -e "s/__WASM_BYTES__/$wasm_bytes/g" web/index.html > dist/index.html
echo 'Built dist/. Serve with: python3 -m http.server 8000 --bind 127.0.0.1 --directory dist'
