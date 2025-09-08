# Vectarine Evolved

## Requirements for making games

- A text editor like Visual Studio Code
- (Optional) Installing a lua extension for your editor

## Getting start making games

See [Getting start](./docs/getting-started.md) for how to make games with vectarine
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

## Structure / Goals / Roadmap

### Editor

The `editor` is a bin package that builds the `vecta` executable.

`vecta` executable is a cross between a CLI and a GUI tool. Running it alone starts the editor where
you can select and run projects or see the documentation.
Running it with options allows you to create new project from templates, to open a specific project
or to build one.

### Runtime

The main package (with code inside `src`) is an hybrid lib/bin package that can compile to the web without all the editor features like debugging, etc...
It provides the run time code for stuff like asset loading with multiple implementations per target.

`runtime` needs to be able to load images and other resources through HTTP requests to avoid super long load times.
This means that asset fetching functions need to be async. We can provide some non-async resources like a font and some icons
for loading screens and the logo of the engine.

### Game

The `game` package is a small binary wrapper around the `runtime` to be able to run the game on desktops.

## Usage

Run the game: `cargo run -p game`
Build the game for the web: `cargo build -p runtime --target wasm32-unknown-unknown`
Run the editor: `cargo run -p editor`

## Build

For cross platform reason, all build script need to be python (if we do python code gen for rust?)

