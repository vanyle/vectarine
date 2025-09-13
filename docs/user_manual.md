# Vectarine User Manual

*This documentation is for game makers, not for engine developers!*

**🍊 Welcome to the Vectarine! 🍊**

Vectarine is a game engine to make games super quickly with the best possible developer experience.

Vectarine uses [Lua](https://www.lua.org/manual/5.4/manual.html) as its scripting language.

## Getting started

I recommend using [Visual Studio Code](https://code.visualstudio.com/) as a text editor with the [Lua extension](https://marketplace.visualstudio.com/items?itemName=sumneko.lua) but
you are free to use any text editor you want.

Start the engine by running the `vecta` executable. A window should open.

Write your game inside `assets/scripts/game.lua`. As you save, the window updates. See the `lua-api` folder for a list of available functions. VSCode should autocomplete from them.

## Release and distribute your game

Files ending with `.exe` are windows executable. Files with the same name, but without the `.exe` extension are for Linux.

**Desktop release**

To distribute your game, put the `runtime` executable and the `assets` folder in a zip. You can share the zip.

**Web release**

To distribute your game for the web, put together `index.html`, `runtime.js`, `runtime.wasm` and the `assets` folder.
You can serve these files with any static file server, for example by doing `python -m http.server` and going to [http://localhost:8000](http://localhost:8000).

If you see: "Error loading runtime", it means that you opened `index.html` directly from the file system instead of starting a server.
It can also mean that you forgot to put `runtime.js` and `runtime.wasm` in the same folder as `index.html`.
