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

--- Loads and runs a Lua file at the given path.
--- If the script returns a table, it will be merged with the `results` table if provided.
--- The return value is a ScriptResource
--- @param path string For example scripts/monster.lua
function module.loadScript<T>(path: string, type_hint: T): (Res.ScriptResource, T)
	error("Not implemented")
end

--- Load an image from a path
--- If antialiasing is true, the image will be drawn with linear filtering, otherwise with nearest filtering.
--- If you are using a pixel art style, you probably want antialiasing to be false.
--- @param path string
--- @return ImageResource
function module.loadImage(path: string, antialiasing: boolean?): Image.ImageResource
	error("Not implemented")
end

--- Load a font from a path
--- @param path string
--- @return FontResource
--- @nodiscard
function module.loadFont(path: string): Text.FontResource
	error("Not implemented")
end

--- Load text from a path.
--- The path should be relative to the game.vecta file, like "./texts/mytext.txt"
--- @param path string
--- @return TextResource
--- @nodiscard
function module.loadText(path: string): Res.TextResource
	error("Not implemented")
end

--- Load a shader from a path
--- @param path string
--- @return ShaderResource
--- @nodiscard
function module.loadShader(path: string): Canvas.ShaderResource
	error("Not implemented")
end

--- Load audio from a path
--- @param path string
--- @return AudioResource
function module.loadAudio(path: string): Audio.AudioResource
	error("Not implemented")
end

--- Load a tileset from a path
--- @param path string
--- @return TilesetResource
function module.loadTileset(path: string): Tile.TilesetResource
	error("Not implemented")
end

--- Load a tilemap from a path
--- @param path string
--- @return TilemapResource
function module.loadTilemap(path: string): Tile.TilemapResource
	error("Not implemented")
end

return module
