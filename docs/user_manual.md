# 🍊 Vectarine User Manual

Vectarine is a game engine to make games super quickly with the best possible experience for game makers.

Vectarine uses the [Lua](https://www.lua.org/manual/5.4/manual.html) programing language.
To be more precise, it uses [Luau](https://luau.org/), a variant of Lua with better performance and autocompletion than regular Lua but the same syntax.

This manual is an unopiniated guide to making games using Vectarine. If you already have a bit of game making experience and want to integrate Vectarine into your workflow,
this guide is for you. If you are new to making games, you can still read this guide, but an opiniated guide is in the works for you!

## Getting started

I recommend using [Visual Studio Code](https://code.visualstudio.com/) as a text editor with the [Luau extension](https://marketplace.visualstudio.com/items?itemName=JohnnyMorganz.luau-lsp) but
you are free to use any text editor you want, for example [Zed](https://zed.dev/) or [Neovim](https://neovim.io/).

Start the engine by running the `vecta` executable. A window should open.

> ⚠️ On MacOS, executables from the internet are quarantined by default.
> You might see this message when attempting to run `vecta-macos`: "This app is damaged"
> You need to run this command to allow the execution:
> `xattr -d com.apple.quarantine vecta-macos`

Write your game inside `assets/scripts/game.luau`. As you save, the window updates. See the `luau-api` folder for a list of available functions.
Your text editor should autocomplete from them.

## Release and distribute your game

Files ending with `.exe` are windows executables. Files with the same name, but without the `.exe` extension are for Linux.

**Desktop release**

To distribute your game, put the `runtime` executable and the `assets` folder in a zip. You can share the zip.

**Web release**

To distribute your game for the web, put together `index.html`, `runtime.js`, `runtime.wasm` and the `assets` folder.
You can serve these files with any static file server, for example by doing `python -m http.server` and going to [http://localhost:8000](http://localhost:8000).

If you see: "Error loading runtime", it means that you opened `index.html` directly from the file system instead of starting a server.
It can also mean that you forgot to put `runtime.js` and `runtime.wasm` in the same folder as `index.html`.

## Using Vectarine and Luau

When your application is first loaded, the function `Load` is called.
Then, every frame, the function `Update` is called with `time_delta` which is the duration since the start of the last frame in seconds.
Vectarine tries to run at 60 fps, so `time_delta` is at least `0.0166667` seconds and increases as your rendering gets more complicated.

A minimal example:

```lua
local Io = require('@vectarine/io')

function Load()
    Io.print("Game loaded")
end

function Update(time_delta)
    Io.print("Frame update, time since last frame: ", time_delta, " seconds")
end
```

## Drawing things on the screen

Drawing functions are inside the `graphics` module. Most function take a `V2` from the `vec` module to define positions and sizes.
The first argument to `V2` (called x) is the horizontal position, the second argument (called y) is the vertical position.

- `(0,0)` is the center of the screen.
- `(-1,1)` is the top left of the screen.
- `(1,-1)` is the bottom right of the screen.

The screen is always 2 units wide and 2 units tall, regardless of the window size or aspect ratio.

Example:

```lua
local Graphics = require('@vectarine/graphics')

function Update(time_delta: number)
    -- Draw a white background.
    local bg_color: Graphics.Color = { r = 1, g = 1, b = 1, a = 1 }
    Graphics.clear(bg_color)

    -- Draw a red circle at the center of the screen with radius 0.1 (2 is the width of the screen)
    local circle_color = {r = 1, g = 0, b = 0, a = 1}
    Graphics.drawCircle(V2(0.0, 0.0), 0.1, circle_color)

    -- Draw a blue rectangle at the bottom right of the screen
    local rect_color = {r = 0, g = 0, b = 1, a = 1}
    Graphics.drawRect(V2(0.7, -1), V2(0.3, 0.3), rect_color)
end
```

`Graphics` contains a lot of other functions to draw images, text, arrows or polygons. See [luau-api/graphics.luau](./luau-api/graphics.luau) for the full list.

## Interacting with the user

## Using sprites

## Making levels

## Spliting code into multiple files

You can write your game inside one giant `main.luau` file, but after a few hundred lines, scrolling takes time and
you spend more and more time searching for relevant lines. That is a sign that you should split your code into
multiple files.

To run another `luau` file, it needs to be loaded as a resource. You can load it using the `loadScript`.
Loading resources is not instant. The system needs to wait the resources to become ready. Meanwhile, you can
show a loading screen or something else.

Example:

```lua
local Resources = require('@vectarine/resources')
local Event = require('@vectarine/event')

local other_script_resource = Resources.loadScript("scripts/other_script.luau")

local resourceReadyEvent = Event.getResourceReadyEvent()
resourceReadyEvent:on(function(resource_handle)
    if other_script_resource == resource_handle then
        -- The resource is ready, you can access global variables and functions defined inside other_script
    end
end)

-- You can also check at any point if a resource is ready or not:
if Resources.isResourceReady(other_script_resource) then
    -- OK
end
```

By default all non-local variables and functions are shared between files.
This has pros and cons:

- If you have one big file, you can just copy and paste chunks into other files and everything will still work
- If two global functions have the same name, they will override each-other and the last one defined will win.
- You don't get typing across file boundaries, so you will get 'unknown global' errors from Luau despite everything working fine.
- Global variables have any types, so it you change them somewhere, you won't get errors about incorrect use elsewhere.

Because of that, we recommend doing the following (this is just a recommendation, you do you!):

- Keep functions local whenever possible using the `local function(...) function_content() end` syntax.
- Use the `Global.aa = bb` syntax to be explicit when defining globals.
- Use `require` to import types between modules
- When calling `loadScript`, pass a table as the second argument to gather the exports of the script.

There is a simple example with 2 files: `helper.luau` and `main.luau`.

```luau
-- helper.luau
local module = {} -- This is where all our exports will go
local my_value = 3

-- add_things is inside module, it gets exported
function module.add_things(a: number, b: number): number
    -- Notice that despite my_value not being exported, it can be used inside exported functions!
    return a + b + my_value
end

export type myType = "abc" | "def"

-- Global.helper = module -- optional: export the module table to Global
return module -- return for the module for typing
```

```lua
-- main.luau
local Resources = require('@vectarine/resources')
local Io = require('@vectarine/io')

-- import the types defined inside helper, this has no runtime effect
-- helper only contains an empty table, but has type information.
local helper = require("helper.luau")

-- We can still use the types defined inside helper!
local s: helper.myType = "abc"

--- loadScript is what actually executes `helper.luau`.
--- Note that `helper.luau` is only executed once. If you rerun loadScript, you'll get a handle to the same resource.
--- loadScript has as an optional argument a table where the values returned by the script will be stored.
--- This only works if the script returns a table, otherwise, this is ignored.
local helperResource = Resources.loadScript("scripts/helper.luau", helper)

function Update()
    if Resources.isResourceReady(helperResource) then
        --- You can retrieve functions from Global.helper with the proper type using this syntax:
        --- local add_things: typeof(helper.add_things) = Global.helper.add_things
        --- Alternatively, you can access `helper` directly because you put `helper` as an argument to `loadScript`.

        Io.fprint(helper.add_things.add_things(1, 2)) -- prints 1+2+3 = 6
    end
end
```

## Performance Tips

Call at `graphics.drawRect` at most 20 000 times per frame for 60 fps on all platforms.
I don't really know why you'd want to draw that many rectangles.

If you want to draw something on every pixel, use a Shader instead of called `graphics.drawRect` on a per-pixel basis!
