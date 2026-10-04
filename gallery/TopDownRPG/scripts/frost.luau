--[[
The cold of the frontier, for the player (see data/terrain.luau and data/props.luau):

- Warmth (0 to 100). Cold places take it away: snow slowly, the killing frost of the whiteout fast, freezing water
  for whoever sails on it or falls in. Anywhere else it comes back slowly, and fast near a fire, a lit brazier or a
  hot spring. At 0 the player freezes: slow, and losing health. Cold hits (frost bats...) chill too.
  The fur cloak (and warm charms) slow the cold down; the watch ember (an artefact) keeps the player warm.
- Ice is slippery: the player keeps their momentum, and steers slowly (see `steer`).
- Thin ice cracks under whoever stands on it a moment: it breaks into freezing water. The player falls in and
  climbs back to the last firm ground, hurt and chilled.
- Braziers are lit with [E] (see interactions.luau): they stay lit, and warm the place.
]]
const effects = require("./effects")
const inventory = require("./inventory")
const items = require("./data/items")
const level = require("./level")
const loader = require("@vectarine/loader")
const player_rules = require("./player")
const props = require("./data/props")
const types = require("./types")
const vec = require("@vectarine/vec")

const module = loader.init() or {}

--- Warmth lost per second sailing on the dark water of the frontier
const SAILING_COLD = 18
--- Warmth regained per second away from the cold, and near a source of heat
const RECOVERY = 3
const HEAT = 30
--- Warmth lost when falling in freezing water, and the health it costs
const PLUNGE_COLD = 40
const PLUNGE_DAMAGE = 9
--- Health lost per second while freezing, and the speed of a frozen player
const FREEZING_DAMAGE = 1.5
const FROZEN_SPEED = 0.55
--- Seconds standing on thin ice before it breaks
const THIN_ICE_TIME = 0.8
--- How fast the player gets to the speed they want on ice (per second): low = slippery
const ICE_GRIP = 1.7

--- State of the frontier that is not saved: the thin ice under the player, where they last stood on firm ground
type State = {
	thinCell: vec.Vec2?,
	thinTime: number,
	safePos: vec.Vec2?,
	safeLevel: string,
	freezingHintTime: number,
	blizzardUntil: number,
}

module.state = {
	thinCell = nil :: vec.Vec2?,
	thinTime = 0,
	safePos = nil :: vec.Vec2?,
	safeLevel = "",
	freezingHintTime = -100,
	--- The blizzard of the Last Watchman blows until this time (see monster_ai.luau)
	blizzardUntil = -100,
} :: State

--- Warmth lost per second in the blizzard of the Last Watchman
const BLIZZARD_COLD = 7

--- Is the blizzard of the Last Watchman blowing?
function module.isBlizzard(game: types.Game): boolean
	return game.time < module.state.blizzardUntil
end

--- How much of the cold reaches the player (1 = all of it): the fur cloak and warm charms keep some away
function module.coldFactor(player: types.Player): number
	local armorId = player.equipment.armor
	local armor = if armorId then items.items[armorId] else nil
	local factor = if armor and armor.warm then armor.warm else 1
	return factor * math.max(0.2, 1 - inventory.charmBonus(player, "warm"))
end

--- Is a source of heat close to this point? (fires, lit braziers, hot springs)
function module.isNearHeat(game: types.Game, pos: vec.Vec2): boolean
	local lvl = game.level :: level.Level
	for _, prop in lvl.props do
		local warmth = props.kinds[prop.kind].warmth
		if warmth and (prop.cell + vec.V2(0.5, 0.5) - pos):length() < warmth then
			return true
		end
	end
	local cell = level.cellOf(pos)
	for y = cell.y - 2, cell.y + 2 do
		for x = cell.x - 2, cell.x + 2 do
			local ground = level.groundAt(lvl, x, y)
			if ground and ground.warm then
				return true
			end
		end
	end
	return false
end

--- Speed multiplier of a freezing player
function module.speedFactor(player: types.Player): number
	return if player.warmth <= 0 then FROZEN_SPEED else 1
end

--- The velocity of the player this frame: on ice they keep their momentum and steer slowly
function module.steer(game: types.Game, wanted: vec.Vec2, dt: number): vec.Vec2
	local player = game.player
	local cell = level.cellOf(player.pos)
	local ground = level.groundAt(game.level :: level.Level, cell.x, cell.y)
	if ground and ground.slippery then
		player.velocity = player.velocity:lerp(wanted, 1 - math.exp(-dt * ICE_GRIP))
	else
		player.velocity = wanted
	end
	return player.velocity
end

--- After moving: what was blocked stops (the player doesn't keep pushing against a wall)
function module.afterMove(game: types.Game, before: vec.Vec2, dt: number)
	local player = game.player
	if dt <= 0 then
		return
	end
	local moved = (player.pos - before):scale(1 / dt)
	local velocity = player.velocity
	-- A wall stops the momentum on its axis (with a small bounce on ice)
	local vx = if math.abs(moved.x) < math.abs(velocity.x) * 0.5 then -velocity.x * 0.2 else velocity.x
	local vy = if math.abs(moved.y) < math.abs(velocity.y) * 0.5 then -velocity.y * 0.2 else velocity.y
	player.velocity = vec.V2(vx, vy)
end

local function message(game: types.Game, text: string)
	game.events[#game.events + 1] = { kind = "message", text = text }
end

--- Chill the player (a cold hit, a plunge): warmth lost, less with warm clothes
function module.chill(player: types.Player, amount: number)
	if player.artefacts.watch_ember then
		return
	end
	player.warmth = math.max(0, player.warmth - amount * module.coldFactor(player))
end

--- The player falls in freezing water: back to the last firm ground, hurt and chilled
local function plunge(game: types.Game)
	local player = game.player
	local state = module.state
	effects.burst(player.pos, "water", 14)
	player_rules.hurt(game, PLUNGE_DAMAGE, "cold", player.pos)
	module.chill(player, PLUNGE_COLD)
	player.knockback = vec.V2(0, 0)
	player.velocity = vec.V2(0, 0)
	-- Back on firm ground: where the player last stood (or the closest firm ground), never on thin ice
	local lvl = game.level :: level.Level
	local wanted = if state.safeLevel == lvl.id then state.safePos else nil
	local landing = level.landingSpot(lvl, wanted, player.pos, player_rules.RX, player_rules.RY, { height = player.height }, function(x: number, y: number): boolean
		local ground = level.groundAt(lvl, x, y)
		return ground ~= nil and not ground.thin
	end)
	if landing then
		player.pos = landing
	end
	message(game, "The ice breaks! You fall in the freezing water, and drag yourself out, shaking.")
end

--- Called every frame, after the player moved
function module.update(game: types.Game, dt: number)
	local player = game.player
	local state: State = module.state
	local lvl = game.level :: level.Level
	local cell = level.cellOf(player.pos)
	local ground = level.groundAt(lvl, cell.x, cell.y)

	-- Thin ice breaks under the player after a moment
	if ground and ground.thin then
		local thinCell = state.thinCell
		if thinCell and thinCell.x == cell.x and thinCell.y == cell.y then
			state.thinTime += dt
		else
			state.thinCell = cell
			state.thinTime = 0
		end
		if state.thinTime > THIN_ICE_TIME then
			level.setGround(lvl, cell.x, cell.y, "cold_water")
			effects.burst(cell + vec.V2(0.5, 0.5), "debris", 10)
			state.thinCell = nil
			ground = level.groundAt(lvl, cell.x, cell.y)
		end
	else
		state.thinCell = nil
	end
	-- Standing on freezing water without a boat: the ice broke under the player
	if ground and ground.freezing and not (player.artefacts.ferry_skiff and ground.sailable) then
		plunge(game)
		return
	end
	if ground and ground.walkable and not ground.thin then
		state.safePos = player.pos
		state.safeLevel = lvl.id
	end

	-- Warmth
	if player.artefacts.watch_ember then
		player.warmth = 100
		return
	end
	local cold = if ground then ground.cold or (if ground.freezing then SAILING_COLD else 0) else 0
	if module.isBlizzard(game) then
		cold = math.max(cold, BLIZZARD_COLD)
	end
	if module.isNearHeat(game, player.pos) then
		player.warmth = math.min(100, player.warmth + HEAT * dt)
	elseif cold > 0 then
		player.warmth = math.max(0, player.warmth - cold * module.coldFactor(player) * dt)
	else
		player.warmth = math.min(100, player.warmth + RECOVERY * dt)
	end
	if player.warmth <= 0 then
		player.hp -= FREEZING_DAMAGE * dt
		if game.time - state.freezingHintTime > 20 then
			state.freezingHintTime = game.time
			message(game, "You are freezing! Get to a fire, or out of the snow, before the cold takes you.")
		end
		if player.hp <= 0 and player.invincible <= 0 then
			player.hp = 0
			game.events[#game.events + 1] = { kind = "fainted" }
		end
	end
end

return module
