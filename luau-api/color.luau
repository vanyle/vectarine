--- Helper functions to create and manipulate colors
--- Colors are Vec4 where all components are between 0 and 1.
local module = {}

export type Vec4 = { r: number, g: number, b: number, a: number }

--- Alias for V4, but indicate that the Vec4 will be used as a color.
function module.createColor(r: number, g: number, b: number, a: number): Vec4
	error("Not implemented")
end

module.BLACK = module.createColor(0, 0, 0, 1)
module.WHITE = module.createColor(1, 1, 1, 1)
module.TRANSPARENT = module.createColor(0, 0, 0, 0)

module.RED = module.createColor(1, 0, 0, 1)
module.GREEN = module.createColor(0, 1, 0, 1)
module.BLUE = module.createColor(0, 0, 1, 1)

module.YELLOW = module.createColor(1, 1, 0, 1)
module.MAGENTA = module.createColor(1, 0, 1, 1)
module.CYAN = module.createColor(0, 1, 1, 1)

module.AZURE = module.createColor(0, 0.5, 1, 1)
module.ORANGE = module.createColor(1, 0.5, 0, 1)
module.PURPLE = module.createColor(0.5, 0, 1, 1)
module.SPRING = module.createColor(0, 1, 0.5, 1)
module.LIME = module.createColor(0.5, 1, 0, 1)
module.PINK = module.createColor(1, 0, 0.5, 1)

module.LIGHT_GRAY = module.createColor(0.7, 0.7, 0.7, 1)
module.DARK_GRAY = module.createColor(0.3, 0.3, 0.3, 1)

module.GRAY = module.createColor(0.5, 0.5, 0.5, 1)
module.DARK_RED = module.createColor(0.5, 0, 0, 1)
module.DARK_GREEN = module.createColor(0, 0.5, 0, 1)
module.DARK_BLUE = module.createColor(0, 0, 0.5, 1)

return module
