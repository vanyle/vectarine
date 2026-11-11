local Audio = require("@vectarine/audio")
local Canvas = require("@vectarine/canvas")
local Image = require("@vectarine/image")
local Res = require("@vectarine/resource")
local Text = require("@vectarine/text")
local Tile = require("@vectarine/tile")

--[[
Loads the external files of your game: images, audio, fonts, texts, shaders, tilesets, tilemaps and scripts.

Each `load*` function returns a resource right away and reads the file in the background, so the game never freezes
while loading. A resource that is not ready yet does nothing when used. Check it with `Resource:isReady`, or read
`Resource:getStatus` to find out why it failed. Loading a file that does not exist does not raise an error: the status
of the resource becomes an error message.

A path is written from the folder of your game (the folder of the `game.vecta` file), like `"images/hero.png"`. A path
that starts with `./` is relative to the script that calls the function instead. Paths must be lowercase, so that the
game works on every platform.

See the `resource` module for what you can do with a resource, and the `image`, `audio` and `text` modules for using the
loaded data.

# Example

```luau
const loader = require("@vectarine/loader")
const vec = require("@vectarine/vec")

local hero = loader.loadImage("images/hero.png")
function Update(deltaTime: number)
	if hero:isReady() then
		hero:draw(vec.V2(-0.1, -0.1), vec.V2(0.2, 0.2))
	end
end
```

# Common mistakes

- Calling a `load*` function inside `Update`. Loading the same path again returns the same resource, but it still does
  extra work every frame. Load once at the top of the script.
- Using a resource as soon as it is loaded. It is not ready for a few frames, see `Resource:isReady`.
- Using an uppercase letter in a path, like `"Images/Hero.png"`. The resource is rejected because paths must be
  lowercase.
]]
local module = {}

--- Starts loading a Luau script in the background and returns it with the table it exports.
---
--- It works like `require`, but it does not wait for the script: it returns immediately with an empty table, and the
--- script is loaded and run later. Once the script has run, that same table is filled with what the script returns, and
--- it is updated again when the script is hot-reloaded. Use `Resource:isReady` on the returned resource to know when the
--- table is filled. If the script was already requested before, the table of its exports is returned directly.
---
--- Usually, use `require`. Use `loader.loadScript` when you have a lot of scripts and want to load them in the
--- background.
---
--- The second argument only helps the type checker: pass `require("other.luau")` to get the right type for the
--- returned module. It has no effect at runtime.
---
--- @param path The path of the script, for example `"scripts/other.luau"`.
--- @param type_hint A table used only for typing the returned module. Defaults to `nil`.
--- @return A pair: the `ScriptResource`, and the table returned by the script, which is empty until the script is loaded.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- -- The second argument gives the type of the module "other".
--- local res, other = loader.loadScript("scripts/other.luau", require("other.luau"))
--- print(res:isReady())
--- ```
function module.loadScript<T>(path: string, type_hint: T?): (Res.ScriptResource, T)
	error("Implemented in native code")
end

--- Starts loading an image and returns it.
---
--- The image can be drawn once `Resource:isReady` is `true`. Use `ImageResource:draw` to draw it.
---
--- @param path The path of the image file, for example `"images/hero.png"`.
--- @param antialiasing True to draw the image with linear filtering (smooth), `false` for nearest filtering (sharp
--- pixels). Defaults to `nil`, which lets the engine choose. For pixel art, pass `false`.
--- @return The `ImageResource`, not ready yet.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
--- const vec = require("@vectarine/vec")
---
--- local tree = loader.loadImage("images/tree.png", false)
--- function Update(deltaTime: number)
--- 	tree:draw(vec.V2(0, 0), vec.V2(0.3, 0.3))
--- end
--- ```
---
--- # See also
---
--- - `ImageResource:draw`: to draw the image once it is ready.
--- - `Resource:isReady`: to know when it is ready.
function module.loadImage(path: string, antialiasing: boolean?): Image.ImageResource
	error("Implemented in native code")
end

--- Starts loading a font and returns it.
---
--- Use the font with the functions of the `text` module once it is ready.
---
--- @param path The path of the font file, for example `"fonts/main.ttf"`.
--- @return The `FontResource`, not ready yet.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- local font = loader.loadFont("fonts/main.ttf")
--- print(font:isReady()) -- false on the first frame
--- ```
--- @nodiscard
function module.loadFont(path: string): Text.FontResource
	error("Implemented in native code")
end

--- Starts loading a text file and returns it.
---
--- Read its content with `TextResource:getText` once it is ready. Use it for dialogs, levels or data files.
---
--- @param path The path of the text file, for example `"texts/intro.txt"`.
--- @return The `TextResource`, not ready yet.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- local intro = loader.loadText("texts/intro.txt")
--- function Update(deltaTime: number)
--- 	if intro:isReady() then
--- 		print(intro:getText())
--- 	end
--- end
--- ```
--- @nodiscard
---
--- # See also
---
--- - `TextResource:getText`: to read the content.
function module.loadText(path: string): Res.TextResource
	error("Implemented in native code")
end

--- Starts loading a shader and returns it.
---
--- A shader is a text file written in WGSL: see the `canvas` module for what it contains. Use it with the functions of
--- the `canvas` module once it is ready. If the shader has a mistake, the error (with its line) is the status of the
--- resource, shown in the Resources window of the editor.
---
--- @param path The path of the shader file, for example `"shaders/wave.wgsl"`.
--- @return The `ShaderResource`, not ready yet.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- local wave = loader.loadShader("shaders/wave.wgsl")
--- print(wave:getStatus())
--- ```
--- @nodiscard
function module.loadShader(path: string): Canvas.ShaderResource
	error("Implemented in native code")
end

--- Starts loading a sound or a music and returns it.
---
--- Play it with `AudioResource:play` once it is ready.
---
--- @param path The path of the audio file, for example `"audio/theme.ogg"`.
--- @return The `AudioResource`, not ready yet.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- local theme = loader.loadAudio("audio/theme.ogg")
--- function Update(deltaTime: number)
--- 	if theme:isReady() and not theme:isPlaying() then
--- 		theme:play(true)
--- 	end
--- end
--- ```
---
--- # See also
---
--- - `AudioResource:play`: to play the sound.
function module.loadAudio(path: string): Audio.AudioResource
	error("Implemented in native code")
end

--- Starts loading a tileset and returns it.
---
--- A tileset describes the tiles of an image. See the `tile` module to use it.
---
--- @param path The path of the tileset file, made with the Tiled editor.
--- @return The `TilesetResource`, not ready yet.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- local tileset = loader.loadTileset("tiles/ground.tsx")
--- print(tileset:isReady())
--- ```
function module.loadTileset(path: string): Tile.TilesetResource
	error("Implemented in native code")
end

--- Starts loading a tilemap and returns it.
---
--- A tilemap is a grid of tiles, like a level. See the `tile` module to use it.
---
--- @param path The path of the tilemap file, made with the Tiled editor.
--- @return The `TilemapResource`, not ready yet.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- local level = loader.loadTilemap("levels/level1.tmx")
--- print(level:isReady())
--- ```
function module.loadTilemap(path: string): Tile.TilemapResource
	error("Implemented in native code")
end

--- Returns a table that stays the same when the script is hot-reloaded.
---
--- Store the functions and values of your script in this table, so that state survives a reload while you edit the
--- script. The first time the script is loaded, the table is empty. After a reload of the same script, it is the same
--- table, with the values stored before. The type of the return value is `nil` so that `loader.init() or {}` is typed
--- as a table by the type checker.
---
--- @return The table of the current script. It is never `nil` at runtime.
---
--- # Example
---
--- ```luau
--- local loader = require("@vectarine/loader")
--- local module = loader.init() or {}
--- if not module.hasBeenLoaded then
--- 	-- This module has been loaded for the first time!
--- end
--- -- Define all the properties of your module here
--- module.hasBeenLoaded = true
--- return module
--- ```
function module.init(): nil
	error("Implemented in native code")
end

return module
