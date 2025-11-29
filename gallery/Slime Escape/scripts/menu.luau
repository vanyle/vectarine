local Coord = require("@vectarine/coord")
local Graphics = require("@vectarine/graphics")
local Loader = require("@vectarine/loader")
local Screen = require("@vectarine/screen")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")

local module = {} -- This is where all our exports will go

local fontResource = Loader.loadFont("fonts/arial.ttf")

module.screen = Screen.newScreen("Menu", function()
	Graphics.drawRect(Coord.gl(Vec.V2(-1, -1)), Coord.glVec(Vec.V2(2, 2)), Vec4.createColor(0.2, 0, 0, 1))
	fontResource:drawText("Main Menu", Coord.gl(Vec.V2(-0.5, 0)), 0.32, Vec4.createColor(1, 1, 1, 1))
	fontResource:drawText(
		"Press SPACE to start",
		Coord.gl(Vec.V2(-0.5, -0.3)),
		0.16,
		Vec4.createColor(0.7, 0.7, 0.7, 1)
	)
end)

Screen.setCurrentScreen(module.screen)

return module -- return for the module for typing
