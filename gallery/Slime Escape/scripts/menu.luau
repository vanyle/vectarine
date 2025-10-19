local Coord = require("@vectarine/coord")
local Graphics = require("@vectarine/graphics")
local Resources = require("@vectarine/resources")
local Screen = require("@vectarine/screen")

local module = {} -- This is where all our exports will go

local fontResource = Resources.loadFont("fonts/arial.ttf")

module.screen = Screen.newScreen("Menu", function()
	Graphics.drawRect(Coord.gl(-1, -1), Coord.glDelta(2, 2), { r = 0.2, g = 0, b = 0, a = 1 })
	Graphics.drawText("Main Menu", fontResource, Coord.gl(-0.5, 0), 0.32, { r = 1, g = 1, b = 1, a = 1 })
	Graphics.drawText(
		"Press SPACE to start",
		fontResource,
		Coord.gl(-0.5, -0.3),
		0.16,
		{ r = 0.7, g = 0.7, b = 0.7, a = 1 }
	)
end)

Screen.setCurrentScreen(module.screen)

return module -- return for the module for typing
