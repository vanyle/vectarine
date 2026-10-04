--[[
The notebook of the explorer: the bestiary, the artefacts found, the texts read and the map of the explored places.

The explored cells of each level are saved as rows of "0" and "1" (`game.notebook.explored[levelId]`).
While playing, they are kept in a faster table (`explored`) and written back to the rows with `flush`.
The map is only shown with the Cartographer's lens, but places are recorded from the start.
]]
const loader = require("@vectarine/loader")
const types = require("./types")
const vec = require("@vectarine/vec")

const module = loader.init() or {}

--- Explored cells of the current level, by cell index (1 + x + y * width)
module.explored = module.explored or {} :: { [number]: boolean }
module.exploredLevel = module.exploredLevel or "" :: string

--- Distance at which the explorer notices the shape of the place
const SIGHT = 6

--- Load the explored cells of the current level from the saved rows
function module.load(game: types.Game)
	module.explored = {}
	module.exploredLevel = game.level.id
	local rows = game.notebook.explored[game.level.id]
	if not rows then
		return
	end
	for y, row in rows do
		for x = 1, #row do
			if string.sub(row, x, x) == "1" then
				module.explored[x + (y - 1) * game.level.width] = true
			end
		end
	end
end

--- Write the explored cells of the current level back to the saved rows
function module.flush(game: types.Game)
	if module.exploredLevel ~= game.level.id then
		return
	end
	local rows = {}
	local width = game.level.width
	for y = 0, game.level.height - 1 do
		local chars = table.create(width, "0")
		for x = 0, width - 1 do
			if module.explored[1 + x + y * width] then
				chars[x + 1] = "1"
			end
		end
		table.insert(rows, table.concat(chars))
	end
	game.notebook.explored[game.level.id] = rows
end

--- Record the cells around the player
function module.explore(game: types.Game)
	if module.exploredLevel ~= game.level.id then
		module.load(game)
	end
	local center = game.player.pos
	local width = game.level.width
	for y = math.max(0, math.floor(center.y - SIGHT)), math.min(game.level.height - 1, math.floor(center.y + SIGHT)) do
		for x = math.max(0, math.floor(center.x - SIGHT)), math.min(width - 1, math.floor(center.x + SIGHT)) do
			if (vec.V2(x + 0.5, y + 0.5) - center):length() <= SIGHT then
				module.explored[1 + x + y * width] = true
			end
		end
	end
end

function module.isExplored(game: types.Game, x: number, y: number): boolean
	return module.explored[1 + x + y * game.level.width] == true
end

return module
