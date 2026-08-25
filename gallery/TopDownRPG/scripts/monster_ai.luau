--[[
Monster behaviors. Each behavior decides where a monster wants to go this turn.

A behavior returns a list of directions to try, in order. rules.luau moves the monster in the first
direction that is possible (potions can change what "moving in a direction" means, see rules.moveFor).

To create a new behavior, add a function to `behaviors` and reference it by name in data/monsters.luau.
]]
const loader = require("@vectarine/loader")
const monsters = require("./data/monsters")
const types = require("./types")

const module = loader.init() or {}

type Behavior = (game: types.Game, monster: types.Entity) -> { types.Direction }

module.behaviors = {
	--- Step towards the player, along the axis where the player is the furthest (horizontal on ties).
	--- If that way is blocked, try the other axis.
	chaser = function(game: types.Game, monster: types.Entity): { types.Direction }
		local delta = game.player.pos - monster.pos
		local horizontal: types.Direction? = if delta.x > 0 then "right" elseif delta.x < 0 then "left" else nil
		local vertical: types.Direction? = if delta.y > 0 then "down" elseif delta.y < 0 then "up" else nil
		local result: { types.Direction } = {}
		local first, second = horizontal, vertical
		if math.abs(delta.y) > math.abs(delta.x) then
			first, second = vertical, horizontal
		end
		if first then
			table.insert(result, first)
		end
		if second then
			table.insert(result, second)
		end
		return result
	end,

	--- Keep going in the same direction. rules.luau turns the monster around when it is blocked.
	patrol = function(_game: types.Game, monster: types.Entity): { types.Direction }
		return { monster.dir or "right" }
	end,

	--- Do the opposite of the player's move. Stand still when the player does not move.
	mirror = function(game: types.Game, _monster: types.Entity): { types.Direction }
		local intent = game.player.lastIntent
		local result: { types.Direction } = {}
		if intent then
			table.insert(result, types.opposite[intent])
		end
		return result
	end,
} :: { [monsters.Behavior]: Behavior }

function module.intents(game: types.Game, monster: types.Entity): { types.Direction }
	local definition = monsters.monsters[monster.monster or ""]
	return module.behaviors[definition.behavior](game, monster)
end

return module
