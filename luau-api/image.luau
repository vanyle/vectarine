local Coord = require("@vectarine/coord")
local ResourceTypes = require("@vectarine/resource")
local Tile = require("@vectarine/tile")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")
local module = {}

local ImageResourceImpl = { image = true }
ImageResourceImpl.__index = ImageResourceImpl
export type ImageResource = typeof(setmetatable({}, ImageResourceImpl)) & ResourceTypes.Resource

type Pos = Coord.ScreenPosition | Vec.Vec2
type Direction = Coord.ScreenVec | Vec.Vec2

--- Draws the image at the position and with the size given
--- You can optionally tint the image with the given color. The color of the pixels of the image
--- are multiplied by the tint.
function ImageResourceImpl.draw(self: ImageResource, pos: Pos, size: Direction, color: Vec4.Vec4?): ()
	error("Implemented in native code")
end

--- Draws a rectangular part of the image delimited by a position and a size
--- This section is deformed to match the quadrilateral delimited by the 4 destination points
--- You can swap the destination points to rotate, flip the image.
--- Use `src_pos = V2(0, 0)` and `src_size = V2(1, 1)` to draw the full image.
function ImageResourceImpl.drawPart(
	self: ImageResource,
	dest_p1: Pos,
	dest_p2: Pos,
	dest_p3: Pos,
	dest_p4: Pos,
	src_pos: Vec.Vec2,
	src_size: Vec.Vec2,
	color: Vec4.Vec4?
): Vec.Vec2
	error("Implemented in native code")
end

--- Gets the size of the image in pixels
function ImageResourceImpl.getSize(self: ImageResource): Vec.Vec2
	error("Implemented in native code")
end

local ImageWithTileset = { imageWithTileset = true }
ImageWithTileset.__index = ImageWithTileset
export type ImageWithTileset = typeof(setmetatable({}, ImageWithTileset))

--- Combine an image with a tileset to be able to easily draw tiles from the image.
--- Methods using the ImageWithTileset require that both the image and the tileset are loaded.
--- They provide convenience over having to specify `src_pos` and `src_size` using `drawPart`.
function ImageResourceImpl:withTileset(tileset: Tile.TilesetResource): ImageWithTileset
	error("Implemented in native code")
end

--- Draw a part of the image corresponding to the tile specified by its id or type.
--- If there are multiple tiles with the same type, one of them will be choosen in an unspecified manner.
--- TODO: We'll need a fastlist version of this.
function ImageWithTileset:drawTile(tileIdOrType: number | string, pos: Pos, size: Direction, color: Vec4.Vec4?): ()
	error("Implemented in native code")
end

--- Draw a part of the image corresponding to the tile specified by its id or type.
--- The part of the image corresponding to the tile is deformed to match the quadrilateral delimited by the 4 destination points.
--- If there are multiple tiles with the same type, one of them will be choosen in an unspecified manner.
function ImageWithTileset:drawTileQuad(
	tileIdOrType: number | string,
	dest_p1: Pos,
	dest_p2: Pos,
	dest_p3: Pos,
	dest_p4: Pos,
	color: Vec4.Vec4?
): ()
	error("Implemented in native code")
end

return module
