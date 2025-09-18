# 🍊 Vectarine User Manual

Vectarine is a game engine to make games super quickly with the best possible developer experience.

Vectarine uses [Lua](https://www.lua.org/manual/5.4/manual.html) as its scripting language.
To be more precise, it uses [Luau](https://luau.org/), a variant of Lua with optional static typing and better performance than regular Lua but the
same syntax.

## Getting started

I recommend using [Visual Studio Code](https://code.visualstudio.com/) as a text editor with the [Lua extension](https://marketplace.visualstudio.com/items?itemName=sumneko.lua) but
you are free to use any text editor you want.

Start the engine by running the `vecta` executable. A window should open.

> ⚠️ On MacOS, executables from the internet are quarantined by default.
> You might see this message when attempting to run `vecta-macos`: "This app is damaged"
> You need to run this command to allow the execution:
> `xattr -d com.apple.quarantine vecta-macos`

Write your game inside `assets/scripts/game.lua`. As you save, the window updates. See the `lua-api` folder for a list of available functions. VSCode should autocomplete from them.

## Release and distribute your game

Files ending with `.exe` are windows executables. Files with the same name, but without the `.exe` extension are for Linux.

**Desktop release**

To distribute your game, put the `runtime` executable and the `assets` folder in a zip. You can share the zip.

**Web release**

To distribute your game for the web, put together `index.html`, `runtime.js`, `runtime.wasm` and the `assets` folder.
You can serve these files with any static file server, for example by doing `python -m http.server` and going to [http://localhost:8000](http://localhost:8000).

If you see: "Error loading runtime", it means that you opened `index.html` directly from the file system instead of starting a server.
It can also mean that you forgot to put `runtime.js` and `runtime.wasm` in the same folder as `index.html`.

## Lua API

When your application is first loaded, the lua function `Load` is called.
Then, every frame, the function `Update` is called with `time_delta` which is the duration since the start of the last frame in seconds.
Vectarine tries to run at 60 fps, so `time_delta` is at least `0.0166667` seconds and increases as your rendering gets more complicated.

A minimal example:

```lua
function Load()
    dprint("Game loaded")
end

function Update(time_delta)
    fprint("Frame update, time since last frame: " .. time_delta .. " seconds")
end
```

## Performance

Call at drawRect at most 20 000 times per frame for 60 fps on all platforms.
I don't really know why you'd want to draw that many rectangles.
