--[[
Visual effects ("juice"): particles, floating texts, screen shake, flashes, flying arrows, ...

The game logic adds effects here and world_view.luau / postfx.luau draw them.
Effects are purely visual, they never change the game state.

Positions are in world space (tiles), with (x, y) the top-left corner of a tile.
]]
const loader = require("@vectarine/loader")
const types = require("./types")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

const module = loader.init() or {}

export type Particle = {
	pos: vec.Vec2,
	velocity: vec.Vec2,
	color: vec4.Vec4,
	size: number,
	age: number,
	duration: number,
	gravity: number,
}

export type Effect = {
	kind: "text" | "sprite" | "arrow" | "dying" | "ring",
	pos: vec.Vec2,
	age: number,
	duration: number,
	text: string?,
	color: vec4.Vec4?,
	sprite: string?,
	quarterTurns: number?,
	to: vec.Vec2?,
	flipX: boolean?,
}

module.list = module.list or {} :: { Effect }
module.particles = module.particles or {} :: { Particle }
--- Current screen shake strength, decays over time
module.shakeAmount = module.shakeAmount or 0
--- Color and strength of the full screen flash, decays over time
module.flashColor = module.flashColor or vec4.WHITE
module.flashAmount = module.flashAmount or 0
--- Increases by one every turn. Ambient particles use it so that they only move when the player does.
module.turnClock = module.turnClock or 0

const QUARTER_TURNS: { [types.Direction]: number } = { up = 0, right = 1, down = 2, left = 3 }

local function add(effect: Effect)
	table.insert(module.list, effect)
end

--- A text rising above a tile, for example "+1".
function module.floatText(pos: vec.Vec2, text: string, color: vec4.Vec4?)
	add({ kind = "text", pos = pos, age = 0, duration = 1.1, text = text, color = color or vec4.WHITE })
end

--- Show the item being used on the tile in front of the player.
function module.swing(pos: vec.Vec2, sprite: string, direction: types.Direction)
	add({ kind = "sprite", pos = pos, age = 0, duration = 0.2, sprite = sprite, quarterTurns = QUARTER_TURNS[direction] })
end

--- An arrow flying from a tile to another.
function module.arrow(from: vec.Vec2, to: vec.Vec2, direction: types.Direction)
	local duration = 0.03 * math.max(1, (to - from):length())
	add({ kind = "arrow", pos = from, to = to, age = 0, duration = duration, quarterTurns = QUARTER_TURNS[direction] })
end

--- A monster that just died: it spins and shrinks.
function module.dying(entity: types.Entity, _cause: string)
	add({ kind = "dying", pos = entity.pos, age = 0, duration = 0.45, sprite = entity.sprite, flipX = entity.facingLeft })
end

--- A picked up item jumps and disappears.
function module.pop(entity: types.Entity)
	add({ kind = "dying", pos = entity.pos, age = 0, duration = 0.35, sprite = entity.sprite })
end

const BURSTS: { [string]: { colors: { vec4.Vec4 }, speed: number, size: number, gravity: number, duration: number } } = {
	dust = {
		colors = { vec4.createColor(0.85, 0.8, 0.7, 0.8) },
		speed = 0.8,
		size = 0.07,
		gravity = -0.5,
		duration = 0.35,
	},
	debris = {
		colors = { vec4.createColor(0.55, 0.6, 0.7, 1), vec4.createColor(0.35, 0.37, 0.48, 1) },
		speed = 3.5,
		size = 0.11,
		gravity = 9,
		duration = 0.6,
	},
	sparkle = {
		colors = { vec4.createColor(1, 0.9, 0.4, 1), vec4.createColor(1, 1, 1, 1) },
		speed = 2.5,
		size = 0.08,
		gravity = 1,
		duration = 0.6,
	},
	poof = {
		colors = { vec4.createColor(0.95, 0.95, 0.95, 0.9), vec4.createColor(0.7, 0.7, 0.75, 0.9) },
		speed = 2,
		size = 0.13,
		gravity = -1,
		duration = 0.5,
	},
	blood = {
		colors = { vec4.createColor(0.9, 0.25, 0.25, 1), vec4.createColor(1, 0.5, 0.4, 1) },
		speed = 3.5,
		size = 0.1,
		gravity = 8,
		duration = 0.7,
	},
	fire = {
		colors = {
			vec4.createColor(1, 0.9, 0.4, 1),
			vec4.createColor(1, 0.55, 0.2, 1),
			vec4.createColor(0.9, 0.25, 0.15, 1),
		},
		speed = 4,
		size = 0.16,
		gravity = -2,
		duration = 0.55,
	},
	smoke = {
		colors = { vec4.createColor(0.25, 0.22, 0.25, 0.7), vec4.createColor(0.4, 0.38, 0.4, 0.6) },
		speed = 1.2,
		size = 0.22,
		gravity = -1.5,
		duration = 1,
	},
}

--- Throw particles from the center of a tile. `kind` is a key of BURSTS.
function module.burst(pos: vec.Vec2, kind: string, count: number)
	local style = BURSTS[kind]
	local center = pos + vec.V2(0.5, 0.5)
	for i = 1, count do
		local angle = math.random() * math.pi * 2
		local speed = style.speed * (0.4 + math.random() * 0.6)
		table.insert(module.particles, {
			pos = center + vec.V2(math.random() - 0.5, math.random() - 0.5):scale(0.4),
			velocity = vec.V2(math.cos(angle), math.sin(angle) - 0.4):scale(speed),
			color = style.colors[(i % #style.colors) + 1],
			size = style.size * (0.6 + math.random() * 0.8),
			age = 0,
			duration = style.duration * (0.6 + math.random() * 0.6),
			gravity = style.gravity,
		})
	end
end

function module.explosion(center: vec.Vec2)
	for dy = -1, 1 do
		for dx = -1, 1 do
			module.burst(center + vec.V2(dx, dy), "fire", 6)
		end
	end
	module.burst(center, "smoke", 14)
	add({ kind = "ring", pos = center, age = 0, duration = 0.35 })
	module.shake(0.45)
	module.flash(vec4.createColor(1, 0.85, 0.6, 1), 0.55)
end

function module.shake(amount: number)
	module.shakeAmount = math.max(module.shakeAmount, amount)
end

function module.flash(color: vec4.Vec4, amount: number)
	module.flashColor = color
	module.flashAmount = math.max(module.flashAmount, amount)
end

--- Called once per turn by rules.luau
function module.tick()
	module.turnClock += 1
end

function module.update(dt: number)
	for i = #module.list, 1, -1 do
		local effect = module.list[i]
		effect.age += dt
		if effect.age >= effect.duration then
			table.remove(module.list, i)
		end
	end
	for i = #module.particles, 1, -1 do
		local p = module.particles[i]
		p.age += dt
		if p.age >= p.duration then
			table.remove(module.particles, i)
		else
			p.velocity = p.velocity:scale(math.exp(-dt * 3)) + vec.V2(0, p.gravity * dt)
			p.pos = p.pos + p.velocity:scale(dt)
		end
	end
	module.shakeAmount = math.max(0, module.shakeAmount - dt * 1.4)
	module.flashAmount = math.max(0, module.flashAmount - dt * 2.5)
end

function module.clear()
	table.clear(module.list)
	table.clear(module.particles)
end

return module
