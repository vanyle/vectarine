<p align="center">
    <img src="./assets/textures/logo.png" alt="Vectarine logo" width="200" align="center"/>
</p>

<h1 align="center"> 🍊 Vectarine Evolved</h1>

*Vectarine is a game engine with a focus on ultra fast prototyping, ease of use and a great developer experience.*

## Goals by importance

- ⏰️ Developer time is valuable
  - 🛠️ Lua scripting: Instant reload
  - 🎨 Assets built into the engine for fast testing
  - 🖼️ Gallery of example: start with working templates
  - 🐛 Powerful debugging tools: waste less time on bugs
- 👾 Don't limit creativity
  - 🏭 Access to low-level primitives
  - 📐 3d & 2d support
- 🌍 Reach a wide audience
  - 🚀 Performance: Render millions of entities at 60 fps
  - 🌐 Supports the Web, Windows, Linux, MacOS
  - 📦 Distribute your game by sharing only 2 files with a small size footprint.

## Getting started making games

You'll need:

- A text editor like [Visual Studio Code](https://code.visualstudio.com/)
- (Optional) Installing a lua extension for your editor

See [the manual](./docs/user_manual.md) for how to make games with vectarine

Below are information on how to work and improve the engine.

## Requirements for working on the engine

- A working `git` installation

- A working `uv` installation (uv is a python manager). See how to [install `uv`](https://docs.astral.sh/uv/getting-started/installation/).

- A working `rust` (and cargo!) installation
You can install `rust` with [`rustup`](https://www.rust-lang.org/tools/install)

- (Optional but needed for web builds) A working [`emscripten`](https://emscripten.org/docs/getting_started/downloads.html) installation

You can install `emscripten` using:

```bash
# Run this inside the vectarine folder
git clone https://github.com/emscripten-core/emsdk.git
# Depending on your OS, run one of the following commands:
# On Windows (PowerShell):
.\emsdk\emsdk.ps1 install 4.0.13
# On Linux or MacOS (bash):
./emsdk/emsdk install 4.0.13
```

See [Targeting the web](./docs/targeting-the-web.md) for more details on how to install emscripten and setup the web build.

## Getting started on the engine

Once you have everything installed, we need to setup SDL2.
SDL2 is the library we use for windowing, input and OpenGL context creation.
You can use the [following reference](https://github.com/Rust-SDL2/rust-sdl2) for more details.
On Linux, you need to install the following package:

```bash
# Replace apt with your package manager of choice
sudo apt-get install libsdl2-dev libsdl2-image-dev libsdl2-ttf-dev libsdl2-mixer-dev libsdl2-gfx-dev

# You might also need this, but this is unlikely as these can already be installed by other programs
sudo apt-get install libmp3lame-dev
```

On MacOS and Windows, SDL2 is installed using vcpkg. First, install [vcpkg](https://github.com/microsoft/vcpkg):

On MacOS, you can install vcpkg using:

```bash
brew install vcpkg
cargo install cargo-vcpkg
cargo vcpkg build
```

On Window, you need to do it manually:

```bash
cd /path/to/where/you/want/it-installed
git clone https://github.com/microsoft/vcpkg.git
./bootstrap-vcpkg.bat # Build it
# Add vcpkg to your PATH
# On Windows, you can use the "Edit environment variables" utils to add a folder to your PATH
# Finally, you can setup the integration between vcpkg and cargo
cargo install cargo-vcpkg
cargo vcpkg build
```

You are now ready to build and run the engine and the runtime!

## Common commands

Start the runtime: `cargo run -p runtime`

Start the editor: `cargo run -p editor`

Start the editor (with hot recompile): `bacon editor`

Build the game for the web
```bash
emsdk/emsdk_env.ps1
cargo build -p runtime --target wasm32-unknown-emscripten
uv run serve.py # Start this in another terminal.
# Open http://localhost:8000 in your browser
```

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

## Create an engine release

To make the project cross-platform, we use python for all build scripts.

To make a release build, run `uv run ./scripts/release-engine.py`.
A distributable zip file will be created at the root.
