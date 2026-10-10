--[[
The animated background of the menus: the camera slowly pans over the sinkhole, where the story starts.

It uses its own game, drawn with the same code as the real one (world_view.luau). Nothing happens in it.
]]
const game_state = require("./game_state")
const graphics = require("@vectarine/graphics")
const io = require("@vectarine/io")
const loader = require("@vectarine/loader")
const types = require("./types")
const ui_components = require("./ui_components")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")
const world_view = require("./world_view")

const module = loader.init() or {}

--- A game used only for decoration
module.game = module.game :: types.Game?

--- Speed of the camera, in radians per second (the camera goes back and forth)
const PAN_SPEED = 0.04

function module.draw()
	if not module.game then
		local game = game_state.new()
		-- Nobody is there: put the player far away from the view
		game.player.pos = vec.V2(-10, -10)
		module.game = game
	end
	local game = module.game :: types.Game

	local window = io.getWindowSize()
	local tilePx = 16 * math.max(1, math.floor(window.y / 200))
	local viewport: world_view.Viewport = {
		pos = vec.V2(-1, -1),
		size = vec.V2(2, 2),
		tileSize = vec.V2(tilePx * 2 / window.x, tilePx * 2 / window.y),
	}
	local visible = world_view.visibleTiles(viewport)
	local maxX = math.max(0, game.level.width - visible.x)
	local x = maxX * (0.5 - 0.5 * math.cos(ui_components.time * PAN_SPEED * math.pi))
	world_view.camera = vec.V2(x, math.max(0, (game.level.height - visible.y) / 2))
	world_view.drawWorld(game, viewport, { time = ui_components.time, darkness = false })
	-- Darken the world so that the menu stands out
	graphics.drawRect(vec.V2(-1, -1), vec.V2(2, 2), vec4.createColor(0.04, 0.06, 0.09, 0.62))
end

return module
