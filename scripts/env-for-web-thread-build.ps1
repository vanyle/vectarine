#!/usr/bin/env pwsh

# Note: This script is only needed to build the engine with threading enabled on the web.
# You should never need this, threading is highly experimental.
# For threading to work, you need to set the rust toolchain to 'nightly'.

# If you are not using threading, these environment variables should be left empty!

$env:RUSTFLAGS = "-C target-feature=+atomics,+bulk-memory"
$env:EMCC_CFLAGS = "-O3 -pthread"
$env:CFLAGS = "-O3 -pthread"

. $PSScriptRoot/../emsdk/emsdk_env.ps1

echo "Environment is ready."
echo "You can know use threading in the web build!"
echo "Don't forget to set the rust toolchain to 'nightly' with 'rustup default nightly'"
echo "Run the build with: cargo build -p runtime --target=wasm32-unknown-emscripten -Z build-std=std,panic_abort"
echo "You can also use 'uv run release-engine.py'"
