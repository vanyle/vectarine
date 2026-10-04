--[[
Things that open: gates, valves, the council door, and the sun eyes that open gates.

Each one has an id (`data.id` in the level, or `data.opens` for a sun eye: the id of the gate it opens).
Opening an id sets the flag "open:<id>" and turns every prop of the level with that id into its `openKind`
(see data/props.luau). level.luau reads the flag to build the level already open the next time.
]]
const effects = require("./effects")
const loader = require("@vectarine/loader")
const props = require("./data/props")
const types = require("./types")
const vec = require("@vectarine/vec")

const module = loader.init() or {}

function module.isOpen(game: types.Game, id: string): boolean
	return game.flags[`open:{id}`] == true
end

--- Open everything with the id `id` in the level. Returns false if it was already open.
function module.open(game: types.Game, id: string): boolean
	if module.isOpen(game, id) then
		return false
	end
	game.flags[`open:{id}`] = true
	for _, prop in game.level.props do
		local kind = props.kinds[prop.kind]
		if kind.openKind and (prop.data.id == id or prop.data.opens == id) then
			prop.kind = kind.openKind
			effects.burst(prop.cell + vec.V2(0.5, 0.5), "sparkle", 14)
		end
	end
	effects.shake(0.25)
	return true
end

return module
