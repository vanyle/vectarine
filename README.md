# 🍊 Vectarine Evolved

## Requirements for making games

- A text editor like Visual Studio Code
- (Optional) Installing a lua extension for your editor

## Getting started making games

See [the manual](./docs/user_manual.md) for how to make games with vectarine
Below are information on how to improve the engine.

## Requirements for working on the engine

- A working `uv` installation (uv is a python manager).
You can [install `uv`](https://docs.astral.sh/uv/getting-started/installation/) with `curl -LsSf https://astral.sh/uv/install.sh | sh`


- A working `rust` (and cargo!) installation
You can install `rust` with [`rustup`](https://www.rust-lang.org/tools/install)

See [Targeting the web](./docs/targeting-the-web.md) for more details on how to install emscripten and setup the web build.

## Getting started on the engine

- Start the game: `cargo run -p runtime`
- Build the game for the web: `emsdk/emsdk_env.ps1 && cargo build -p runtime --target wasm32-unknown-emscripten && uv run serve.py`
- Start the editor: `cargo run -p editor`
- Start the editor (hot recompile): `bacon editor`

## Structure

### Runtime

The main package (with code inside `runtime`) is an hybrid lib/bin package that can compile to the web without all the editor features like debugging, etc...
It provides the run time code for stuff like asset loading with multiple implementations per target.

### Editor

The `editor` is a bin package that builds the `vecta` executable.

`vecta` executable is a cross between a CLI and a GUI tool. Running it alone starts the editor where
you can select and run projects or see the documentation.
Running it with options allows you to create new project from templates, to open a specific project
or to build one.

The `editor` package is a wrapper around the `runtime` package that adds editor features like
debugging, hot reloading, etc...

## Usage

Run the game: `cargo run -p runtime`
Build the game for the web: `cargo build -p runtime --target wasm32-unknown-unknown`
Run the editor: `cargo run -p editor`

## Create an engine release

To make the project cross-platform, we use python for all build scripts.

To make a release build, run `uv run ./scripts/release-engine.py`.
A distributable zip file will be created at the root.
