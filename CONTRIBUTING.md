# Contributing

When adding features, keep in mind [the design principles of Vectarine!](./docs/engine/engine-design-principles.md)

Documentation about parts of the engine are inside the `docs/engine` folder.

## Preparing your environment

All setup commands in this CONTRIBUTING need to be run in a bash/zsh shell if you are on Unix and on a Powershell shell if you are on Windows.
Commands are prefixed with `both>`, `wind>`, `unix>`, depending on the target where they need to be run, `unix` meaning Linux or Mac.

You'll need to get started:

- A working `git` installation

- A working `mise` installation (mise is a package manager). See how to [install `mise`](https://mise.jdx.dev/getting-started.html).

Mise will install all the dependencies for you, including Rust, Python, Emscripten, and uv.

To make commands shorter, you need to [activate mise in your shell](https://mise.jdx.dev/getting-started.html#activate-mise)

> ### Activating mise on Windows
>
> First you have to allow `Invoke-Expression` execution in Powershell.
>
> ```bash
> wind> Set-ExecutionPolicy Unrestricted -Scope CurrentUser -Force
> ```
>
> Then open your Powershell profile file with your favorite text editor:
>
> ```bash
> wind> code $profile
> ```
>
> Then add the following line to your Powershell profile:
>
> ```bash
> (mise activate pwsh) | Out-String | Invoke-Expression
> ```
>
> Save and restart your Powershell terminal.

```bash
# Clone this repository
git clone ...
cd vectarine-enhanced
# Install all dependencies
mise install
# Add the emscripten target to the rust compiler
rustup target add wasm32-unknown-emscripten
```

> ℹ️ Getting familiar with mise
>
> You can list available mise tasks with `mise tasks`.
> You can run a task with `mise run <task-name>`.
> Mise is configured through the `mise.toml` file at the root of the repository.

## Getting started on the engine

Once you have everything installed, we need to setup SDL2.
SDL2 is the library we use for windowing, input, and OpenGL context creation.
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
both> cd path/to/the/location/of/vectarine/runtime
both> cargo install cargo-vcpkg
both> cd runtime
both> cargo vcpkg build
```

You are now ready to build and run the engine and the runtime!

> ℹ️ Troubleshooting
>
> On Windows, if after running `cargo run -p runtime` or `mise run run-editor` or any build command and you see a message like:
>
> `package sdl2 is not installed for vcpkg triplet x64-windows-static-md`
>
> This means `cargo vcpkg build` did not install the required packages.
> Assuming vcpkg is in your PATH, you can manually install them with:
>
> ```bash
> vcpkg install sdl2 sdl2-image sdl2-ttf sdl2-mixer sdl2-gfx --triplet x64-windows-static-md
> ```

## Common commands

Start the runtime: `cargo run -p runtime`

Start the editor: `cargo run -p editor`

Start the editor (with hot recompile): `bacon editor`

Build the game for the web:

```bash
# On Mac/Linux/WSL, mise can install emscripten for you. On Windows, you clone the repository yourself
wind> git clone https://github.com/emscripten-core/emsdk
wind> ./emsdk/emsdk install 4.0.13
wind> ./emsdk/emsdk activate 4.0.13
wind> ./emsdk/emsdk_env.ps1 # once you have run activate once, you can use this as a shorthand
both> cargo build -p runtime --target wasm32-unknown-emscripten
both> uv run ./script/serve.py # Start this in another terminal.
# Open http://localhost:8000 in your browser
```

When trying out web builds, you can also use the test runner by going to `http://localhost:8000/test-runner.html`.
The test-runner allows you to switch between debug and release builds.

Moreover, when debugging web specific crashes, you can consider installing the [C/C++ DevTools Support Chrome Extension](https://chromewebstore.google.com/detail/cc++-devtools-support-dwa/pdcpmagijalfljmkmjngeonclgbbannb)
More information about [debugging wasm code is available here](https://developer.chrome.com/docs/devtools/wasm/)

## Structure

### Runtime

The main package (with code inside `runtime`) is a hybrid lib/bin package that can compile to the web without all the editor features like debugging.
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

To make a release build, run `mise run release`.
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
cross build -p editor --target x86_64-unknown-linux-musl --release
```

**Cross compilation matrix**:

Can you build for platform _target_ when using _host_?

| Target \ Host | Windows | Linux | MacOS |
| ------------- | ------- | ----- | ----- |
| Windows       | ✅      | ⚠️    | ⚠️    |
| Linux         | ✅      | ✅    | ✅    |
| MacOS         | ❌      | ❌    | ✅    |
| Web           | ✅      | ✅    | ✅    |

> ⚠️ You'll need to use [cross toolchains](https://github.com/cross-rs/cross-toolchains) to perform the build. This is not tested and might not work.
> ❌ This is not legally possible due to licensing on Apple's SDK (but technically possible with cross).
