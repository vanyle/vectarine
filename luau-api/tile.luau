local ResourceTypes = require("@vectarine/resource")
local vec = require("@vectarine/vec")

--[[
Tilesets and tilemaps, used to draw sprite sheets and levels made of a grid of tiles.

A tileset describes how one image is cut into equal tiles and gives names to some of them. Load one with
`loader.loadTileset`, combine it with its image using `ImageResource:withTileset` and draw tiles with
`ImageWithTileset:drawTile`. A tilemap is a grid of tile ids that represents a level. Load one made in Tiled with
`loader.loadTilemap`, or generate one with `tile.createGeneratedTilemap`, and read it with `TilemapResource:getTile`.
See the `image` module to draw tiles.

# Example

```luau
const loader = require("@vectarine/loader")
const vec = require("@vectarine/vec")

const tilesImage = loader.loadImage("textures/tiles.png", false)
const tilesData = loader.loadTileset("textures/tiles.tsx")
const tiles = tilesImage:withTileset(tilesData)
const level = loader.loadTilemap("levels/level.tmx")

function Update(deltaTime: number)
	if not level:isReady() or not tilesData:isReady() then
		return
	end
	tiles:drawTile(level:getTile(0, 0, 0), vec.V2(-0.5, 0), vec.V2(0.1, 0.1))
end
```

# Common mistakes

- `TilemapResource:getTile` returns `0` for empty cells, and `0` is also the id of the first tile of the tileset.
- Layers are numbered from `0`, and `TilemapResource:getTilemapPart` does not include its `highx` and `highy` bounds.
]]
local module = {}

--MARK: Tileset
local TilesetResourceImpl = { tileset = true }
TilesetResourceImpl.__index = TilesetResourceImpl

--- A tileset describes how an image is cut into equal rectangular tiles, and the names of some tiles.
---
--- You get one from `loader.loadTileset`, from a `.tsx` file made with Tiled. Combine it with the image using
--- `ImageResource:withTileset` to draw tiles, or read its data with `TilesetResource:getTile`. Drawing many tiles
--- from the same image is more efficient than drawing many separate images.
export type TilesetResource = typeof(setmetatable({}, TilesetResourceImpl)) & ResourceTypes.Resource

--- Returns the size of one tile in pixels.
---
--- Returns `V2(0, 0)` if the tileset is not loaded yet.
---
--- @return The width and height of a tile, in pixels.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- const tilesData = loader.loadTileset("textures/tiles.tsx")
---
--- function Update(deltaTime: number)
--- 	local size = tilesData:getTileSize()
--- 	print(size.x, size.y) -- for example 64 64
--- end
--- ```
function TilesetResourceImpl:getTileSize(): vec.Vec2
	error("Implemented in native code")
end

--- Returns how many tiles fit in the image along each axis.
---
--- It is the size of the image divided by the size of a tile. Spacing and margin are not taken into account. For
--- example, with `8` tiles per row and `4` per column, it returns `V2(8, 4)`. Returns `V2(0, 0)` if the tileset is not loaded
--- yet or has no image.
---
--- @return The number of tiles along the horizontal and vertical axes.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- const tilesData = loader.loadTileset("textures/tiles.tsx")
---
--- function Update(deltaTime: number)
--- 	local ratio = tilesData:getTileRatio()
--- 	print(ratio.x, ratio.y) -- for example 8 4
--- end
--- ```
function TilesetResourceImpl:getTileRatio(): vec.Vec2
	error("Implemented in native code")
end

--- Returns the data of a tile, or `nil` if the tileset has no data for this tile.
---
--- The position is the top left corner of the tile in the image, given as a ratio of the image size: `(0, 0)` is the
--- top left corner of the image. It is computed from the tile size only, spacing and margin are not taken into
--- account. The result is a table with these fields:
---
--- - `pos`: the position of the tile in the image, as a ratio.
--- - `probability`: the probability value stored in the tileset file for this tile.
--- - `type`: the name given to the tile in the tileset file.
---
--- Returns `nil` if the tileset is not loaded yet, has no image, or does not define this tile id.
---
--- @param tileId The id of the tile, counting from `0` along each row of the image.
--- @return The table described above, or `nil`.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- const tilesData = loader.loadTileset("textures/tiles.tsx")
---
--- function Update(deltaTime: number)
--- 	local tile = tilesData:getTile(3)
--- 	if tile then
--- 		print(tile.type) -- for example "bee_monster"
--- 	end
--- end
--- ```
function TilesetResourceImpl:getTile(tileId: number): {
	pos: vec.Vec2,
	probability: number,
	type: string,
}?
	error("Implemented in native code")
end

--- Returns the ids of the tiles defined in the tileset.
---
--- Only tiles that have an entry in the tileset file are listed, and their order is not specified. Returns an empty
--- list if the tileset is not loaded yet.
---
--- @return A list of tile ids.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- const tilesData = loader.loadTileset("textures/tiles.tsx")
---
--- function Update(deltaTime: number)
--- 	print(#tilesData:getTiles()) -- how many tiles are defined
--- end
--- ```
function TilesetResourceImpl:getTiles(): { number }
	error("Implemented in native code")
end

--MARK: Tilemap
local TilemapResourceImpl = { tilemap = true }
TilemapResourceImpl.__index = TilemapResourceImpl

--- A tilemap is a grid of tile ids with several layers, used to represent a level.
---
--- You get one from `loader.loadTilemap`, from a `.tmx` file made with Tiled or another level editor. Read it with
--- `TilemapResource:getTile` and `TilemapResource:getTilemapPart`, and draw the ids with
--- `ImageWithTileset:drawTile`. To build a tilemap from code, see `tile.createGeneratedTilemap`. The cells of a layer are
--- addressed by integer coordinates `x` and `y`, and layers are numbered from `0`.
export type TilemapResource =
	typeof(setmetatable({} :: typeof(TilemapResourceImpl), TilemapResourceImpl))
	& ResourceTypes.Resource

--- A tilemap built by a Lua function instead of being loaded from a file.
---
--- You get one from `tile.createGeneratedTilemap`. It is read like a `TilemapResource`
--- (see `GeneratedTilemap:getTile`), but it is not a resource: it is ready as soon as it is created. It has no limits, so it can
--- represent an endless world.
local GeneratedTilemapImpl = { generatedtilemap = true }
GeneratedTilemapImpl.__index = GeneratedTilemapImpl

export type GeneratedTilemap = typeof(setmetatable({} :: typeof(GeneratedTilemapImpl), GeneratedTilemapImpl))

--- Either a `TilemapResource` or a `GeneratedTilemap`. Both can be read the same way.
export type Tilemap = TilemapResource | GeneratedTilemap

--- Creates a tilemap whose tiles come from a function.
---
--- The tilemap is infinite, and ready to use immediately. `tilemapGenerator` is called when a part of the tilemap is
--- needed for the first time, and its result is cached. It is called with the layer and with the `x` and `y` of the
--- top left cell of a 16x16 block. These are always multiples of `16` and can be negative. It must return a list of `256`
--- numbers for this block: the tile id of the cell (`x + i`, `y + j`) is `result[1 + i + j * 16]`. Missing entries
--- count as `0`.
---
--- Use it to build worlds from `math.random` or a noise function, or to merge several tilemaps by calling `getTile` on
--- each of them in the generator. Call `GeneratedTilemap:invalidate` to make the generator run again for a block.
---
--- @param tilemapGenerator The function that returns the tile ids of a 16x16 block of the given layer.
--- @return A new tilemap.
---
--- # Errors
---
--- An error raised inside the generator is not reported. The block is treated as empty (all tiles are `0`), and the
--- generator runs again the next time the block is needed.
---
--- # Example
---
--- ```luau
--- const tile = require("@vectarine/tile")
---
--- local world = tile.createGeneratedTilemap(function(layer: number, x: number, y: number)
--- 	local block = {}
--- 	for i = 1, 256 do
--- 		block[i] = if math.random() < 0.1 then 2 else 1
--- 	end
--- 	return block
--- end)
--- print(world:getTile(0, 100, -40)) -- 1 or 2
--- ```
---
--- # See also
---
--- - `GeneratedTilemap:invalidate`: makes a block generate again.
function module.createGeneratedTilemap(
	tilemapGenerator: (layer: number, x: number, y: number) -> { number }
): GeneratedTilemap
	error("Implemented in native code")
end

--- Forgets the cached tiles around a cell, so that the generator runs again for them.
---
--- The cache works by blocks of 16x16 cells, so the whole block that contains the cell is forgotten. Calling it for a
--- block that is not cached does nothing. Use it after the data your generator depends on has changed.
---
--- @param layer The layer of the cell.
--- @param x The horizontal coordinate of the cell.
--- @param y The vertical coordinate of the cell.
---
--- # Example
---
--- ```luau
--- const tile = require("@vectarine/tile")
---
--- local seed = 1
--- local world = tile.createGeneratedTilemap(function(layer: number, x: number, y: number)
--- 	local block = {}
--- 	for i = 1, 256 do
--- 		block[i] = seed
--- 	end
--- 	return block
--- end)
--- print(world:getTile(0, 3, 3)) -- 1
--- seed = 2
--- world:invalidate(0, 3, 3)
--- print(world:getTile(0, 3, 3)) -- 2
--- ```
function GeneratedTilemapImpl:invalidate(layer: number, x: number, y: number)
	error("Implemented in native code")
end

--- Returns the id of the tile at a cell of a layer.
---
--- Returns `0` if the cell is empty, if it is outside of the tilemap, if the layer does not exist, if the layer is not a
--- tile layer, or if the tilemap is not loaded yet. Note that `0` is also the id of the first tile of the tileset.
--- Use `TilemapResource:getTilemapPart` to read many cells at once.
---
--- @param layer The layer number, starting at `0`.
--- @param x The horizontal coordinate of the cell.
--- @param y The vertical coordinate of the cell.
--- @return The tile id, or `0`.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- const level = loader.loadTilemap("levels/level.tmx")
---
--- function Update(deltaTime: number)
--- 	local id = level:getTile(0, 1, 2) -- layer 0, x = 1, y = 2
--- 	print(id)
--- end
--- ```
---
--- # See also
---
--- - `TilemapResource:getTilemapPart`: reads many cells at once.
--- - `ImageWithTileset:drawTile`: draws a tile from its id.
function TilemapResourceImpl:getTile(layer: number, x: number, y: number): number
	error("Implemented in native code")
end

--- The content of an object layer of a tilemap: a named layer holding a position, an optional text and extra data.
---
--- It is the value returned by `TilemapResource:getObjectLayer`.
type ObjectLayer = {
	name: string,
	text: string?,
	position: vec.Vec2,
	extra: unknown,
}

--- Returns an object layer of the tilemap.
---
--- This method is declared but not implemented yet: the engine does not provide it, so calling it raises an error.
--- To read objects from a `.tmx` file, store them in a separate file or read the file as text.
---
--- @param layer The layer number.
--- @return An `ObjectLayer` table with a `name`, an optional `text`, a `position` and an `extra` value.
function TilemapResourceImpl:getObjectLayer(layer: number): ObjectLayer
	error("Implemented in native code")
end

--- Returns the id of the tile at a cell of a layer.
---
--- It calls the generator for the 16x16 block of the cell if that block is not cached yet. Returns `0` if the generator
--- gave no tile for the cell or failed. Use `GeneratedTilemap:getTilemapPart` to read many cells at once.
---
--- @param layer The layer number. It is given to the generator as is.
--- @param x The horizontal coordinate of the cell.
--- @param y The vertical coordinate of the cell.
--- @return The tile id, or `0`.
---
--- # Example
---
--- ```luau
--- const tile = require("@vectarine/tile")
---
--- local world = tile.createGeneratedTilemap(function(layer: number, x: number, y: number)
--- 	local block = {}
--- 	for i = 1, 256 do
--- 		block[i] = 5
--- 	end
--- 	return block
--- end)
--- print(world:getTile(0, -20, 7)) -- 5
--- ```
function GeneratedTilemapImpl:getTile(layer: number, x: number, y: number): number
	error("Implemented in native code")
end

--- Calls a function for every cell in a rectangle of a layer.
---
--- `accessFunction` is called with the tile id and the coordinates of each cell, for `lowx <= x < highx` and
--- `lowy <= y < highy`. The high bounds are not included. The order of the calls is not specified. This is faster than
--- calling `TilemapResource:getTile` in a loop, so use it to draw a visible part of a level. Cells that are not defined
--- in the tilemap are skipped, so the function may be called for fewer cells than the rectangle contains.
---
--- @param layer The layer number, starting at `0`.
--- @param lowx The first column to visit.
--- @param lowy The first row to visit.
--- @param highx The column after the last one to visit.
--- @param highy The row after the last one to visit.
--- @param accessFunction The function called as `accessFunction(tileId, x, y)` for each cell.
---
--- # Errors
---
--- Raises an error if the tilemap is not loaded yet, or if the layer does not exist or is not a tile layer. Errors
--- raised by `accessFunction` stop the visit and are raised again.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
--- const vec = require("@vectarine/vec")
---
--- const tilesImage = loader.loadImage("textures/tiles.png", false)
--- const tiles = tilesImage:withTileset(loader.loadTileset("textures/tiles.tsx"))
--- const level = loader.loadTilemap("levels/level.tmx")
---
--- function Update(deltaTime: number)
--- 	if not level:isReady() then
--- 		return
--- 	end
--- 	level:getTilemapPart(0, 0, 0, 10, 10, function(tileId: number, x: number, y: number)
--- 		tiles:drawTile(tileId, vec.V2(x * 0.1 - 0.5, y * 0.1 - 0.5), vec.V2(0.1, 0.1))
--- 	end)
--- end
--- ```
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

--- Calls a function for every cell in a rectangle of a layer.
---
--- `accessFunction` is called with the tile id and the coordinates of each cell, for `lowx <= x < highx` and
--- `lowy <= y < highy`. The high bounds are not included. The order of the calls is not specified. The generator
--- runs for the blocks that are not cached yet. Cells for which the generator gave no tile are skipped.
---
--- @param layer The layer number. It is given to the generator as is.
--- @param lowx The first column to visit.
--- @param lowy The first row to visit.
--- @param highx The column after the last one to visit.
--- @param highy The row after the last one to visit.
--- @param accessFunction The function called as `accessFunction(tileId, x, y)` for each cell.
---
--- # Errors
---
--- Errors raised by `accessFunction` stop the visit and are raised again.
---
--- # Example
---
--- ```luau
--- const tile = require("@vectarine/tile")
---
--- local world = tile.createGeneratedTilemap(function(layer: number, x: number, y: number)
--- 	local block = {}
--- 	for i = 1, 256 do
--- 		block[i] = 1
--- 	end
--- 	return block
--- end)
--- local count = 0
--- world:getTilemapPart(0, 0, 0, 4, 4, function(tileId: number, x: number, y: number)
--- 	count += 1
--- end)
--- print(count) -- 16
--- ```
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
