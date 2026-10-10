--[[
Ice blocks (props with `pushable`, see data/props.luau): walk into one for a moment and it slides away, cell by
cell, in a straight line, until something stops it: a wall, a solid prop, another block, a creature, the player.

- A block that slides into freezing water sinks and freezes it: the water becomes ice (a bridge).
- Blocks slide through the veils: the veils are only frozen dreams, and blocks don't dream.
- Rune plates (`plate`) are pressed by a block resting on them. When every plate with the same `data.opens` is
  pressed, what they open opens for good (see switches.luau).
- A frozen chest slides like a block. Next to a lit brazier, it thaws: it becomes a chest (the flag "thawed:<key>").

Blocks go back to their place when the player enters the level again: no puzzle can be ruined for good.
]]
const dream = require("./dream")
const effects = require("./effects")
const level = require("./level")
const loader = require("@vectarine/loader")
const monsters = require("./data/monsters")
const props = require("./data/props")
const switches = require("./switches")
const types = require("./types")
const vec = require("@vectarine/vec")

const module = loader.init() or {}

--- Seconds of walking into a block before it moves, and its speed (cells per second)
const PUSH_TIME = 0.22
const SLIDE_SPEED = 9
--- Seconds next to a lit brazier before a frozen chest thaws
const THAW_TIME = 1.5

type Slide = { prop: types.Prop, from: vec.Vec2, to: vec.Vec2, t: number, duration: number, fill: boolean }

module.state = {
	levelId = "",
	pushing = nil :: types.Prop?,
	pushTime = 0,
	--- Blocks sliding right now, by prop key
	slides = {} :: { [string]: Slide },
	--- How long each frozen chest has been next to a brazier, by prop key
	thaw = {} :: { [string]: number },
}

local function message(game: types.Game, text: string)
	game.events[#game.events + 1] = { kind = "message", text = text }
end

--- Is something standing on a cell (a creature, the player)? Blocks stop before it.
local function occupied(game: types.Game, cell: vec.Vec2): boolean
	local playerCell = level.cellOf(game.player.pos)
	if playerCell.x == cell.x and playerCell.y == cell.y then
		return true
	end
	for _, monster in game.monsters do
		local species = monsters.species[monster.kind]
		if not monster.dead and not species.flying and not species.prize and dream.isPresent(game, monster) then
			local c = level.cellOf(monster.pos)
			if c.x == cell.x and c.y == cell.y then
				return true
			end
		end
	end
	return false
end

--- Where a block pushed from `cell` towards `dir` stops, and whether it ends in water (which it fills)
local function destination(game: types.Game, cell: vec.Vec2, dir: vec.Vec2): (vec.Vec2, boolean)
	local lvl = game.level :: level.Level
	local current = cell
	while true do
		local next = current + dir
		if not level.inside(lvl, next.x, next.y) or level.wallAt(lvl, next.x, next.y) then
			return current, false
		end
		local prop = level.propAt(lvl, next.x, next.y)
		if prop and props.kinds[prop.kind].solid then
			return current, false
		end
		local ground = level.groundAt(lvl, next.x, next.y)
		if ground and ground.freezing then
			return next, true
		end
		if not ground or not ground.walkable or occupied(game, next) then
			return current, false
		end
		current = next
	end
end

--- Push a block towards `dir` (a unit vector along an axis). Returns true if it moved.
function module.push(game: types.Game, prop: types.Prop, dir: vec.Vec2): boolean
	local state = module.state
	if state.slides[prop.key] then
		return false
	end
	local to, fill = destination(game, prop.cell, dir)
	if to.x == prop.cell.x and to.y == prop.cell.y then
		return false
	end
	local from = prop.cell
	level.moveProp(game.level :: level.Level, prop, to)
	state.slides[prop.key] = { prop = prop, from = from, to = to, t = 0, duration = (to - from):length() / SLIDE_SPEED, fill = fill }
	effects.burst(from + vec.V2(0.5, 0.8), "snow", 6)
	return true
end

--- Where to draw a prop: sliding blocks are between two cells
function module.drawCell(prop: types.Prop): vec.Vec2
	local slide = module.state.slides[prop.key]
	if not slide then
		return prop.cell
	end
	return slide.from:lerp(slide.to, math.min(1, slide.t / math.max(0.01, slide.duration)))
end

--- When every plate of a group has a block on it, what they open opens
local function checkPlates(game: types.Game)
	local lvl = game.level :: level.Level
	local groups: { [string]: boolean } = {}
	for _, prop in lvl.props do
		if props.kinds[prop.kind].plate and prop.data.opens then
			local onIt = level.propAt(lvl, prop.cell.x, prop.cell.y)
			local pressed = onIt ~= nil and props.kinds[onIt.kind].pushable == true and not module.state.slides[onIt.key]
			local id = prop.data.opens :: string
			groups[id] = (groups[id] == nil or groups[id] == true) and pressed
		end
	end
	for id, pressed in groups do
		if pressed and switches.open(game, id) then
			message(game, "The runes under the ice glow. Somewhere, a gate of the Watch grinds open.")
		end
	end
end

local function arrive(game: types.Game, slide: Slide)
	local lvl = game.level :: level.Level
	local prop = slide.prop
	module.state.slides[prop.key] = nil
	effects.shake(0.08)
	if slide.fill then
		-- The block sinks in the freezing water, and freezes it: a bridge of ice
		level.setGround(lvl, slide.to.x, slide.to.y, "ice")
		level.clearProp(lvl, slide.to)
		local index = table.find(lvl.props, prop)
		if index then
			table.remove(lvl.props, index)
		end
		effects.burst(slide.to + vec.V2(0.5, 0.5), "water", 12)
		if not game.flags.iceBridgeHint then
			game.flags.iceBridgeHint = true
			message(game, "The block sinks with a splash... and the water freezes around it. A bridge of ice!")
		end
		return
	end
	checkPlates(game)
end

local function updateThaw(game: types.Game, dt: number)
	local lvl = game.level :: level.Level
	for _, prop in lvl.props do
		if prop.kind == "frozen_chest" and not module.state.slides[prop.key] then
			local warm = false
			for _, other in lvl.props do
				local kind = props.kinds[other.kind]
				if kind.warmth and (other.cell - prop.cell):length() < 1.6 then
					warm = true
				end
			end
			local time = if warm then (module.state.thaw[prop.key] or 0) + dt else 0
			module.state.thaw[prop.key] = time
			if warm and math.random() < dt * 6 then
				effects.burst(prop.cell + vec.V2(0.5, 0.3), "steam", 2)
			end
			if time > THAW_TIME then
				prop.kind = "chest"
				game.flags[`thawed:{prop.key}`] = true
				effects.burst(prop.cell + vec.V2(0.5, 0.5), "water", 16)
				message(game, "The ice melts in the heat of the brazier, and a chest comes out of it!")
			end
		end
	end
end

--- Called every frame. `move` is the direction the player is trying to go.
function module.update(game: types.Game, move: vec.Vec2, dt: number)
	local state = module.state
	if state.levelId ~= game.level.id then
		state.levelId = game.level.id
		state.slides = {}
		state.thaw = {}
		state.pushing = nil
	end
	for _, slide in state.slides do
		slide.t += dt
		if slide.t >= slide.duration then
			arrive(game, slide)
		end
	end
	updateThaw(game, dt)

	-- Walking into a block (along an axis) pushes it after a moment
	local dir: vec.Vec2? = nil
	if math.abs(move.x) > 0.5 and math.abs(move.y) < 0.5 then
		dir = vec.V2(if move.x > 0 then 1 else -1, 0)
	elseif math.abs(move.y) > 0.5 and math.abs(move.x) < 0.5 then
		dir = vec.V2(0, if move.y > 0 then 1 else -1)
	end
	local target: types.Prop? = nil
	if dir then
		local ahead = level.cellOf(game.player.pos + (dir :: vec.Vec2):scale(0.5))
		local prop = level.propAt(game.level :: level.Level, ahead.x, ahead.y)
		if prop and props.kinds[prop.kind].pushable and not state.slides[prop.key] then
			target = prop
		end
	end
	if target and target == state.pushing then
		state.pushTime += dt
		if state.pushTime > PUSH_TIME then
			module.push(game, target, dir :: vec.Vec2)
			state.pushing = nil
		end
	else
		state.pushing = target
		state.pushTime = 0
	end
end

return module
