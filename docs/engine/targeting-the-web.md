# Targeting the Web

> This documentation is for engine developers, not game builders!

Note for windows users:
- Always use the same OS with emsdk (WSL or Window, but not both)
- Make sure the python in your path is not the one from mingw, it messes with the activation scripts.
- You can check the environment works by running `where.exe python`, `where.exe node` and `where.exe emcc`

On non-windows platform, mise takes care of the emscripten installation for you!

```bash
# Needed to setup emsdk
git clone https://github.com/emscripten-core/emsdk
cd emsdk
git checkout 404dc1ec13f64fce1af1eaf5c007e18212f63527 # optional
cd ..
emsdk/emsdk install 4.0.13
# You only need to do this once emsdk is installed.
emsdk/emsdk activate 4.0.13
# Check the installation / activation
emcc -v
# emcc (Emscripten gcc/clang-like replacement + linker emulating GNU ld) 4.0.13 (2659582941bef14008476903f48941909db1b196)
# Copyright (C) 2025 the Emscripten authors (see AUTHORS.txt)
# This is free and open source software under the MIT license.
# There is NO warranty; not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.

cargo build -p runtime --target=wasm32-unknown-emscripten
```

## Graphics on the web

The runtime draws with wgpu on every platform. On the web, wgpu uses its OpenGL ES backend, which Emscripten translates
to WebGL 2 (`-sUSE_WEBGL2=1 -sFULL_ES3=1` in `runtime/build.rs`). No patch of wgpu is needed, but 2 things are specific
to this target:

- wgpu is built for EGL 1.5 and Emscripten only provides EGL 1.4. The 2 missing functions are never called on the web,
  so `runtime/src/graphics/emscripten_egl.rs` defines them to make the link succeed.
- `-sEXPORTED_FUNCTIONS` needs to stay in `runtime/build.rs`: without an explicit list, every function that
  `wasm-bindgen` generates in the dependencies of wgpu is exported, and the link fails on `__wbindgen_placeholder__`.

wgpu cannot use WebGPU with Emscripten yet (see https://github.com/gfx-rs/wgpu/pull/10515, which needs
Emscripten 6.0.10 or newer).

## Quickly testing the web version (export + web runtime)

```bash
mise test-export path/to/game.vecta path/to/output web [is_obfuscated] [--debug-web] [--serve]

# Example:
mise test-export gallery/game.vecta personal/tmp-export web --debug-web --serve
```

This exports with the latest GitHub release (cached in `target/latest-release`), where the editor and the CLI are replaced by debug builds of the current code.
`--debug-web` and `--debug-native` also replace the web runtime or the runtime of the current platform by a debug build, and `--serve` serves the export on http://localhost:8000.

## Using threads in the web build

The `env-for-web-thread-build.ps1` script is interesting to read if you want to enable `-pthread` on the web.

While `-pthread` works, we don't use threading for now and the build is experimental.

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
