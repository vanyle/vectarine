# Targeting the Web

> This documentation is for engine developers, not game builders!
> 
> If you are a game builder, use `vecta web` to start a webserver and see how your game looks in a browser.

Note for windows users:
- Always use the same OS with emsdk (WSL or Window, but not both)
- Make sure the python in your path is not the one from mingw, it messes with the activation scripts.
- You can check the environment works by running `where.exe python`, `where.exe node` and `where.exe emcc`

```bash
git clone https://github.com/emscripten-core/emsdk
cd emsdk
git checkout 404dc1ec13f64fce1af1eaf5c007e18212f63527
cd ..
emsdk/emsdk install 4.0.13
emsdk/emsdk activate 4.0.13
# Check the installation / activation
emcc -v
# emcc (Emscripten gcc/clang-like replacement + linker emulating GNU ld) 4.0.13 (2659582941bef14008476903f48941909db1b196)
# Copyright (C) 2025 the Emscripten authors (see AUTHORS.txt)
# This is free and open source software under the MIT license.
# There is NO warranty; not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.

cargo build -p runtime --target=wasm32-unknown-emscripten
```

## Using threads in the web build

The `env-for-web-thread-build.ps1` script is interesting to read if you want to enable `-pthread` on the web.

The idea is that we need to build with `-pthread` for all C files and `+atomics,+bulk-memory,+mutable-globals` for all rust files, including the standard library.
We thus need to set the rust version to nightly to enable the `-build-std=std,panic_abort` flag.

Setting environment variable is the only way to pass flags to emcc and cargo. Using `toml` files or `build.rs` **will NOT work**.

## Miscellaneous


This link contains rustflags that we might want: https://github.com/emscripten-core/emscripten/discussions/18156

```bash
EMCC_CFLAGS="-g -s ERROR_ON_UNDEFINED_SYMBOLS=0 --no-entry -s FULL_ES3=1"

# You need to set these manually or using a script, I cannot get cargo to set them.
# $env:RUSTFLAGS='-C target-feature=+atomics,+bulk-memory'
# $env:EMCC_CFLAGS = "-O3 -pthread"
# $env:CFLAGS = "-O3 -pthread"
```
