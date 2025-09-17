---@meta

--- @alias Color {r: number, g: number, b: number, a: number}

--- Clear the canvas
--- @param color Color
function clear(color) end

--- Draws a filled rectangle
--- @param pos Vec2
--- @param size Vec2
--- @param color Color
function drawRect(pos, size, color) end

--- Draws a filled circle
--- @param center Vec2
--- @param radius number Radius
--- @param color Color
function drawCircle(center, radius, color) end

--- Draws an image
--- @param image ImageResource
--- @param pos Vec2
--- @param size Vec2
function drawImage(image, pos, size) end

--- Draws text at (x,y) with given font, size and color
--- size is the maximum height that a text of that font will take on the screen
--- This is sometimes called the line height.
--- @param text string
--- @param font FontResource
--- @param pos Vec2
--- @param size number
--- @param color Color
function drawText(text, font, pos, size, color) end

--- Measures how much space the text will take when drawn
--- height will always be less than size.
--- bearingY will always be less than height. This is the distance from the top of the text to the baseline.
--- @param text string
--- @param font FontResource
--- @param size number
--- @return {width: number, height: number, bearingY: number}
--- @nodiscard
function measureText(text, font, size) end
