local ResourceTypes = require("@vectarine/resource")
local module = {}

--MARK: Tileset
local TilesetResourceImpl = { tileset = true }
TilesetResourceImpl.__index = TilesetResourceImpl

--- A tileset is a mapping from names to rectangular pieces of an image named tiles.
--- You can draw a tile directly or using drawImagePart.
--- Drawing multiple tiles from the same tileset is more efficient than drawing multiple images.
export type TilesetResource = typeof(setmetatable({}, TilesetResourceImpl)) & ResourceTypes.Resource

function TilesetResourceImpl.getTile(
	self: TilesetResource,
	tilename: string
): { x: number, y: number, width: number, height: number }
	error("Not implemented")
end

--MARK: TileId

local TileIdImpl = { tileid = true }
TileIdImpl.__index = TileIdImpl
--- Represents a unique identifier for a tile in a tileset.
--- This is just a positive integer. You can use the `id` function to get it.
--- TileId 0 represents the absence of a tile.
export type TileId = typeof(setmetatable({}, TileIdImpl))

--- Convert a TileId to an integer.
function TileIdImpl.id(self: TileId): number
	error("Not implemented")
end

--- Convert an integer to a TileId. Floats will be rounded down and negative numbers will become 0.
function module.tileId(id: number): TileId
	error("Not implemented")
end

--MARK: Tilemap
local TilemapResourceImpl = { tilemap = true }
TilemapResourceImpl.__index = TilemapResourceImpl

--- A tilemap is a 3d grid of integers which can be used to represent levels.
--- Tilemaps can be imported from .tmx files, generated on the fly or a combination of both.
--- Tilemaps represents their data efficiently and can be infinite in size.
export type TilemapResource = typeof(setmetatable({}, TilemapResourceImpl)) & ResourceTypes.Resource

function TilemapResourceImpl.getTile(self: TilemapResource, layer: number, x: number, y: number): TileId
	error("Not implemented")
end

--- A high-performance way to draw or access a part of a tilemap
function TilemapResourceImpl.accessTilemapPart(
	self: TilemapResource,
	layer: number,
	lowx: number,
	lowy: number,
	highx: number,
	highy: number,
	accessFunction: (tileId: TileId, x: number, y: number) -> ()
)
	error("Not implemented")
end

--- Create a new tilemap from a generation function.
--- The tilemap can use caching, so the generationFunction may not be called every time a tile is accessed.
function module.createTilemap(generationFunction: (layer: number, x: number, y: number) -> TileId): TilemapResource
	error("Not implemented")
end

return module
