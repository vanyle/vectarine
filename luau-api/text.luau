local Coord = require("@vectarine/coord")
local Resource = require("@vectarine/resource")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")

--[[
Draws and measures text with a font.

Use `text.font` to draw with the built-in font (Roboto), or load your own `.ttf` file with `loader.loadFont` to get a
`FontResource`. Then call `FontResource:drawText` every frame to show text, and `FontResource:measureText` to know how
much room it takes, for example to center it or to put a box around it. For text inside menus, with wrapping and
alignment, see `ui.text`.

Sizes are in screen coordinates, where the whole height of the screen is 2. A size of 0.1 is a line that takes 5% of the
height of the screen. See `coord` for the coordinate systems.

# Example

```luau
const text = require("@vectarine/text")
const coord = require("@vectarine/coord")
const vec4 = require("@vectarine/vec4")

function Update(deltaTime: number)
	text.font:drawText("Score: 120", coord.TOP_LEFT + coord.glVec(0.05, -0.15), 0.1, vec4.WHITE)
end
```

# Common mistakes

- `FontResource:drawText` draws nothing, without any error, while a font from `loader.loadFont` is not loaded yet.
- The position is the start of the baseline, not the bottom left of the text: letters like `"g"` go below it.
- The size is the height of a line in screen coordinates, not in pixels. Use `coord.pxVec` to write it in pixels.
]]
local module = {}

local FontResourceImpl = { type = "font" }
FontResourceImpl.__index = FontResourceImpl

--- A font that can draw text, either the built-in font `text.font` or a file loaded with `loader.loadFont`.
---
--- It is a resource: you can check `Resource:isReady` before using it. Call `FontResource:drawText` to draw with it and
--- `FontResource:measureText` to measure text.
export type FontResource = typeof(setmetatable({}, FontResourceImpl)) & Resource.Resource

--- A position given as a `ScreenPosition`, or as a `Vec2` in screen coordinates.
type Pos = Coord.ScreenPosition | Vec.Vec2
--- A size given as a number in screen coordinates, or as a `ScreenVec` whose y component is used.
type Size = number | Coord.ScreenVec

--- Draws a line of text at a position, with a size and a color.
---
--- `pos` is where the text starts: the left end of its baseline, which is the line the letters sit on. Letters like
--- `"g"` or `"y"` go below it. `size` is the height of a line of text, sometimes called the line height: no letter is
--- taller than this. A number is in screen coordinates, where 2 is the height of the screen. With a `ScreenVec`, only the y
--- component is used. The width of the text follows from the height and keeps the letters undistorted on any window
--- shape.
---
--- The text is drawn on one line. Line breaks are not handled here. Use `ui.text` for wrapping.
---
--- The first time a character that is not a letter, digit or common symbol of the English keyboard is drawn (an
--- accented letter for example), the font prepares it, which takes a moment. If the font is not loaded yet, nothing is
--- drawn.
---
--- @param text The text to draw.
--- @param pos Where to start the text, as a `ScreenPosition` or a `Vec2` in screen coordinates.
--- @param size The height of a line, in screen coordinates, or a `ScreenVec` whose y component is used.
--- @param color The color of the text. Defaults to black.
---
--- # Example
---
--- ```luau
--- const text = require("@vectarine/text")
--- const coord = require("@vectarine/coord")
--- const vec4 = require("@vectarine/vec4")
---
--- function Update(deltaTime: number)
--- 	-- Red text in the middle of the screen
--- 	text.font:drawText("Game over", coord.CENTER, 0.2, vec4.RED)
--- 	-- Written in pixels: 24 pixels high, starting 20 pixels from the left, with its baseline 40 pixels from the top
--- 	text.font:drawText("Menu", coord.px(20, 40), coord.pxVec(0, -24))
--- end
--- ```
---
--- # See also
---
--- - `FontResource:measureText`: measure text before drawing it.
--- - `ui.text`: text with wrapping and alignment inside a UI.
function FontResourceImpl.drawText(self: FontResource, text: string, pos: Pos, size: Size, color: Vec4.Vec4?): ()
	error("Implemented in native code")
end

--- Measures how much space a line of text takes when drawn.
---
--- All values are in screen coordinates, where 2 is the height of the screen, and use the same `size` as
--- `FontResource:drawText`. Use it to center text or to draw a background behind it.
---
--- The returned table has three fields:
---
--- - `width`: the horizontal space of the text.
--- - `height`: the height of the tallest letter, which is at most `size`.
--- - `bearingY`: the distance from the baseline to the top of the tallest letter, which is at most `height`.
---
--- If the font is not loaded yet, all three values are 0.
---
--- Characters that the font has not prepared yet count as 0 wide. The English letters, digits and common symbols are
--- always ready, but an accented letter only counts once it has been drawn with `FontResource:drawText`.
---
--- @param text The text to measure.
--- @param size The height of a line, in screen coordinates, or a `ScreenVec` whose y component is used.
--- @return A table with the `width`, `height` and `bearingY` of the text.
---
--- # Example
---
--- ```luau
--- const text = require("@vectarine/text")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- function Update(deltaTime: number)
--- 	local size = text.font:measureText("Hello", 0.2)
--- 	-- Draw the text so that its middle is at the center of the screen
--- 	text.font:drawText("Hello", vec.V2(-size.width / 2, 0), 0.2, vec4.WHITE)
--- end
--- ```
---
--- # See also
---
--- - `FontResource:drawText`: draw the text.
function FontResourceImpl.measureText(
	self: FontResource,
	text: string,
	size: Size
): { width: number, height: number, bearingY: number }
	error("Implemented in native code")
end

local function getDefaultFontPrivate(): FontResource
	error("Implemented in native code")
end

--- The default font (Roboto), which you can use to draw without loading a `.ttf` file.
---
--- It is not a real resource: it is always loaded and is available from the first frame.
module.font = getDefaultFontPrivate()

return module
