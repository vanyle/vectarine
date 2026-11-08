local Coord = require("@vectarine/coord")
local ResourceTypes = require("@vectarine/resource")
local Tile = require("@vectarine/tile")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")

--[[
Draws images and tiles loaded from image files.

You get an `ImageResource` from `loader.loadImage` and draw it with `ImageResource:draw` from `Update`. Images are
loaded in the background: drawing an image that is not loaded yet draws nothing and raises no error. To draw one small
picture out of a larger image (a sprite sheet), use `ImageResource:drawPart`, or combine the image with a tileset using
`ImageResource:withTileset` and draw tiles by name with `ImageWithTileset:drawTile`. Positions and sizes use screen
coordinates, like in the `graphics` module. See the `tile` module for tilesets.

# Example

```luau
const loader = require("@vectarine/loader")
const vec = require("@vectarine/vec")

const ballImage = loader.loadImage("textures/ball.png")

function Update(deltaTime: number)
	ballImage:draw(vec.V2(-0.1, -0.1), vec.V2(0.2, 0.2))
end
```

# Common mistakes

- Calling `ImageResource:getSize` before the image is loaded raises an error. Check `Resource:isReady` first.
- Drawing before the image is loaded shows nothing and gives no error, so a missing picture can also mean a wrong path.
- Pixel art looks blurry with the default filtering. Pass `false` as the second argument of `loader.loadImage`.
]]
local module = {}

local ImageResourceImpl = { image = true }
ImageResourceImpl.__index = ImageResourceImpl
--- An image loaded from a file, ready to be drawn on the screen.
---
--- You get one from `loader.loadImage`. Like every resource, it loads in the background: check
--- `Resource:isReady` if you need to know when it can be used. Draw it with `ImageResource:draw`.
export type ImageResource = typeof(setmetatable({}, ImageResourceImpl)) & ResourceTypes.Resource

--- A position on the screen. It is either a `Vec2` in screen coordinates or a `ScreenPosition` from the `coord` module.
type Pos = Coord.ScreenPosition | Vec.Vec2
--- A size or direction on the screen. It is either a `Vec2` in screen units or a `ScreenVec` from the `coord` module.
type Direction = Coord.ScreenVec | Vec.Vec2

--- Draws the image in a rectangle of the screen.
---
--- `pos` is a corner of the rectangle and `size` goes from that corner to the opposite one. With a positive size,
--- `pos` is the bottom left corner. A negative size flips the image. Nothing is drawn if the image is not loaded yet.
---
--- @param pos The corner of the rectangle. With a positive size, this is the bottom left corner.
--- @param size The width and height of the rectangle on the screen.
--- @param color A tint. The color of each pixel of the image is multiplied by it. Defaults to white, which leaves the
--- image unchanged. Use the alpha component to make the image transparent.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- const ballImage = loader.loadImage("textures/ball.png")
---
--- function Update(deltaTime: number)
--- 	ballImage:draw(vec.V2(-0.1, -0.1), vec.V2(0.2, 0.2))
--- 	-- The same image, half transparent.
--- 	ballImage:draw(vec.V2(0.3, -0.1), vec.V2(0.2, 0.2), vec4.createColor(1, 1, 1, 0.5))
--- end
--- ```
---
--- # See also
---
--- - `graphics.drawRect`: draws a filled rectangle.
--- - `ImageResource:drawPart`: draws only a part of the image.
function ImageResourceImpl.draw(self: ImageResource, pos: Pos, size: Direction, color: Vec4.Vec4?): ()
	error("Implemented in native code")
end

--- Draws a rectangular part of the image on a quadrilateral of the screen.
---
--- The part is chosen with `src_pos` and `src_size`, as ratios of the image: `(0, 0)` is the top left corner of the
--- image (its first row of pixels is at the top) and `(1, 1)` is the bottom right one. The part is stretched to fit the four destination points. They must be
--- given in this order: bottom left, bottom right, top right, top left. Swap them to flip or rotate the image. Nothing
--- is drawn if the image is not loaded yet.
---
--- For tiles of a sprite sheet, `ImageWithTileset:drawTile` computes `src_pos` and `src_size` for you.
---
--- @param dest_p1 The screen position of the bottom left corner of the part.
--- @param dest_p2 The screen position of the bottom right corner of the part.
--- @param dest_p3 The screen position of the top right corner of the part.
--- @param dest_p4 The screen position of the top left corner of the part.
--- @param src_pos The top left corner of the part, as a ratio of the image size. Use `V2(0, 0)` to start at the corner.
--- @param src_size The size of the part, as a ratio of the image size. Use `V2(1, 1)` with `V2(0, 0)` to draw the whole
--- image.
--- @param color A tint, multiplied with the color of each pixel. Defaults to white, which leaves the image unchanged.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
--- const vec = require("@vectarine/vec")
---
--- const sheet = loader.loadImage("textures/sheet.png")
---
--- function Update(deltaTime: number)
--- 	-- Draw the left half of the image in a 0.4 x 0.4 square.
--- 	sheet:drawPart(
--- 		vec.V2(-0.2, -0.2), vec.V2(0.2, -0.2), vec.V2(0.2, 0.2), vec.V2(-0.2, 0.2),
--- 		vec.V2(0, 0), vec.V2(0.5, 1)
--- 	)
--- end
--- ```
function ImageResourceImpl.drawPart(
	self: ImageResource,
	dest_p1: Pos,
	dest_p2: Pos,
	dest_p3: Pos,
	dest_p4: Pos,
	src_pos: Vec.Vec2,
	src_size: Vec.Vec2,
	color: Vec4.Vec4?
): ()
	error("Implemented in native code")
end

--- Returns the size of the image in pixels.
---
--- @return The width and height of the image, in pixels.
---
--- # Errors
---
--- Raises an error if the image is not loaded yet. Check `Resource:isReady` first.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- const ballImage = loader.loadImage("textures/ball.png")
---
--- function Update(deltaTime: number)
--- 	if ballImage:isReady() then
--- 		print(ballImage:getSize().x) -- for example 64
--- 	end
--- end
--- ```
function ImageResourceImpl.getSize(self: ImageResource): Vec.Vec2
	error("Implemented in native code")
end

local ImageWithTileset = { imageWithTileset = true }
ImageWithTileset.__index = ImageWithTileset
--- An image paired with a tileset, used to draw tiles by id or by name.
---
--- You get one from `ImageResource:withTileset`. Draw tiles with `ImageWithTileset:drawTile` or
--- `ImageWithTileset:drawTileQuad`. It is a lightweight value: create it once and keep it.
export type ImageWithTileset = typeof(setmetatable({}, ImageWithTileset))

--- Combines an image with a tileset to draw tiles from the image.
---
--- The drawing methods of the result need both the image and the tileset to be loaded, and draw nothing until they are.
--- They are more convenient than computing `src_pos` and `src_size` for `ImageResource:drawPart`. This call itself
--- does not check that the resources are loaded.
---
--- @param tileset The tileset that describes how the image is split into tiles.
--- @return An `ImageWithTileset` that can draw the tiles of the image.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- const tilesImage = loader.loadImage("textures/tiles.png", false)
--- const tilesData = loader.loadTileset("textures/tiles.tsx")
--- const tiles = tilesImage:withTileset(tilesData)
--- ```
function ImageResourceImpl:withTileset(tileset: Tile.TilesetResource): ImageWithTileset
	error("Implemented in native code")
end

--- Draws one tile of the image in a rectangle of the screen.
---
--- The tile is found from its id or from its type name given in the tileset file. The part of the image is computed
--- from the tile size, the spacing and the margin of the tileset. Nothing is drawn if the image or the tileset is not
--- loaded yet. Drawing many tiles from one image is efficient, because they share the same image.
---
--- If several tiles have the same type name, which one is drawn is not specified. If the name does not exist in the
--- tileset, or the value is neither a number nor a string, an error message is printed to the console and nothing is
--- drawn. A numeric id is not checked against the tileset: it is drawn from where that id would be in the image.
---
--- @param tileIdOrType The id of the tile (a number, counting from `0` along each row), or its type name (a string).
--- @param pos The corner of the rectangle. With a positive size, this is the bottom left corner.
--- @param size The width and height of the rectangle on the screen. A negative size flips the tile.
--- @param color A tint, multiplied with the color of each pixel. Defaults to white, which leaves the tile unchanged.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
--- const vec = require("@vectarine/vec")
---
--- const tiles = loader.loadImage("textures/tiles.png", false):withTileset(loader.loadTileset("textures/tiles.tsx"))
---
--- function Update(deltaTime: number)
--- 	tiles:drawTile(0, vec.V2(-0.5, 0), vec.V2(0.2, 0.2))
--- 	tiles:drawTile("bee_monster", vec.V2(0.3, 0), vec.V2(0.2, 0.2))
--- end
--- ```
---
--- # See also
---
--- - `ImageWithTileset:drawTileQuad`: draws a tile on four points.
--- - `TilemapResource:getTile`: reads the tile ids of a level.
function ImageWithTileset:drawTile(tileIdOrType: number | string, pos: Pos, size: Direction, color: Vec4.Vec4?): ()
	error("Implemented in native code")
end

--- Draws one tile of the image on a quadrilateral of the screen.
---
--- Like `ImageWithTileset:drawTile`, but the tile is stretched to fit four points. Use it to rotate or flip a tile.
--- The points must be given in this order: bottom left, bottom right, top right, top left. Nothing is drawn if the
--- image or the tileset is not loaded yet.
---
--- @param tileIdOrType The id of the tile (a number), or its type name (a string). If several tiles have the same
--- type name, which one is drawn is not specified.
--- @param dest_p1 The screen position of the bottom left corner of the tile.
--- @param dest_p2 The screen position of the bottom right corner of the tile.
--- @param dest_p3 The screen position of the top right corner of the tile.
--- @param dest_p4 The screen position of the top left corner of the tile.
--- @param color A tint, multiplied with the color of each pixel. Defaults to white, which leaves the tile unchanged.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
--- const vec = require("@vectarine/vec")
---
--- const tiles = loader.loadImage("textures/tiles.png", false):withTileset(loader.loadTileset("textures/tiles.tsx"))
---
--- function Update(deltaTime: number)
--- 	-- The tile drawn upside down: the bottom points are given at the top.
--- 	tiles:drawTileQuad(
--- 		0,
--- 		vec.V2(-0.1, 0.1), vec.V2(0.1, 0.1), vec.V2(0.1, -0.1), vec.V2(-0.1, -0.1)
--- 	)
--- end
--- ```
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
