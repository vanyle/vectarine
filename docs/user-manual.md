---
title: "Vectarine User Manual"
geometry: "left=1cm,right=1cm,top=1cm,bottom=1cm"
output: pdf_document
---

# 🍊 Vectarine User Manual

Vectarine is a game engine to make games super quickly with the best possible experience for game makers.

Vectarine uses the [Lua](https://www.lua.org/manual/5.4/manual.html) programing language.
To be more precise, it uses [Luau](https://luau.org/), a variant of Lua with better performance and autocompletion than regular Lua but the same syntax.

This manual is an unopiniated guide to making games using Vectarine. If you already have a bit of game making experience and want to integrate Vectarine into your workflow,
this guide is for you. If you are new to making games, you can still read this guide, but an opiniated guide is in the works for you!

> ℹ️ Parts annotated with 👷 are a work-in-progress and describe the goals of vectarine, not its current state.

## Getting started

I recommend using [Visual Studio Code](https://code.visualstudio.com/) as a text editor with the [Luau extension](https://marketplace.visualstudio.com/items?itemName=JohnnyMorganz.luau-lsp) but
you are free to use any text editor you want, for example [Zed](https://zed.dev/) or [Neovim](https://neovim.io/).

Start the engine by running the `vecta` executable. A window should open.

> ⚠️ On MacOS, executables from the internet are quarantined by default.
> You might see this message when attempting to run `vecta-macos`: "This app is damaged"
> You need to run this command to allow the execution:
> `xattr -d com.apple.quarantine vecta-macos`

Write your game inside `gamedata/scripts/game.luau`. As you save, the window updates. See the `luau-api` folder for a list of available functions.
Your text editor should autocomplete from them.

## Release and distribute your game

Files ending with `.exe` are windows executables. Files with the same name, but without the `.exe` extension are for Linux.

**Desktop release**

To distribute your game, put the `runtime` executable and the `gamedata` folder in a zip. You can share the zip.

**Web release**

To distribute your game for the web, put together `index.html`, `runtime.js`, `runtime.wasm` and the `gamedata` folder.
You can serve these files with any static file server, for example by doing `python -m http.server` and going to [http://localhost:8000](http://localhost:8000).

If you see: "Error loading runtime", it means that you opened `index.html` directly from the filesystem instead of starting a server.
It can also mean that you forgot to put `runtime.js` and `runtime.wasm` in the same folder as `index.html`.

You put these files in a zip and upload it to [itch.io](https://itch.io) if you want.

## Using Vectarine and Luau

When your application is first loaded, the function `Load` is called.
Then, every frame, the function `Update` is called with `time_delta` which is the duration since the start of the last frame in seconds.
Vectarine tries to run at 60 fps, so `time_delta` is at least `0.0166667` seconds and increases as your rendering gets more complicated.

A minimal example:

```lua
local Debug = require('@vectarine/debug')

function Load()
    Debug.print("Game loaded")
end

function Update(time_delta)
    Debug.print("Frame update, time since last frame: ", time_delta, " seconds")
end
```

## Drawing things on the screen

Drawing functions are inside the `graphics` module.

When drawing things on the screen, you need to tell Vectarine where to put them.
To do so, you have two options, you can either use **Vec** or **Coord**.

### Using Vec

Most functions can take a `V2` from the `vec` module to define positions and sizes.
The first argument to `V2` (called x) is the horizontal position, the second argument (called y) is the vertical position.

- `(0,0)` is the center of the screen.
- `(-1,-1)` is the bottom left of the screen.
- `(-1,1)` is the top left of the screen.
- `(1,-1)` is the bottom right of the screen.
- `(1,1)` is the top right of the screen.

The screen is always 2 units wide and 2 units tall, regardless of the window size or aspect ratio.

Example:

```lua
local Vec = require('@vectarine/vec')
local V2 = Vec.V2 -- alias the V2 function as it is used very often
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

### Using Coord

> ℹ️ TLDR; Coordinates are like vectors with a unit.

Drawing with `Vec` is convenient, however, often, you want to draw squares, or shapes where the width to height ratio needs to stay constant.
When using `Vec`, this means manually multiplying your position by `screen_height/screen_width` to normalize everything.

As this is something that all games need, Vectarine provides a shortcut: Coordinates!
Coordinates come from the `@vectarine/coord` module and allow you to refer to position and distance on the screen in the way you like.

```lua
local Coord = require('@vectarine/coord')
local Debug = require('@vectarine/debug')
local Graphics = require('@vectarine/graphics')

function Update(time_delta)
    Graphics.Clear({ r = 1, g = 1, b = 1, a = 1 })
    local rect_color = { r = 1, g = 0, b = 0, a = 1 } -- red

    local pos = Coord.gl(0, 0) -- refer to the center of the screen
    local other_pos = Coord.px(200, 200) -- refers to position (200,200), in pixels, from the top-left
    Debug.print(pos:px()) -- print the corresponding pixel position as a regular vector

    -- Draw a square at the center of the screen, with size 200px
    Graphics.drawRect(Coord.gl(0, 0) - Coord.pxDelta(100, 100), Coord.pxDelta(100, 100), rect_color)

    local pos2 = Coord.px(100, 100) -- refer to a position in pixels
    local size2 = Coord.glDelta(1, 1) -- a quarter of the screen
    Graphics.drawRect(pos2, size2, rect_color)
end
```

You can use `px`, `gl`, `vw` and `vh` to define position on the screen using the coordinate system you prefer.
Same for screen vectors, just use `pxDelta`, `glDelta`, etc...
You can add or remove a screen vector to a screen position to get another position. In general, you can perform the usual operations you'd expect with them.

`Graphics` contains a lot of other functions to draw images, text, arrows, or polygons. See [luau-api/graphics.luau](./luau-api/graphics.luau) for the full list.
All functions can use `Vec` or `ScreenPosition` to draw things. Use the style you prefer!

## Interacting with the user

The functions to get user input are inside the `Io` module.
There are many functions inside `Io`, which won't all get
listed as they have explicit names, but we'll show the main
ones and how to use them.

**Getting the position of the mouse**

```lua
local Io = require("@vectarine/io")
local Vec = require("@vectarine/vec")
local Debug = require("@vectarine/debug")

function Update()
    local m: Vec.Vec2 = Io.getMouse()
    Debug.fprint(m) -- Print the position of the mouse on every frame
    -- Draw a green circle at the position of the cursor.
    Graphics.drawCircle(m, 0.1, { r = 0, g = 1, b = 0, a = 1 })
end
```

**Checking if a key is pressed**

```lua
local Io = require("@vectarine/io")
local Vec = require("@vectarine/vec")
local Debug = require("@vectarine/debug")

function Update()
    local isSpacePressed = Io.isKeyDown("space")
    -- Draw a rectangle when space is pressed
    if isSpacePressed then
        Graphics.drawRect(Vec.V2(0, 0), Vec.V2(0.1, 0.2), { r = 1, g = 0, b = 0, a = 1 })
    end
    -- Print pressed keys
    Debug.fprint(Io.getKeysDown())
    -- Print which mouse buttons are pressed
    Debug.fprint(Io.getMouseState())
end
```

Sometimes, instead of checking every frame is a button is pressed, you want to perform something only once it
is pressed. To do this, you can use _events_.

```lua
local Event = require("@vectarine/event")
local Debug = require("@vectarine/debug")

-- Notice that we subscribe to the event only once, not on every frame!
local counter = 0
Event.getKeyDownEvent():on(function(key: string)
    -- This is called once per press.
	Debug.print("Key down: ", key)
    counter = counter + 1
end)

function Update()
    Debug.fprint("Count: ", counter)
end

```

The _Event_ module has multiple useful events you can subscribe to.
You can also create your own events using `Event.newEvent("name")` if you need to.

> ℹ️ Sometimes, you commonly want to perform an action when debugging
> This can be spawning a specific enemy, teleporting to a location or resetting the state to some value.
> You can use the `Event.getConsoleCommandEvent()` event to listen to what you are typing inside the console
> and trigger specific helpful behavior.

## Global and Local variables

In luau, variables and functions are global by default. You can make them local by adding the `local` keyword before defining them.

```lua
local Debug = require("@vectarine/debug")

myGlobalVar = 3
local someLocalNumber = 10

-- To be explicit when defining globals, we usually use the syntax _G.variableName = value
-- _G is the global object
someLocalNumber = 11 -- Changing the value of an existing variable
_G.globalValue = "abc" -- Setting a global value

function thisIsGlobal()
    Debug.print("a global function is called!")
end

local function thisIsLocal()
    Debug.print("a local function is called!")
end
```

When possible, prefer using local variables. This prevents you from overwriting a variable by mistake by creating 2 variables with the same name in two different functions.
However, global variables have some advantages.

First, you can inspect and edit the value of a global variable in the _Watcher_ tool (Open using <kbd>Ctrl</kbd>+<kbd>3</kbd>)

Second, the value of global variables is preserved between script reloads. This is useful when developing as there is usually part of your state that you
want to reset when reloading and part that you want to keep.

My recommendation is to use the following pattern:

```lua
-- Let's say that we want to preserve the player position between reloads.

function initFromGlobal<T>(defaultValue: T, globalName: string): T
	if _G[globalName] == nil then
		_G[globalName] = defaultValue
	end
    return _G[globalName]
end

-- player_pos is both a local and a global.
-- Because it is local that you get proper typing and information if it is unused
-- Because it is global, its value is preserved between reloads and you can edit it inside the watcher tool
local playerPos = initFromGlobal(Vec.V2(0, 0), "playerPos")

```

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

Once a script is loaded, all future calls to `loadScript` with the same path will return a handle to the same resource and are instant.

By default all non-local variables and functions are shared between files.
This has pros and cons:

- If you have one big file, you can just copy and paste chunks into other files and everything will still work
- If two global functions have the same name, they will override each-other and the last one defined will win.
- You don't get typing across file boundaries, so you will get 'unknown global' errors from Luau despite everything working fine.
- Global variables have any types, so it you change them somewhere, you won't get errors about incorrect use elsewhere.

Because of that, we recommend doing the following (this is just a recommendation, you do you!):

- Keep functions local whenever possible using the `local function(...) function_content() end` syntax.
- Use the `_G.aa = bb` syntax to be explicit when defining globals.
- Use `require` to import types between modules
- When calling `loadScript`, pass the require call as the second argument to gather the exports of the script with proper types

There is a simple example with 2 files: `helper.luau` and `main.luau`.

```lua
-- helper.luau
local module = {} -- This is where all our exports will go
local my_value = 3

-- add_things is inside module, it gets exported
function module.add_things(a: number, b: number): number
    -- Notice that despite my_value not being exported, it can be used inside exported functions!
    return a + b + my_value
end

export type myType = "abc" | "def"

-- _G.helper = module -- optional: export the module table to the global namespace '_G'
return module -- return for the module for typing
```

```lua
-- main.luau
local Resources = require('@vectarine/resources')

--- We use the import 'technique'
local helperResource, helper = Resources.loadScript("scripts/helper.luau", require("helper.luau"))

--- Resources.loadScript is what actually executes `helper.luau`.
--- require() returns an empty table, but is properly typed.
--- When a table is passed as the second argument to loadScript, it is filled with the exports of the script.
--- This gives the impression that require() returns the exports of the script, but it does not.

--- Note that the helper variable is still empty until the resource is ready.

--- Also, note that `helper.luau` is only executed once. If you rerun loadScript, you'll get a handle to the same resource.
--- However, helper will always be filled with the latest exports of the script, even if it is reloaded.
--- This only works if the script returns a table, otherwise, this is ignored.

-- We can still use the types defined inside helper!
local s: helper.myType = "abc"

function Update()
    if Resources.isResourceReady(helperResource) then
        --- You can retrieve functions from _G.helper with the proper type using this syntax:
        --- local add_things: typeof(helper.add_things) = _G.helper.add_things
        --- Alternatively, you can access `helper` directly because you put `helper` as an argument to `loadScript`.

        fprint(helper.add_things(1, 2)) -- prints 1+2+3 = 6
    end
end
```

## Organising rendering using Screens

## Using Shaders

(Fragment) Shaders are little programs that are executed by the GPU and which run on every pixel of an input image.
They are useful to apply custom graphics effects like blurs, outlines, sepia filters, recoloring images, etc...

In Vectarine, shaders are attached to a canvas, a custom drawing surface.

As an example, Let's use this shader which applies a wave deformation effect to its input

```c
precision mediump float; // you need to specify this for your shader to run in a browser
in vec2 uv; // input position
uniform sampler2D tex; // input image (the content of the canvas)
uniform float iTime; // you have access to a time variable in all shaders to apply dynamic effects
out vec4 frag_color; // output color

void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    fragColor = texture(tex, vec2(uv.x + cos((uv.y*4.0+iTime)*10.0)/100.0, uv.y + sin((uv.x*4.0+iTime)*10.0)/100.0));
}
void main() {
    mainImage(frag_color, uv);
}
```

You can put it inside `gamedata/shaders/wave.glsl`.

Then you can use it like so:

```lua
local Canvas = require("@vectarine/canvas")
local Graphics = require("@vectarine/graphics")
local Vec = require("@vectarine/vec")
local V2 = Vec.V2
local shaderResource = Resource.loadShader("shaders/wave.glsl")

-- Create a canvas of size 1200x800 pixels and attach the shader to it.
local canvas = Canvas.createCanvas(1200, 800)
canvas:setShader(shaderResource)
-- You can call canvas:setShader(nil) to stop using a given shader and draw the content of the canvas as-is.

function Update()
    -- Now, we can draw to the canvas
    Canvas.paint(canvas, function()
        -- Every call to Graphics.drawSomething inside paint will be not displayed on the screen
        -- It will be drawn to the canvas instead
        -- Note: you can draw the content of one canvas to another canvas to chain multiple shader effects.
        Graphics.drawRect(V2(0, 0), V2(0.1, 0.1), {r = 1, g = 0, b = 0, a = 1})
    end)

    -- The canvas can be drawn like an image.
    -- When it is drawn, the shader is executed
    Graphics.drawCanvas(canvas, V2(-1, -1), V2(2, 2))
end
```

You can find more information about shaders in [the great book of shaders](https://thebookofshaders.com/)

## Performance Tips

Call at `graphics.drawRect` at most 20 000 times per frame for 60 fps on all platforms.
I don't really know why you'd want to draw that many rectangles.

If you want to draw something on every pixel, use a Shader instead of called `graphics.drawRect` on a per-pixel basis!

## 👷 Writing automatic tests

> ❓ What are automatic tests?

Automatic tests are piece of code that make sure that parts of your game behave correctly.
You could test that 2 systems in your game interact as intended, for example that launching a fireball sets grass on fire (thus testing
that the projectile system and the fire spreading system work together).

Automatic test run automatically and allow you to quickly catch **regression** bugs, things that worked previously, but don't anymore.

> ❓ Why write automatic tests?

If you are working on a small project, like a jam, or are working alone, there is **no reasons to write tests**! **Don't do it**, you'll waste time you could have used
to improve your game!

Tests are useful for large games, for example multi-year projects or when you have a lot of people (more than 2 programmers).
When you feel like you are spending a lot of time debugging a part of your game instead of adding feature, it is a sign that this parts needs tests.
This is especially true if this is a part that you (or somebody else!) wrote a few months, or years ago.

Tests need to be put inside the `tests` folder of your game. They are not exported in the final build. You can run the tests in the editor.

TODO: Design an API for `assert` and general test organisation. The editor needs to work as a CLI to run the tests in watch mode.

## 👷 Networking and Multiplayer

TODO
