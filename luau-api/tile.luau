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
function TilesetResourceImpl.getTileSize(): Vec.Vec2
	error("Not implemented")
end

--- Get the size of a tile, as the ratio of the full image.
--- For example, if you have 8 tiles per row and column, this will return V2(1/8, 1/8)
function TilesetResourceImpl.getTileRatio(): Vec.Vec2
	error("Not implemented")
end

--- Get the position of a tile in the image.
--- The position is returned as a ratio of the full image.
function TilesetResourceImpl.getTile(
	self: TilesetResource,
	tileId: number
): {
	pos: Vec.Vec2,
	probability: number,
	type: string,
}?
	error("Not implemented")
end

--- Return a list of tile ids defined in the tileset provided.
function TilesetResourceImpl.getTiles(self: TilesetResource): { number }
	error("Not implemented")
end

--MARK: Tilemap
local TilemapResourceImpl = { tilemap = true }
TilemapResourceImpl.__index = TilemapResourceImpl

--- A tilemap is a 3d grid of integers which can be used to represent levels.
--- Tilemaps can be imported from .tmx files, generated on the fly or a combination of both.
--- Tilemaps represents their data efficiently and can be infinite in size.
export type TilemapResource = typeof(setmetatable({}, TilemapResourceImpl)) & ResourceTypes.Resource

--- Returns the id of the tile at the location specified.
---
--- Returns 0 if the tilemap is not defined at the provided location, the layer does not exist or if the layer is not a tile layer.
function TilemapResourceImpl.getTile(self: TilemapResource, layer: number, x: number, y: number): number
	error("Not implemented")
end

--- A high-performance way to draw or access a part of a tilemap.
---
--- Call accessFunction for all tiles with coordinates (x,y) such that:
--- lowx <= x <= highx and lowy <= y <= highy
---
--- If the layer does not exist or the coordinates are outside of where the tilemap is defined, the accessFunction might never be called.
function TilemapResourceImpl.getTilemapPart(
	self: TilemapResource,
	layer: number,
	lowx: number,
	lowy: number,
	highx: number,
	highy: number,
	accessFunction: (tileId: number, x: number, y: number) -> ()
)
	error("Not implemented")
end

return module
