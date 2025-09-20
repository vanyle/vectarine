<p align="center">
    <img src="./assets/textures/logo.png" alt="Vectarine logo" width="200" align="center"/>
</p>

<h1 align="center"> 🍊 Vectarine Evolved</h1>

_Vectarine is a game engine with a focus on ultra fast prototyping, ease of use and a great developer experience._

## Goals by importance

- ⏰️ Developer time is valuable
  - 🛠️ Luau scripting: Instant reload and strong typing
  - 🎨 Assets built into the engine for fast testing
  - 🖼️ Gallery of example: start with working templates
  - 🐛 Powerful debugging tools: waste less time on bugs
- 👾 Don't limit creativity
  - 🏭 Access to low-level primitives
  - 📐 3d & 2d support
  - 🚀 Performance: Render millions of entities at 60 fps
- 🌍 Reach a wide audience
  - 🌐 Supports the Web, Windows, Linux, MacOS
  - 📦 Distribute your game by sharing only 2 files with a small size footprint.
  - 📖 Free and open-source

## Getting started making games

You'll need:

- A text editor like [Visual Studio Code](https://code.visualstudio.com/)
- (Optional) Installing a Luau extension for your editor

See [the manual](./docs/user_manual.md) for how to make games with vectarine

Below are information on how to work and improve the engine.

## Requirements for working on the engine

All setup commands in this README need to be ran in a bash/zsh shell if you are on Unix and on a Powershell shell if you are on Windows.
Commands are prefixed with `both>`, `wind>`, `unix>`, depending on the target where they need to be ran, `unix` meaning Linux or Mac.

You'll need to get started:

- A working `git` installation

- A working `uv` installation (uv is a python manager). See how to [install `uv`](https://docs.astral.sh/uv/getting-started/installation/).

- A working `rust` (and cargo!) installation. You can install `rust` with [`rustup`](https://www.rust-lang.org/tools/install)

- (Optional but needed for web builds) A working [`emscripten`](https://emscripten.org/docs/getting_started/downloads.html) installation

You can install `emscripten` using:

```bash
# Run this inside the vectarine folder
both> git clone https://github.com/emscripten-core/emsdk.git
# Depending on your OS, run one of the following commands:
wind> .\emsdk\emsdk.ps1 install 4.0.13
unix> ./emsdk/emsdk install 4.0.13
# Activate it
wind> .\emsdk\emsdk.ps1 activate 4.0.13
unix> ./emsdk/emsdk activate 4.0.13
# Add the emscripten target to the rust compiler
both> rustup target add wasm32-unknown-emscripten
```

See [Targeting the web](./docs/targeting-the-web.md) for more details on how to install emscripten and setup the web build.

## Getting started on the engine

Once you have everything installed, we need to setup SDL2.
SDL2 is the library we use for windowing, input and OpenGL context creation.
You can use the [following reference](https://github.com/Rust-SDL2/rust-sdl2) for more details.

If you are on linux and want to only compile for linux or the web, you can simply install the following packages:

```bash
# Replace apt with your package manager of choice
sudo apt-get install libsdl2-dev libsdl2-image-dev libsdl2-ttf-dev libsdl2-mixer-dev libsdl2-gfx-dev

# You might also need this, but it can already be installed
sudo apt-get install libmp3lame-dev
```

Otherwise, you'll need to install SDL2 through vcpkg.
On MacOS and Windows, this is mandatory. First, install [vcpkg](https://github.com/microsoft/vcpkg):

```bash
wind> cd $home
unix> cd ~
# We assume you are install vcpkg in your home folder, but you can put it anywhere on your computer
both> git clone https://github.com/microsoft/vcpkg.git
both> cd vcpkg

wind> ./bootstrap-vcpkg.bat # Build it on windows
unix> ./bootstrap-vcpkg.sh # Build it on Linux/MacOS

# Add vcpkg to your PATH
# On Windows, you can use the "Edit environment variables" utils to add a folder to your PATH
unix> export PATH="$(pwd):$PATH"
# Add VCPKG_ROOT to your path too:
unix> export VCPKG_ROOT=$(pwd)
wind> $env:VCPKG=(Get-Item .).FullName

# Finally, you can setup the integration between vcpkg and cargo
both> cd path/to/the/location/of/vectarine
both> cargo install cargo-vcpkg
both> cd runtime
both> cargo vcpkg build
```

You are now ready to build and run the engine and the runtime!

## Common commands

Start the runtime: `cargo run -p runtime`

Start the editor: `cargo run -p editor`

Start the editor (with hot recompile): `bacon editor`

Build the game for the web

```bash
wind> ./emsdk/emsdk_env.ps1
unix> source "./emsdk/emsdk_env.sh"
both> cargo build -p runtime --target wasm32-unknown-emscripten
both> uv run serve.py # Start this in another terminal.
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

## Building for other platforms than yours

You will need a [working Docker installation](https://docs.docker.com/engine/install/)

If you are using Linux, you might want to create a release for Windows.
If you are on Mac, you'd like to target Linux.

To do so, first install `cross`:

```bash
cargo install cross --git https://github.com/cross-rs/cross
```

Then, build as usual while replacing `cargo` with `cross`:

```bash
# Making a Linux build on Windows/MacOS
cross build -p editor --target x86_64-unknown-linux-gnu --release
```
