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
