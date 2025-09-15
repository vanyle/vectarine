---@meta

--- @alias Color {r: number, g: number, b: number, a: number}

--- Clear the canvas
--- @param color Color
function clear(color) end

--- Draws a filled rectangle
--- @param x number X
--- @param y number Y
--- @param w number Width
--- @param h number Height
--- @param color Color
function drawRect(x, y, w, h, color) end

--- Draws a filled circle
--- @param x number X
--- @param y number Y
--- @param radius number Radius
--- @param color Color
function drawCircle(x, y, radius, color) end

--- Draws an image
--- @param image ImageResource
--- @param x number
--- @param y number
--- @param w number
--- @param h number
function drawImage(image, x, y, w, h) end

--- Draws text at (x,y) with given font, size and color
--- size is the maximum height that a text of that font will take on the screen
--- This is sometimes called the line height.
--- @param text string
--- @param font FontResource
--- @param x number
--- @param y number
--- @param size number
--- @param color Color
function drawText(text, font, x, y, size, color) end

--- Measures how much space the text will take when drawn
--- height will always be less than size.
--- bearingY will always be less than height. This is the distance from the top of the text to the baseline.
--- @param text string
--- @param font FontResource
--- @param size number
--- @return {width: number, height: number, bearingY: number}
--- @nodiscard
function measureText(text, font, size) end
