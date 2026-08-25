--[[
The animated background of the menus: the camera slowly pans over the world of the game.

It uses its own copy of the world, drawn with the same code as the game (world_view.luau).
]]
const game_state = require("./game_state")
const graphics = require("@vectarine/graphics")
const io = require("@vectarine/io")
const loader = require("@vectarine/loader")
const rooms = require("./data/rooms")
const types = require("./types")
const ui_components = require("./ui_components")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")
const world_view = require("./world_view")

const module = loader.init() or {}

--- A world used only for decoration. Nothing happens in it.
module.game = module.game :: types.Game?

--- Which row of rooms to show (1 is the top row)
const ROOM_ROW = 1
--- Speed of the camera, in radians per second (the camera goes back and forth)
const PAN_SPEED = 0.05

function module.draw()
	if not module.game then
		module.game = game_state.new()
	end
	local game = module.game :: types.Game

	local window = io.getWindowSize()
	local tilePx = window.y / rooms.ROOM_HEIGHT
	local tileSize = vec.V2(tilePx * 2 / window.x, tilePx * 2 / window.y)
	local visibleWidth = 2 / tileSize.x
	local maxX = #rooms.layout[ROOM_ROW] * rooms.ROOM_WIDTH - visibleWidth
	local x = maxX * (0.5 - 0.5 * math.cos(ui_components.time * PAN_SPEED * math.pi))
	local camera = vec.V2(x, (ROOM_ROW - 1) * rooms.ROOM_HEIGHT)

	world_view.drawWorld(
		game,
		{ pos = vec.V2(-1, -1), size = vec.V2(2, 2), tileSize = tileSize },
		camera,
		{ time = ui_components.time, showPlayer = false }
	)
	-- Darken the world so that the menu stands out
	graphics.drawRect(vec.V2(-1, -1), vec.V2(2, 2), vec4.createColor(0.03, 0.02, 0.06, 0.62))
end

return module
