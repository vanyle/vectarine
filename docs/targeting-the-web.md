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
