local Coord = require("@vectarine/coord")
local Graphics = require("@vectarine/graphics")
local Loader = require("@vectarine/loader")
local Screen = require("@vectarine/screen")
local Vec = require("@vectarine/vec")

local module = {} -- This is where all our exports will go

local fontResource = Loader.loadFont("fonts/arial.ttf")

module.screen = Screen.newScreen("Menu", function()
	Graphics.drawRect(Coord.gl(Vec.V2(-1, -1)), Coord.glVec(Vec.V2(2, 2)), { r = 0.2, g = 0, b = 0, a = 1 })
	fontResource:drawText("Main Menu", Coord.gl(Vec.V2(-0.5, 0)), 0.32, { r = 1, g = 1, b = 1, a = 1 })
	fontResource:drawText(
		"Press SPACE to start",
		Coord.gl(Vec.V2(-0.5, -0.3)),
		0.16,
		{ r = 0.7, g = 0.7, b = 0.7, a = 1 }
	)
end)

Screen.setCurrentScreen(module.screen)

return module -- return for the module for typing
