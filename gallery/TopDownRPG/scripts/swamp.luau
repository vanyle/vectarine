--[[
The dangers of the marsh, for the player (see data/terrain.luau and data/props.luau):
- bogs swallow whoever stands still in them: the player climbs back to the last firm ground they stood on, hurt.
  Moving keeps them afloat.
- pale lily pads sink a moment after the player steps on them, and come back up later. Whoever stands on a pad that
  sinks falls in the water, and climbs back to the last firm ground.
- toxic water poisons and burns whoever sails on it, unless they carry the bezoar.
- miasma blocks the way (see level.isFree): next to it, without the beaked mask, a hint says why (hints.luau).
]]
const effects = require("./effects")
const level = require("./level")
const loader = require("@vectarine/loader")
const player_rules = require("./player")
const props = require("./data/props")
const types = require("./types")
const vec = require("@vectarine/vec")

const module = loader.init() or {}

--- Seconds standing still in a bog before it swallows the player
module.SINK_TIME = 1.4
--- Damage when a bog swallows the player (a real threat: bogs are obstacles, not puddles), and when they fall
--- from a sunk lily pad (a lesser one: the pads are a puzzle of timing)
const BOG_DAMAGE = 8
const PAD_DAMAGE = 3
--- Seconds on a pale lily pad before it sinks, and seconds before it comes back up
const PAD_TIME = 0.9
const PAD_BACK = 4

--- State of the marsh that is not saved: how long the player has been sinking, where they last stood on firm
--- ground, and the pads that are sinking or sunk (by prop key: the time they were stepped on, or came back up)
module.state = {
	sink = 0,
	safePos = nil :: vec.Vec2?,
	safeLevel = "",
	padStepped = {} :: { [string]: number },
	padSunk = {} :: { [string]: { prop: types.Prop, upTime: number } },
	--- The level these pads belong to: everything is reset in another level
	levelId = "",
}

--- How deep the player is in a bog, from 0 to 1 (to draw them sinking)
function module.sinking(): number
	return math.clamp(module.state.sink / module.SINK_TIME, 0, 1)
end

--- Is the player on water (sailing on the ferry)?
function module.isSailing(game: types.Game): boolean
	local cell = level.cellOf(game.player.pos)
	local lvl = game.level :: level.Level
	local ground = level.groundAt(lvl, cell.x, cell.y)
	local prop = level.propAt(lvl, cell.x, cell.y)
	local onPad = prop ~= nil and props.kinds[prop.kind].floor == true
	return ground ~= nil and ground.sailable == true and not onPad
end

local function message(game: types.Game, text: string)
	game.events[#game.events + 1] = { kind = "message", text = text }
end

--- The player falls (swallowed by a bog, or from a sunk pad): back to the last firm ground, hurt
local function fall(game: types.Game, damage: number, text: string)
	local player = game.player
	local state = module.state
	effects.burst(player.pos, "water", 12)
	player_rules.hurt(game, damage, "blunt", player.pos)
	player.knockback = vec.V2(0, 0)
	-- Back on firm ground: where the player last stood (or the closest firm ground), never in a bog
	local lvl = game.level :: level.Level
	local wanted = if state.safeLevel == lvl.id then state.safePos else nil
	local landing = level.landingSpot(lvl, wanted, player.pos, player_rules.RX, player_rules.RY, { height = player.height }, function(x: number, y: number): boolean
		local ground = level.groundAt(lvl, x, y)
		local prop = level.propAt(lvl, x, y)
		return ground ~= nil and not ground.bog and not (prop ~= nil and props.kinds[prop.kind].sinks)
	end)
	if landing then
		player.pos = landing
	end
	state.sink = 0
	message(game, text)
end

local function updatePads(game: types.Game, cell: vec.Vec2, dt: number)
	local state = module.state
	local lvl = game.level :: level.Level
	-- Sunk pads come back up when their time is over (and nobody is in the way)
	for key, sunk in state.padSunk do
		if game.time > sunk.upTime then
			sunk.prop.kind = "lily_sink"
			state.padSunk[key] = nil
		end
	end
	local prop = level.propAt(lvl, cell.x, cell.y)
	if prop and props.kinds[prop.kind].sinks then
		local stepped = state.padStepped[prop.key]
		if not stepped then
			state.padStepped[prop.key] = game.time
		elseif game.time - stepped > PAD_TIME then
			prop.kind = "lily_sunk"
			state.padStepped[prop.key] = nil
			state.padSunk[prop.key] = { prop = prop, upTime = game.time + PAD_BACK }
			effects.burst(prop.cell + vec.V2(0.5, 0.5), "water", 10)
		end
	end
	-- Pads the player left stop sinking
	for key in state.padStepped do
		if not prop or prop.key ~= key then
			state.padStepped[key] = nil
		end
	end
end

--- Called every frame, after the player moved. `moving` tells if the player is trying to move.
function module.update(game: types.Game, moving: boolean, dt: number)
	local player = game.player
	local state = module.state
	local lvl = game.level :: level.Level
	if state.levelId ~= lvl.id then
		state.levelId = lvl.id
		state.padStepped = {}
		state.padSunk = {}
		state.sink = 0
	end
	local cell = level.cellOf(player.pos)
	local ground = level.groundAt(lvl, cell.x, cell.y)
	updatePads(game, cell, dt)

	-- A pad sank under the player: without the ferry, they fall in the water
	local under = level.propAt(lvl, cell.x, cell.y)
	if under and under.kind == "lily_sunk" and ground and not ground.walkable and not player.artefacts.ferry_skiff then
		fall(game, PAD_DAMAGE, "The lily pad sinks under your feet! You climb back, soaked.")
		return
	end

	if ground and ground.bog then
		if moving then
			state.sink = math.max(0, state.sink - dt * 2)
		else
			state.sink += dt
		end
		if state.sink > module.SINK_TIME then
			fall(game, BOG_DAMAGE, "The bog swallows your legs! You pull yourself out. Keep moving in the mud.")
		end
		return
	end
	state.sink = 0

	if ground and ground.toxic then
		-- The green water burns through the hull: only the bezoar protects from it
		if not player.artefacts.bezoar then
			player_rules.poison(player, 2.5)
			player.hp -= 3 * dt
			local last = game.flags.toxicHintTime
			if type(last) ~= "number" or game.time - last > 15 then
				game.flags.toxicHintTime = game.time
				message(game, "The green water burns through the hull and into your skin! Turn back!")
			end
		end
		return
	end

	-- The last firm ground the player stood on, to climb back to
	local prop = level.propAt(lvl, cell.x, cell.y)
	local onPad = prop ~= nil and props.kinds[prop.kind].floor == true
	if ground and ground.walkable and not onPad then
		state.safePos = player.pos
		state.safeLevel = lvl.id
	end
end

return module
