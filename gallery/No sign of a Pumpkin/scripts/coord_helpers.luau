local Coord = require("@vectarine/coord")
local Io = require("@vectarine/io")
local Vec = require("@vectarine/vec")

local V2 = Vec.V2
Io.setResizeable(true)

function convertToScreenPos(GameState, worldPos: Vec.Vec2, correctForTimerOffset: boolean): Coord.ScreenPosition
	local scaled: Vec.Vec2 = (worldPos - GameState.camera.position):scale(GameState.camera.zoom)
	local sp = Coord.px(scaled, Io.getWindowSize()) + Coord.glVec(V2(1, -1))
	return sp
end

function convertToWorldPos(GameState, screenPos: Vec.Vec2, correctForTimerOffset: boolean): Vec.Vec2
	local sp = Coord.gl(screenPos)
	local scaled = (sp - Coord.glVec(V2(1, -1))):px(Io.getWindowSize())
	return scaled:scale(1 / GameState.camera.zoom) + GameState.camera.position
end

function convertToScreenSize(GameState, worldSize: Vec.Vec2): Coord.ScreenVec
	local scaled: Vec.Vec2 = worldSize:scale(GameState.camera.zoom)
	return Coord.pxVec(scaled, Io.getWindowSize())
end

-- Export the module functions
return {
	convertToScreenPos = convertToScreenPos,
	convertToWorldPos = convertToWorldPos,
	convertToScreenSize = convertToScreenSize,
}
