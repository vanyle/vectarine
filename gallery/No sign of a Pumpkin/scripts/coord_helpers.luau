const coord = require("@vectarine/coord")
const io = require("@vectarine/io")
const vec = require("@vectarine/vec")

const V2 = vec.V2
io.setResizeable(true)

function convertToScreenPos(GameState, worldPos: vec.Vec2, correctForTimerOffset: boolean): coord.ScreenPosition
	local scaled: vec.Vec2 = (worldPos - GameState.camera.position):scale(GameState.camera.zoom)
	local sp = coord.px(scaled, io.getWindowSize()) + coord.glVec(V2(1, -1))
	return sp
end

function convertToWorldPos(GameState, screenPos: vec.Vec2, correctForTimerOffset: boolean): vec.Vec2
	local sp = coord.gl(screenPos)
	local scaled = (sp - coord.glVec(V2(1, -1))):px(io.getWindowSize())
	return scaled:scale(1 / GameState.camera.zoom) + GameState.camera.position
end

function convertToScreenSize(GameState, worldSize: vec.Vec2): coord.ScreenVec
	local scaled: vec.Vec2 = worldSize:scale(GameState.camera.zoom)
	return coord.pxVec(scaled, io.getWindowSize())
end

-- Export the module functions
return {
	convertToScreenPos = convertToScreenPos,
	convertToWorldPos = convertToWorldPos,
	convertToScreenSize = convertToScreenSize,
}
