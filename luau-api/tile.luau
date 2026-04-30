local ResourceTypes = require("@vectarine/resource")
local Vec = require("@vectarine/vec")
local module = {}

--MARK: Tileset
local TilesetResourceImpl = { tileset = true }
TilesetResourceImpl.__index = TilesetResourceImpl

--- A tileset is a mapping from names to rectangular pieces of an image named tiles.
--- You can draw a tile directly or using drawImagePart.
--- Drawing multiple tiles from the same tileset is more efficient than drawing multiple images.
export type TilesetResource = typeof(setmetatable({}, TilesetResourceImpl)) & ResourceTypes.Resource

--- Get the size of a tile in pixels
function TilesetResourceImpl:getTileSize(): Vec.Vec2
	error("Implemented in native code")
end

--- Get the size of a tile, as the ratio of the full image.
--- For example, if you have 8 tiles per row and column, this will return V2(1/8, 1/8)
function TilesetResourceImpl:getTileRatio(): Vec.Vec2
	error("Implemented in native code")
end

--- Get the position of a tile in the image.
--- The position is returned as a ratio of the full image.
function TilesetResourceImpl:getTile(tileId: number): {
	pos: Vec.Vec2,
	probability: number,
	type: string,
}?
	error("Implemented in native code")
end

--- Return a list of tile ids defined in the tileset provided.
function TilesetResourceImpl:getTiles(): { number }
	error("Implemented in native code")
end

--MARK: Tilemap
local TilemapResourceImpl = { tilemap = true }
TilemapResourceImpl.__index = TilemapResourceImpl

--- A tilemap is a 3d grid of integers which can be used to represent levels.
--- Tilemaps can be imported from .tmx files, generated on the fly or a combination of both.
--- Tilemaps represents their data efficiently and can be infinite in size.
export type TilemapResource =
	typeof(setmetatable({} :: typeof(TilemapResourceImpl), TilemapResourceImpl))
	& ResourceTypes.Resource

--- A generated tilemap is a tilemap that is generated on the fly using a function rather than loaded from a file.
--- A generated tilemap is not a resource.
local GeneratedTilemapImpl = { generatedtilemap = true }
GeneratedTilemapImpl.__index = GeneratedTilemapImpl

export type GeneratedTilemap = typeof(setmetatable({} :: typeof(GeneratedTilemapImpl), GeneratedTilemapImpl))

export type Tilemap = TilemapResource | GeneratedTilemap

--- Create a tilemap resource from a function. The tilemap created is instantly loaded.
--- The tilemapGenerator is a function that generates the part of the tilemap requested.
--- For performance reasons, it should return a 256 list of numbers representing a 16x16 part of the tilemap (of the form `result[1 + x + y * 16]`).
--- You can assume that x and y will always be multiples of 16 (but they can be negative).
--- The tilemap is infinite in size. The tilemapGenerator is called as needed and its result is cached.
---
--- You can use this to merge tilemaps together by calling `getTile` on multiple tilemaps in the generator. You can proceduraly generate tilemaps using the `noise` function.
function module.createGeneratedTilemap(
	tilemapGenerator: (layer: number, x: number, y: number) -> { number }
): GeneratedTilemap
	error("Implemented in native code")
end

--- Invalidate the cache of a part of the tilemap.
---
--- The next time this part is needed, the tilemapGenerator will be called again to generate it.
---
--- Because of how caching works, invalidating a location will also invalidate a 16x16 chunk which contains it.
--- Invalidating a location that is already invalidated has no effect.
function GeneratedTilemapImpl:invalidate(layer: number, x: number, y: number)
	error("Implemented in native code")
end

--- Returns the id of the tile at the location specified.
---
--- Returns 0 if the tilemap is not defined at the provided location, the layer does not exist or if the layer is not a tile layer.
function TilemapResourceImpl:getTile(layer: number, x: number, y: number): number
	error("Implemented in native code")
end

--- Returns the id of the tile at the location specified.
---
--- Returns 0 if the tilemap is not defined at the provided location, the layer does not exist or if the layer is not a tile layer.
function GeneratedTilemapImpl:getTile(layer: number, x: number, y: number): number
	error("Implemented in native code")
end

--- A high-performance way to draw or access a part of a tilemap.
---
--- Call accessFunction for all tiles with coordinates (x,y) such that:
--- lowx <= x <= highx and lowy <= y <= highy
---
--- If the layer does not exist or the coordinates are outside of where the tilemap is defined, the accessFunction might never be called.
function TilemapResourceImpl:getTilemapPart(
	layer: number,
	lowx: number,
	lowy: number,
	highx: number,
	highy: number,
	accessFunction: (tileId: number, x: number, y: number) -> ()
)
	error("Implemented in native code")
end

--- A high-performance way to draw or access a part of a tilemap.
---
--- Call accessFunction for all tiles with coordinates (x,y) such that:
--- lowx <= x <= highx and lowy <= y <= highy
---
--- If the layer does not exist or the coordinates are outside of where the tilemap is defined, the accessFunction might never be called.
function GeneratedTilemapImpl:getTilemapPart(
	layer: number,
	lowx: number,
	lowy: number,
	highx: number,
	highy: number,
	accessFunction: (tileId: number, x: number, y: number) -> ()
)
	error("Implemented in native code")
end

return module
