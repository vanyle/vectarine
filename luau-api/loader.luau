local Audio = require("@vectarine/audio")
local Canvas = require("@vectarine/canvas")
local Image = require("@vectarine/image")
local Res = require("@vectarine/resource")
local Text = require("@vectarine/text")
local Tile = require("@vectarine/tile")

--- Loader
---
--- Functions to load external resources like images, scripts, fonts, shaders, audio, etc.
local module = {}

--- Loads and runs a Lua file at the given path.$
--- For typing purposes, you can pass a table as the second argument, but this has no runtime effect.
--- The return value is a ScriptResource and the module table returned by the script.
---
--- Example:
--- ```luau
--- local loader = require("@vectarine/loader")
--- -- We pass require("other.luau") as the second argument to get proper type inference for the returned module "other".
--- local res, other = loader.loadScript("scripts/other.luau", require("other.luau"))
--- ```
--- @param path For example scripts/monster.lua
--- @param type_hint A table to hint the type of the module returned by the script. This has no runtime effect.
function module.loadScript<T>(path: string, type_hint: T): (Res.ScriptResource, T)
	error("Implemented in native code")
end

--- Load an image from a path
--- If antialiasing is true, the image will be drawn with linear filtering, otherwise with nearest filtering.
--- If you are using a pixel art style, you probably want antialiasing to be false.
--- @param path string
--- @return ImageResource
function module.loadImage(path: string, antialiasing: boolean?): Image.ImageResource
	error("Implemented in native code")
end

--- Load a font from a path
--- @param path string
--- @return FontResource
--- @nodiscard
function module.loadFont(path: string): Text.FontResource
	error("Implemented in native code")
end

--- Load text from a path.
--- The path should be relative to the game.vecta file, like "./texts/mytext.txt"
--- @param path string
--- @return TextResource
--- @nodiscard
function module.loadText(path: string): Res.TextResource
	error("Implemented in native code")
end

--- Load a shader from a path
--- @param path string
--- @return ShaderResource
--- @nodiscard
function module.loadShader(path: string): Canvas.ShaderResource
	error("Implemented in native code")
end

--- Load audio from a path
--- @param path string
--- @return AudioResource
function module.loadAudio(path: string): Audio.AudioResource
	error("Implemented in native code")
end

--- Load a tileset from a path
--- @param path string
--- @return TilesetResource
function module.loadTileset(path: string): Tile.TilesetResource
	error("Implemented in native code")
end

--- Load a tilemap from a path
--- @param path string
--- @return TilemapResource
function module.loadTilemap(path: string): Tile.TilemapResource
	error("Implemented in native code")
end

--- Return a table which is referentially stable across script reloads for a given script, or nil if the script has never been loaded before.
--- You should use this table to store the functions and properties declared in your script.
--- ```luau
--- local loader = require("@vectarine/loader")
--- local module = loader.init() or {}
--- if not module.hasBeenLoaded then
---		--- This module has been loaded for the first time!
--- end
--- --- Define all the properties of your module here
--- module.hasBeenLoaded = true
--- return module
--- ```
function module.init(): nil
	error("Implemented in native code")
end

return module
