--[[
How creatures behave. Each behavior is a small state machine updated every frame.

Every creature telegraphs its attacks (a worm shakes its mound, a snake coils, a beetle stops before charging),
so that a careful player can always react. `monster.state` is the current state, `monster.stateTime` the time spent in it.

To add a behavior, add a function to `behaviors` and use its name in data/monsters.luau.
]]
const combat = require("./combat")
const dream = require("./dream")
const effects = require("./effects")
const frost = require("./frost")
const level = require("./level")
const loader = require("@vectarine/loader")
const monsters = require("./data/monsters")
const theme = require("./theme")
const types = require("./types")
const vec = require("@vectarine/vec")

const module = loader.init() or {}

type Behavior = (game: types.Game, monster: types.Monster, species: monsters.Species, dt: number) -> ()

-- MARK: Helpers

local function setState(monster: types.Monster, state: string)
	monster.state = state
	monster.stateTime = 0
end

local function toPlayer(game: types.Game, monster: types.Monster): vec.Vec2
	return game.player.pos - monster.pos
end

--- Does the creature see the player? (close enough, same height unless flying, nothing in between)
local function seesPlayer(game: types.Game, monster: types.Monster, species: monsters.Species, range: number?): boolean
	local delta = toPlayer(game, monster)
	if delta:length() > (range or species.sight) then
		return false
	end
	if not species.flying and game.player.height ~= monster.height then
		return false
	end
	return level.lineIsClear(game.level :: any, monster.pos, game.player.pos)
end

local function faceTowards(monster: types.Monster, direction: vec.Vec2)
	if math.abs(direction.x) > 0.01 then
		monster.facing = if direction.x < 0 then "left" else "right"
	end
end

--- Move in a direction (a vector of any length: it is normalized). Returns false if blocked.
local function walk(game: types.Game, monster: types.Monster, species: monsters.Species, direction: vec.Vec2, speed: number, dt: number): boolean
	if direction:length() < 0.001 then
		return true
	end
	-- Cold slows creatures down (cold weapons)
	local chilled = if monster.chill and monster.chill > 0 then 0.5 else 1
	local delta = direction:normalized():scale(speed * chilled * dt)
	local r = species.radius
	local before = monster.pos
	-- Phantoms come from the dream: they drift through the veils
	local mover: level.Mover = { height = monster.height, flying = species.flying, dreaming = species.dream }
	monster.pos = level.move(game.level :: any, monster.pos, delta, r, r * 0.6, mover)
	faceTowards(monster, direction)
	return (monster.pos - before):length() > delta:length() * 0.5
end

--- Walk to a random point near the spawn, from time to time
local function wander(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
	if not monster.target or monster.stateTime > 2.5 or (monster.target - monster.pos):length() < 0.2 then
		local angle = math.random() * math.pi * 2
		monster.target = monster.spawn + vec.fromAngle(angle, math.random() * 2.5)
		monster.stateTime = math.random() * 1.5
	end
	if not walk(game, monster, species, (monster.target :: vec.Vec2) - monster.pos, species.speed * 0.35, dt) then
		monster.target = nil
	end
end

--- Seconds a marsh frog spends in the air
const HOP_TIME = 0.6
--- Bosses wake up when the player comes this close to their place in their arena
const ARENA_RADIUS = 9
const HEAL_COLORS = { theme.hex("73804b"), theme.hex("5f7b53"), theme.hex("e3ddd1") }

--- Bring a creature to life (bosses call for help)
local function summon(game: types.Game, kind: string, pos: vec.Vec2, height: number)
	local monster: types.Monster = {
		id = game.nextMonsterId,
		kind = kind,
		pos = pos,
		height = height,
		spawn = pos,
		hp = monsters.species[kind].hp,
		state = "hidden",
		stateTime = 0,
		facing = "down",
		cooldown = 0,
		hitTime = 10,
		knockback = vec.V2(0, 0),
		phase = 1,
		dead = false,
	}
	table.insert(game.monsters, monster)
	game.nextMonsterId += 1
end

--- Bosses wait in their arena: they sleep until the player comes close to their place, or hits them.
--- Returns true while the boss sleeps (the behavior then does nothing else).
local function sleepsInArena(game: types.Game, monster: types.Monster, wakeText: string): boolean
	if monster.state ~= "idle" and monster.state ~= "sleep" then
		return false
	end
	if (game.player.pos - monster.spawn):length() > ARENA_RADIUS and monster.hitTime > 0.2 then
		if monster.state ~= "sleep" then
			setState(monster, "sleep")
		end
		return true
	end
	effects.shake(0.3)
	game.events[#game.events + 1] = { kind = "message", text = wakeText }
	setState(monster, "wake")
	return true
end

-- MARK: Behaviors

module.behaviors = {
	--- Rats: wander, and run at the player once they see them. When close, they crouch (they shake) and pounce.
	chaser = function(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
		local delta = toPlayer(game, monster)
		monster.cooldown -= dt
		if monster.state == "crouch" then
			if monster.stateTime > 0.3 then
				monster.target = delta
				setState(monster, "pounce")
			end
		elseif monster.state == "pounce" then
			walk(game, monster, species, monster.target :: vec.Vec2, species.speed * 3.2, dt)
			if monster.stateTime > 0.22 then
				monster.cooldown = 1.1
				setState(monster, "chase")
			end
		elseif monster.state == "chase" then
			walk(game, monster, species, delta, species.speed, dt)
			if delta:length() < 1.7 and monster.cooldown <= 0 then
				faceTowards(monster, delta)
				setState(monster, "crouch")
			elseif not seesPlayer(game, monster, species, species.sight * 1.5) and monster.stateTime > 2 then
				setState(monster, "wander")
			end
		else
			wander(game, monster, species, dt)
			if seesPlayer(game, monster, species) then
				setState(monster, "chase")
			end
		end
	end,

	--- Bats: fly in loops around the player, and dive at them every few seconds.
	flyer = function(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
		local delta = toPlayer(game, monster)
		if monster.state == "dive" then
			walk(game, monster, species, monster.target or delta, species.speed * 2.2, dt)
			if monster.stateTime > 0.45 then
				setState(monster, "loop")
			end
		elseif seesPlayer(game, monster, species) then
			-- Circle around the player at about 2.5 tiles
			local angle = delta:scale(-1):angle() + dt * 1.8
			local wanted = game.player.pos + vec.fromAngle(angle, 2.5)
			walk(game, monster, species, wanted - monster.pos, species.speed, dt)
			if monster.stateTime > 2.2 then
				setState(monster, "dive")
				monster.target = delta
			end
		else
			wander(game, monster, species, dt)
		end
	end,

	--- Green bats: keep their distance and spit acid.
	spitter = function(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
		local delta = toPlayer(game, monster)
		if seesPlayer(game, monster, species) then
			local distance = delta:length()
			local sideways = vec.V2(-delta.y, delta.x):scale(math.sin(monster.stateTime * 1.3))
			if distance < 3 then
				walk(game, monster, species, delta:scale(-1) + sideways, species.speed, dt)
			elseif distance > 4.5 then
				walk(game, monster, species, delta + sideways, species.speed * 0.7, dt)
			else
				walk(game, monster, species, sideways, species.speed * 0.6, dt)
			end
			monster.cooldown -= dt
			if monster.cooldown <= 0 then
				monster.cooldown = 2.2
				combat.monsterShoot(game, monster, game.player.pos + vec.V2(0, -0.35))
			end
		else
			wander(game, monster, species, dt)
		end
	end,

	--- Beetles: slow, but when the player is lined up with them, they stop, then charge in a straight line.
	charger = function(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
		local delta = toPlayer(game, monster)
		if monster.state == "aim" then
			if monster.stateTime > 0.5 then
				setState(monster, "charge")
			end
		elseif monster.state == "charge" then
			local moved = walk(game, monster, species, monster.target :: vec.Vec2, species.speed * 4, dt)
			if not moved then
				-- Hitting a wall stuns the beetle: a chance to hit it
				effects.burst(monster.pos, "debris", 8)
				effects.shake(0.15)
				setState(monster, "stunned")
			elseif monster.stateTime > 1.1 then
				setState(monster, "chase")
			end
		elseif monster.state == "stunned" then
			if monster.stateTime > 1.3 then
				setState(monster, "chase")
			end
		elseif monster.state == "chase" then
			walk(game, monster, species, delta, species.speed, dt)
			local aligned = math.abs(delta.x) < 0.45 or math.abs(delta.y) < 0.45
			if aligned and delta:length() < 5 and monster.stateTime > 1 and seesPlayer(game, monster, species, 5) then
				-- Charge along the axis where the player is
				monster.target = if math.abs(delta.x) > math.abs(delta.y)
					then vec.V2(if delta.x > 0 then 1 else -1, 0)
					else vec.V2(0, if delta.y > 0 then 1 else -1)
				faceTowards(monster, delta)
				setState(monster, "aim")
			elseif not seesPlayer(game, monster, species, species.sight * 1.5) and monster.stateTime > 3 then
				setState(monster, "wander")
			end
		else
			wander(game, monster, species, dt)
			if seesPlayer(game, monster, species) then
				setState(monster, "chase")
			end
		end
	end,

	--- Worms: hidden under a mound that creeps towards the player. When close, the mound shakes, then the worm
	--- comes out and bites. It can only be hurt while it is out.
	burrower = function(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
		local delta = toPlayer(game, monster)
		if monster.state == "rising" then
			if monster.stateTime > 0.5 then
				effects.burst(monster.pos, "debris", 8)
				setState(monster, "up")
			end
		elseif monster.state == "up" then
			faceTowards(monster, delta)
			if monster.stateTime > 1.8 then
				effects.burst(monster.pos, "dust", 6)
				setState(monster, "hidden")
			end
		else
			if monster.state ~= "hidden" then
				setState(monster, "hidden")
			end
			if delta:length() < 6 and game.player.height == monster.height and monster.stateTime > 1.5 then
				walk(game, monster, species, delta, 1.1, dt)
				if delta:length() < species.sight then
					setState(monster, "rising")
				end
			end
		end
	end,

	--- Snakes: hide and wait. When the player is close, they coil up (they shake), then lunge, then rest.
	lunger = function(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
		local delta = toPlayer(game, monster)
		if monster.state == "coil" then
			if monster.stateTime > 0.55 then
				monster.target = delta
				setState(monster, "lunge")
			end
		elseif monster.state == "lunge" then
			walk(game, monster, species, monster.target :: vec.Vec2, species.speed * 4.5, dt)
			if monster.stateTime > 0.35 then
				setState(monster, "rest")
			end
		elseif monster.state == "rest" then
			if monster.stateTime > 0.9 then
				setState(monster, "approach")
			end
		elseif monster.state == "approach" then
			walk(game, monster, species, delta, species.speed, dt)
			if delta:length() < 2.8 and seesPlayer(game, monster, species) then
				faceTowards(monster, delta)
				setState(monster, "coil")
			elseif not seesPlayer(game, monster, species, species.sight * 1.5) and monster.stateTime > 3 then
				setState(monster, "wander")
			end
		else
			wander(game, monster, species, dt)
			if seesPlayer(game, monster, species) then
				setState(monster, "approach")
			end
		end
	end,

	--- Wasps: hover at a distance, dart at the player, then fly away.
	darter = function(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
		local delta = toPlayer(game, monster)
		if monster.state == "dart" then
			walk(game, monster, species, monster.target :: vec.Vec2, species.speed * 1.8, dt)
			if monster.stateTime > 0.4 then
				setState(monster, "retreat")
			end
		elseif monster.state == "retreat" then
			walk(game, monster, species, delta:scale(-1), species.speed, dt)
			if monster.stateTime > 0.5 then
				setState(monster, "hover")
			end
		elseif monster.state == "hover" then
			local angle = delta:scale(-1):angle() + dt * 2.5
			walk(game, monster, species, game.player.pos + vec.fromAngle(angle, 2.6) - monster.pos, species.speed * 0.6, dt)
			if monster.stateTime > 1.2 + (monster.id % 3) * 0.3 then
				monster.target = delta
				setState(monster, "dart")
			end
			if not seesPlayer(game, monster, species, species.sight * 1.5) then
				setState(monster, "wander")
			end
		else
			wander(game, monster, species, dt)
			if seesPlayer(game, monster, species) then
				setState(monster, "hover")
			end
		end
	end,

	--- Marsh frogs: they crouch, then leap towards the player and land on them (a small shockwave). In the air,
	--- nothing can hit them: their shadow on the ground shows where they will land.
	hopper = function(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
		local delta = toPlayer(game, monster)
		if monster.state == "crouch" then
			if monster.stateTime > 0.5 then
				local leap = if delta:length() > 3.5 then delta:normalized():scale(3.5) else delta
				monster.target = monster.pos + leap
				setState(monster, "jump")
			end
		elseif monster.state == "jump" then
			local target = monster.target :: vec.Vec2
			walk(game, monster, species, target - monster.pos, (target - monster.pos):length() / math.max(0.05, HOP_TIME - monster.stateTime), dt)
			if monster.stateTime > HOP_TIME then
				effects.burst(monster.pos, "dust", 8)
				if delta:length() < 1 then
					combat.creatureHurts(game, monster, species.damage, species.damageType, monster.pos, species.poison)
				end
				setState(monster, "rest")
			end
		elseif monster.state == "rest" then
			if monster.stateTime > 0.9 then
				setState(monster, "hunt")
			end
		elseif monster.state == "hunt" then
			walk(game, monster, species, delta, species.speed * 0.4, dt)
			if delta:length() < 4.5 and seesPlayer(game, monster, species) then
				faceTowards(monster, delta)
				setState(monster, "crouch")
			elseif not seesPlayer(game, monster, species, species.sight * 1.5) and monster.stateTime > 3 then
				setState(monster, "wander")
			end
		else
			wander(game, monster, species, dt)
			if seesPlayer(game, monster, species) then
				setState(monster, "hunt")
			end
		end
	end,

	--- Menders: medical automatons. They keep away from the player and mend the wounded creatures around them
	--- (a beam of green crosses flows to the creature they mend). Between two mendings, they zap the player:
	--- their lamp glows (they aim), then a spark flies.
	mender = function(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
		local delta = toPlayer(game, monster)
		monster.cooldown -= dt
		local mender = monster :: any
		if monster.state == "heal" then
			local patient = mender.patient :: types.Monster?
			if not patient or patient.dead then
				setState(monster, "keep")
				return
			end
			if monster.stateTime > 0.9 then
				local patientSpecies = monsters.species[patient.kind]
				local healed = math.min(species.heal or 6, patientSpecies.hp - patient.hp)
				patient.hp += healed
				effects.floatText(patient.pos + vec.V2(0, -1.2), `+{math.floor(healed)}`, HEAL_COLORS[1])
				effects.burst(patient.pos + vec.V2(0, -0.4), "leaves", 10)
				monster.cooldown = 2.5
				setState(monster, "keep")
			end
			return
		end
		if monster.state == "aim" then
			if monster.stateTime > 0.6 then
				combat.monsterShoot(game, monster, game.player.pos + vec.V2(0, -0.35))
				mender.zap = 2.2
				setState(monster, "keep")
			end
			return
		end
		mender.zap = (mender.zap or 1.5) - dt
		if seesPlayer(game, monster, species) and mender.zap <= 0 then
			faceTowards(monster, delta)
			setState(monster, "aim")
			return
		end
		if seesPlayer(game, monster, species) then
			-- Keep a few steps away from the player
			if delta:length() < 3 then
				walk(game, monster, species, delta:scale(-1), species.speed, dt)
			elseif delta:length() > 5 then
				walk(game, monster, species, delta, species.speed * 0.5, dt)
			end
			faceTowards(monster, delta)
		else
			wander(game, monster, species, dt)
		end
		if monster.cooldown <= 0 then
			-- The most wounded creature around (not itself)
			local best: types.Monster? = nil
			local bestMissing = 0
			for _, other in game.monsters do
				local otherSpecies = monsters.species[other.kind]
				local missing = otherSpecies.hp - other.hp
				if other ~= monster and not other.dead and not otherSpecies.hazard and missing > bestMissing
					and (other.pos - monster.pos):length() < 8 then
					best, bestMissing = other, missing
				end
			end
			if best then
				mender.patient = best
				setState(monster, "heal")
			end
		end
	end,

	--- Ooze dripping from a wall: a drop grows (its shadow too), falls, and splashes. Only the splash hurts.
	drip = function(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
		local t = monster.stateTime
		if monster.state == "form" then
			if t > 1.4 then
				setState(monster, "fall")
			end
		elseif monster.state == "fall" then
			if t > 0.3 then
				effects.burst(monster.pos, "leaves", 6)
				setState(monster, "splash")
			end
		elseif monster.state == "splash" then
			if t > 0.35 then
				setState(monster, "dry")
			end
		elseif t > 1 + (monster.id % 3) * 0.4 then
			setState(monster, "form")
		end
	end,

	--- A puddle of poison left by the Frog King: it dries after a while
	puddle = function(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
		if monster.stateTime > 6 and monster.state ~= "idle" then
			monster.dead = true
		end
		if monster.state == "idle" then
			setState(monster, "wet")
		end
	end,

	--- Sentinels: they never move. When they see the player, their lens glows (they aim), then they fire.
	turret = function(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
		local delta = toPlayer(game, monster)
		faceTowards(monster, delta)
		monster.cooldown -= dt
		if monster.state == "aim" then
			if monster.stateTime > 0.7 then
				combat.monsterShoot(game, monster, game.player.pos + vec.V2(0, -0.35))
				monster.cooldown = 1.3
				setState(monster, "idle")
			end
		elseif monster.cooldown <= 0 and seesPlayer(game, monster, species) then
			setState(monster, "aim")
		end
	end,
} :: { [string]: Behavior }

-- MARK: The Frog King

--- How high the Frog King leaps (in tiles): out of view
const KING_HEIGHT = 9
--- How long the tongue of the Frog King is, in tiles
module.TONGUE_LENGTH = 5.5

--- How high above the ground a creature is drawn (leaping frogs), in tiles
function module.lift(monster: types.Monster, species: monsters.Species): number
	local t = monster.stateTime
	if species.behavior == "hopper" and monster.state == "jump" then
		return math.sin(math.pi * math.min(1, t / HOP_TIME)) * 1.3
	elseif species.behavior == "frog_king" then
		if monster.state == "rise" then
			return math.min(1, t / 0.4) * KING_HEIGHT
		elseif monster.state == "air" or monster.state == "lock" then
			return KING_HEIGHT
		elseif monster.state == "fall" then
			return KING_HEIGHT * (1 - math.min(1, t / 0.3))
		end
	end
	return 0
end

--- Where the tongue of the Frog King reaches right now, if it is out: its base and its tip
function module.tongue(monster: types.Monster): (vec.Vec2?, vec.Vec2?)
	if monster.state ~= "lash" or not monster.target then
		return nil, nil
	end
	local length = module.TONGUE_LENGTH * math.sin(math.pi * math.min(1, monster.stateTime / 0.5))
	local base = monster.pos + vec.V2(0, -0.9)
	return base, base + (monster.target :: vec.Vec2):normalized():scale(length)
end

local function distanceToSegment(point: vec.Vec2, a: vec.Vec2, b: vec.Vec2): number
	local ab = b - a
	local t = math.clamp((point - a):dot(ab) / math.max(0.0001, ab:dot(ab)), 0, 1)
	return (point - (a + ab:scale(t))):length()
end

--- The boss of the wetlands. It sits, then crouches and leaps out of view: its shadow follows the player, stops,
--- and the King falls there, with a shockwave. Phase 2 (below 2/3 of its health): one attack out of two, it puffs
--- its throat and lashes its tongue at the player (poison). Phase 3 (below 1/3): it leaps twice in a row and leaves
--- puddles of poison where it lands.
module.behaviors.frog_king = function(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
	local fraction = monster.hp / species.hp
	local phase = if fraction > 2 / 3 then 1 elseif fraction > 1 / 3 then 2 else 3
	if phase ~= monster.phase then
		monster.phase = phase
		effects.shake(0.5)
		game.events[#game.events + 1] = { kind = "bossPhase", phase = phase }
	end
	local king = monster :: any
	local delta = toPlayer(game, monster)
	local t = monster.stateTime
	if sleepsInArena(game, monster, "The Frog King opens one eye... and croaks.") then
		return
	end
	if monster.state == "crouch" then
		if t > 0.6 then
			setState(monster, "rise")
		end
	elseif monster.state == "rise" then
		if t > 0.4 then
			setState(monster, "air")
		end
	elseif monster.state == "air" then
		-- The shadow follows the player
		monster.pos = monster.pos:lerp(game.player.pos, math.min(1, dt * 3.5))
		if t > 1.3 then
			setState(monster, "lock")
		end
	elseif monster.state == "lock" then
		if t > 0.5 then
			setState(monster, "fall")
		end
	elseif monster.state == "fall" then
		if t > 0.3 then
			effects.burst(monster.pos, "dust", 30)
			effects.shake(0.7)
			if delta:length() < 2.3 then
				combat.creatureHurts(game, monster, species.damage, "blunt", monster.pos)
			end
			if phase == 3 then
				for i = 0, 2 do
					summon(game, "poison_puddle", monster.pos + vec.fromAngle(i * 2.1 + game.time, 1.6), monster.height)
				end
			end
			king.leaps = (king.leaps or 0) + 1
			-- In phase 3, it leaps again right away once
			if phase == 3 and king.leaps % 2 == 1 then
				setState(monster, "crouch")
			else
				setState(monster, "sit")
			end
		end
	elseif monster.state == "puff" then
		monster.target = delta
		faceTowards(monster, delta)
		if t > 0.6 then
			setState(monster, "lash")
		end
	elseif monster.state == "lash" then
		local base, tip = module.tongue(monster)
		if base and tip and not king.tongueHit
			and distanceToSegment(game.player.pos + vec.V2(0, -0.35), base, tip) < 0.5 then
			king.tongueHit = true
			combat.creatureHurts(game, monster, species.damage - 2, "acid", tip, species.poison)
		end
		if t > 0.5 then
			king.tongueHit = false
			setState(monster, "sit")
		end
	else
		-- Sitting: open to hits
		if monster.state ~= "sit" then
			setState(monster, "sit")
		end
		if t > (if phase == 3 then 1 else 1.5) then
			king.attacks = (king.attacks or 0) + 1
			if phase >= 2 and king.attacks % 2 == 0 and delta:length() < module.TONGUE_LENGTH + 1 then
				setState(monster, "puff")
			else
				setState(monster, "crouch")
			end
		end
	end
end

-- MARK: The Great Mole


--- The boss of the oasis. Phase 1: she chases you and digs under your feet. Phase 2 (below 2/3 of her health):
--- worms come out with her. Phase 3 (below 1/3): she throws rocks between her digs, and moves faster.
module.behaviors.mole = function(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
	local fraction = monster.hp / species.hp
	local phase = if fraction > 2 / 3 then 1 elseif fraction > 1 / 3 then 2 else 3
	if phase ~= monster.phase then
		monster.phase = phase
		effects.shake(0.5)
		game.events[#game.events + 1] = { kind = "bossPhase", phase = phase }
	end
	local speed = species.speed * (if phase == 3 then 1.4 else 1)
	local delta = toPlayer(game, monster)
	if sleepsInArena(game, monster, "The ground trembles. The Great Mole wakes up!") then
		return
	end

	if monster.state == "underground" then
		-- Moving under the ground towards the player. Dust shows where she will come out.
		monster.target = game.player.pos
		walk(game, monster, species, delta, speed * 2.2, dt)
		if math.random() < dt * 12 then
			effects.burst(monster.pos, "dust", 2)
		end
		if monster.stateTime > 1.6 then
			setState(monster, "erupt")
		end
	elseif monster.state == "erupt" then
		if monster.stateTime > 0.45 then
			effects.burst(monster.pos, "debris", 20)
			effects.shake(0.45)
			if (game.player.pos - monster.pos):length() < 1.4 then
				combat.creatureHurts(game, monster, species.damage + 2, "blunt", monster.pos)
			end
			if phase >= 2 then
				for i = 1, 2 do
					local angle = math.random() * math.pi * 2
					summon(game, "worm", monster.pos + vec.fromAngle(angle, 2.5), monster.height)
				end
			end
			setState(monster, "chase")
		end
	elseif monster.state == "throw" then
		if monster.stateTime > 0.5 and monster.cooldown <= 0 then
			for _, offset in { -0.35, 0, 0.35 } do
				combat.monsterShoot(game, monster, game.player.pos, offset)
			end
			monster.cooldown = 1
		end
		monster.cooldown -= dt
		if monster.stateTime > 1.2 then
			setState(monster, "chase")
		end
	else
		if monster.state ~= "chase" then
			setState(monster, "chase")
		end
		walk(game, monster, species, delta, speed, dt)
		if monster.stateTime > (if phase == 3 then 3 else 4.5) then
			if phase == 3 and math.random() < 0.5 then
				monster.cooldown = 0
				setState(monster, "throw")
			else
				effects.burst(monster.pos, "debris", 14)
				setState(monster, "underground")
			end
		end
	end
end

-- MARK: The Sun Warden

--- A ring of shots around the monster, starting at `angle`
local function ring(game: types.Game, monster: types.Monster, count: number, angle: number)
	for i = 0, count - 1 do
		local a = angle + i * 2 * math.pi / count
		combat.monsterShoot(game, monster, monster.pos + vec.V2(0, -0.4) + vec.fromAngle(a, 3))
	end
end

--- The water drunk by the Sun Warden heals it: health per point of water (100 points fill the water bar)
const DRAIN_HEAL = 1.2
const WATER_COLORS = { theme.hex("4e7499"), theme.hex("879ea3"), theme.hex("5e7e8b") }

--- The Warden drinks the water of the player: the water bar first, then the sips of the bottle. It heals with it.
--- The everflowing amphora never empties: the Warden drinks too much, floods and short-circuits.
local function drain(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
	local player = game.player
	effects.stream(player.pos + vec.V2(0, -0.4), monster.pos + vec.V2(0, -1.2), WATER_COLORS, 2)
	if player.artefacts.everflowing_amphora then
		if monster.stateTime > 0.8 then
			effects.burst(monster.pos + vec.V2(0, -1), "spark", 30)
			effects.shake(0.6)
			if not game.flags.wardenFlooded then
				game.flags.wardenFlooded = true
				game.events[#game.events + 1] = { kind = "message", text = "The amphora never empties: the Warden drinks too much and short-circuits!" }
			end
			setState(monster, "flooded")
		end
		return
	end
	-- Even when the player has nothing left to drink, it pulls the moisture of the air: it always heals
	local drunk = math.min(player.water, 45 * dt) + 25 * dt
	player.water -= drunk
	if player.water <= 0 and player.sips > 0 and monster.stateTime % 0.5 < dt then
		-- The bottle empties too, a sip at a time
		player.sips -= 1
		drunk += 40
	end
	monster.hp = math.min(species.hp, monster.hp + drunk * DRAIN_HEAL)
	if drunk > 0 and not game.flags.wardenDrank then
		game.flags.wardenDrank = true
		game.events[#game.events + 1] = { kind = "message", text = "The Warden drinks your water, and its bronze mends itself!" }
	end
end

--- The boss of the desert. Phase 1: it walks to the player, raises its fists (it shakes), and stomps: the ground
--- hurts around it and a ring of sand flies. After each attack it vents, open to hits.
--- Every third attack, its eye turns blue and it drinks the water of the player, which heals it (see `drain`):
--- without the everflowing amphora, the fight can't be won. With it, the Warden floods and is open to every hit.
--- Phase 2 (below 2/3 of its health): it also fires a beam of sparks sweeping across the player.
--- Phase 3 (below 1/3): it walks faster and stomps twice.
module.behaviors.warden = function(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
	local fraction = monster.hp / species.hp
	local phase = if fraction > 2 / 3 then 1 elseif fraction > 1 / 3 then 2 else 3
	if phase ~= monster.phase then
		monster.phase = phase
		effects.shake(0.5)
		game.events[#game.events + 1] = { kind = "bossPhase", phase = phase }
	end
	local delta = toPlayer(game, monster)
	if sleepsInArena(game, monster, "The eye of the Sun Warden lights up. Bronze grinds against bronze.") then
		return
	end

	if monster.state == "raise" then
		if monster.stateTime > 0.75 then
			setState(monster, "stomp")
		end
	elseif monster.state == "stomp" then
		-- The first stomp right away; in phase 3, a second one a moment later
		local stomps = if phase == 3 then 2 else 1
		local done = monster.cooldown
		if monster.stateTime >= done * 0.4 and done < stomps then
			effects.burst(monster.pos, "dust", 24)
			effects.shake(0.6)
			if delta:length() < 2.2 then
				combat.creatureHurts(game, monster, species.damage + 3, "blunt", monster.pos)
			end
			ring(game, monster, 12, done * math.pi / 12)
			monster.cooldown = done + 1
		end
		if monster.stateTime > 0.4 * stomps + 0.2 then
			setState(monster, "vent")
		end
	elseif monster.state == "charge" then
		monster.target = delta
		if monster.stateTime > 0.9 then
			setState(monster, "beam")
		end
	elseif monster.state == "beam" then
		-- The beam sweeps from one side of the player to the other
		local aim = (monster.target :: vec.Vec2):angle()
		local angle = aim - 0.9 + 1.8 * (monster.stateTime / 2)
		monster.cooldown -= dt
		if monster.cooldown <= 0 then
			combat.monsterShoot(game, monster, monster.pos + vec.V2(0, -0.4) + vec.fromAngle(angle, 3))
			monster.cooldown = 0.08
		end
		if monster.stateTime > 2 then
			setState(monster, "vent")
		end
	elseif monster.state == "drain" then
		drain(game, monster, species, dt)
		if monster.state == "drain" and monster.stateTime > 2.4 then
			setState(monster, "vent")
		end
	elseif monster.state == "flooded" then
		if math.random() < dt * 8 then
			effects.burst(monster.pos + vec.V2(0, -1), "spark", 4)
		end
		if monster.stateTime > 3 then
			setState(monster, "walk")
		end
	elseif monster.state == "vent" then
		if math.random() < dt * 10 then
			effects.burst(monster.pos + vec.V2(0, -1.2), "dust", 2)
		end
		if monster.stateTime > 1.2 then
			setState(monster, "walk")
		end
	else
		if monster.state ~= "walk" then
			setState(monster, "walk")
		end
		walk(game, monster, species, delta, species.speed * (if phase == 3 then 1.5 else 1), dt)
		if delta:length() < 2.4 or monster.stateTime > 3.5 then
			-- The attacks follow each other: stomp, beam (from phase 2) or stomp, drain
			local warden = monster :: any
			warden.attacks = (warden.attacks or 0) + 1
			monster.cooldown = 0
			if warden.attacks % 3 == 0 then
				setState(monster, "drain")
			elseif phase >= 2 and warden.attacks % 3 == 2 then
				setState(monster, "charge")
			else
				setState(monster, "raise")
			end
		end
	end
end


-- MARK: The frontier

--- Tusk seals: they crawl on the snow, but on the ice they rear up (they aim), then slide on their belly towards
--- the player, fast, in a straight line, until something stops them.
module.behaviors.slider = function(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
	local delta = toPlayer(game, monster)
	local cell = level.cellOf(monster.pos)
	local onIce = (function()
		local ground = level.groundAt(game.level :: any, cell.x, cell.y)
		return ground ~= nil and ground.slippery == true
	end)()
	if monster.state == "aim" then
		if monster.stateTime > 0.65 then
			setState(monster, "slide")
		end
	elseif monster.state == "slide" then
		local moved = walk(game, monster, species, monster.target :: vec.Vec2, 7.5, dt)
		if not moved or monster.stateTime > (if onIce then 1.6 else 0.4) then
			if not moved then
				effects.burst(monster.pos, "snow", 8)
				effects.shake(0.12)
			end
			setState(monster, "rest")
		end
	elseif monster.state == "rest" then
		if monster.stateTime > 1.2 then
			setState(monster, "chase")
		end
	elseif monster.state == "chase" then
		walk(game, monster, species, delta, species.speed, dt)
		if onIce and delta:length() < 7 and monster.stateTime > 0.6 and seesPlayer(game, monster, species) then
			monster.target = delta
			faceTowards(monster, delta)
			setState(monster, "aim")
		elseif not seesPlayer(game, monster, species, species.sight * 1.5) and monster.stateTime > 3 then
			setState(monster, "wander")
		end
	else
		wander(game, monster, species, dt)
		if seesPlayer(game, monster, species) then
			setState(monster, "chase")
		end
	end
end

--- Snowmen of the gallery: hidden behind a snow wall, they pop up, stay a moment, and sink back.
--- A hit knocks them "down" for a while.
module.behaviors.popup = function(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
	local t = monster.stateTime
	if monster.state == "rise" then
		if t > 0.25 then
			setState(monster, "up")
		end
	elseif monster.state == "up" then
		if t > 1.5 then
			setState(monster, "sink")
		end
	elseif monster.state == "sink" then
		if t > 0.25 then
			setState(monster, "hidden")
			monster.stateTime = -math.random() * 1.6
		end
	elseif monster.state == "down" then
		if t > 0.8 then
			setState(monster, "hidden")
			monster.stateTime = -math.random() * 1.6
		end
	elseif t > 0.5 then
		setState(monster, "rise")
	end
end

--- Pucks of the gallery: they slide back and forth on the rink, and bounce on its edges
module.behaviors.puck = function(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
	if monster.state == "down" then
		if monster.stateTime > 3 then
			monster.pos = monster.spawn
			setState(monster, "slide")
		end
		return
	end
	if monster.state ~= "slide" then
		if monster.stateTime > 0 then
			setState(monster, "slide")
		end
		return
	end
	local direction = monster.target or vec.V2(1, 0)
	if not walk(game, monster, species, direction, species.speed, dt) then
		monster.target = direction:scale(-1)
	end
end

--- The frost hare of the gallery: it waits in its burrow, then dashes across the yard
module.behaviors.hare = function(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
	local direction = monster.target or vec.V2(1, 0)
	if monster.state == "run" then
		walk(game, monster, species, direction, species.speed, dt)
		if monster.stateTime > 2.4 then
			monster.pos = monster.spawn
			setState(monster, "hidden")
			monster.stateTime = -2 - math.random() * 3
		end
	elseif monster.state == "down" then
		monster.pos = monster.spawn
		setState(monster, "hidden")
		monster.stateTime = -3 - math.random() * 3
	elseif monster.stateTime > 0.5 then
		setState(monster, "run")
	end
end

-- MARK: The Last Watchman

--- How far the halberd of the Watchman reaches, and how wide it sweeps (cosine of the half angle)
const SWEEP_REACH = 2.6
const SWEEP_WIDTH = 0.1
--- At most this many shades fight at his side
const MAX_SHADES = 4

local function countAlive(game: types.Game, kind: string): number
	local count = 0
	for _, other in game.monsters do
		if other.kind == kind and not other.dead then
			count += 1
		end
	end
	return count
end

--- The boss of the frontier, the captain of the Watch who never slept. To the waking he is a husk of ice (`husk`):
--- only a dreaming player can really hurt him, but the dreaming also see his shades, which he calls to his side.
--- He walks up to the player, raises his halberd (it glows), and sweeps in front of him. From afar, he throws fans
--- of ice shards. Phase 2 (below 60%): he slides across the ice at the player and stuns himself on the walls.
--- Phase 3 (below 30%): a blizzard fills the hall (the cold bites, see frost.luau): stay close to the braziers.
module.behaviors.watchman = function(game: types.Game, monster: types.Monster, species: monsters.Species, dt: number)
	local fraction = monster.hp / species.hp
	local phase = if fraction > 0.6 then 1 elseif fraction > 0.3 then 2 else 3
	if phase ~= monster.phase then
		monster.phase = phase
		effects.shake(0.5)
		game.events[#game.events + 1] = {
			kind = "message",
			text = if phase == 2
				then "The Watchman's ice cracks. He crouches, ready to slide across the hall!"
				else "The Watchman roars. A blizzard fills the hall: stay by the fire!",
		}
	end
	if sleepsInArena(game, monster, "The frozen captain lifts his head. 'Who goes there? The Watch... never... sleeps.'") then
		return
	end
	if phase >= 3 then
		frost.state.blizzardUntil = game.time + 0.5
	end
	local captain = monster :: any
	local delta = toPlayer(game, monster)
	local t = monster.stateTime
	local dreaming = dream.isDreaming(game)
	monster.cooldown -= dt
	captain.summon = (captain.summon or 6) - dt

	-- The dreaming see his shades: he calls them, a few at a time
	if dreaming and captain.summon <= 0 and countAlive(game, "shade") < MAX_SHADES then
		captain.summon = if phase >= 2 then 7 else 9
		for _, side in { -1, 1 } do
			summon(game, "shade", monster.pos + vec.V2(side * 1.6, 0.4), monster.height)
		end
	end

	if monster.state == "wake" then
		if t > 1 then
			setState(monster, "walk")
		end
	elseif monster.state == "raise" then
		-- The halberd glows: step back or around
		if t > 0.7 then
			setState(monster, "sweep")
			effects.shake(0.2)
			local facing = monster.target or delta
			local toPlayerNow = toPlayer(game, monster)
			local aligned = toPlayerNow:length() < 0.01 or toPlayerNow:normalized():dot(facing:normalized()) > SWEEP_WIDTH
			if toPlayerNow:length() < SWEEP_REACH and aligned then
				combat.creatureHurts(game, monster, species.damage, species.damageType, monster.pos)
			end
		end
	elseif monster.state == "sweep" then
		if t > 0.35 then
			setState(monster, "recover")
		end
	elseif monster.state == "recover" then
		-- The halberd is down: the moment to strike
		if t > 1.1 then
			setState(monster, "walk")
		end
	elseif monster.state == "cast" then
		if t > 0.6 then
			local base = delta:angle()
			if phase >= 3 then
				ring(game, monster, 12, base)
			else
				for i = -2, 2 do
					combat.monsterShoot(game, monster, game.player.pos + vec.V2(0, -0.35), i * 0.22)
				end
			end
			monster.cooldown = 3.5
			setState(monster, "walk")
		end
	elseif monster.state == "aim" then
		if t > 0.75 then
			setState(monster, "slide")
		end
	elseif monster.state == "slide" then
		local moved = walk(game, monster, species, monster.target :: vec.Vec2, 8, dt)
		if not moved or t > 1.8 then
			effects.burst(monster.pos, "snow", 16)
			effects.shake(0.4)
			setState(monster, if moved then "walk" else "stunned")
		end
	elseif monster.state == "stunned" then
		if t > 1.8 then
			setState(monster, "walk")
		end
	else
		-- Walk up to the player: faster for a dreamer, who sees him as he is
		walk(game, monster, species, delta, species.speed * (if dreaming then 1.35 else 1), dt)
		if delta:length() < 2.2 then
			monster.target = delta
			faceTowards(monster, delta)
			setState(monster, "raise")
		elseif monster.cooldown <= 0 and phase >= 2 and t > 1 and math.random() < 0.5 then
			monster.target = delta
			monster.cooldown = 4
			faceTowards(monster, delta)
			setState(monster, "aim")
		elseif monster.cooldown <= 0 and t > 1 then
			faceTowards(monster, delta)
			setState(monster, "cast")
		end
	end
end

--- Does touching this creature hurt right now?
function module.isDangerous(monster: types.Monster): boolean
	-- Sentinels only hurt with their shots: walking up to them to hit them is the way to fight them
	local species = monsters.species[monster.kind]
	if monster.dead or species.behavior == "turret" or species.prize then
		return false
	end
	-- The Watchman hurts with his halberd and his shards; touching him only hurts when he slides into you
	if species.behavior == "watchman" then
		return monster.state == "slide"
	end
	local state = monster.state
	-- Ooze only hurts when it splashes; leaping frogs don't touch anyone while in the air
	if species.behavior == "drip" then
		return state == "splash"
	end
	if species.behavior == "puddle" then
		return true
	end
	return state ~= "hidden" and state ~= "underground" and state ~= "rising" and state ~= "erupt" and state ~= "flooded"
		and state ~= "jump" and state ~= "rise" and state ~= "air" and state ~= "lock" and state ~= "fall"
end

function module.update(game: types.Game, monster: types.Monster, dt: number)
	local species = monsters.species[monster.kind]
	monster.stateTime += dt
	monster.hitTime += dt
	if monster.chill then
		monster.chill = math.max(0, monster.chill - dt)
	end
	if monster.knockback:length() > 0.05 then
		local r = species.radius
		local lvl = game.level :: any
		local cell = level.cellOf(monster.pos)
		local ground = level.groundAt(lvl, cell.x, cell.y)
		-- On the ice, a creature knocked back slides far, maybe into the freezing water
		local onIce = ground ~= nil and ground.slippery == true and not species.flying
		monster.pos = level.move(
			lvl,
			monster.pos,
			monster.knockback:scale(dt),
			r,
			r * 0.6,
			{ height = monster.height, flying = species.flying, falls = onIce and not species.boss }
		)
		monster.knockback = monster.knockback:scale(math.exp(-dt * (if onIce then 2.2 else 10)))
		cell = level.cellOf(monster.pos)
		ground = level.groundAt(lvl, cell.x, cell.y)
		if ground and ground.freezing and not species.flying then
			effects.burst(monster.pos, "water", 14)
			game.events[#game.events + 1] = { kind = "message", text = `The {string.lower(species.name)} slides into the freezing water!` }
			combat.killMonster(game, monster)
		end
		return
	end
	module.behaviors[species.behavior](game, monster, species, dt)
end

return module
