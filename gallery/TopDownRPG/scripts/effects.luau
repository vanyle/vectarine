--[[
Visual effects ("juice"): particles, floating texts, swings, hit sparks, screen shake and flashes.

The game logic adds effects here, world_view.luau and postfx.luau draw them.
Effects are purely visual: they never change the state of the game.

Positions are in world space (tiles, y pointing down).
]]
const loader = require("@vectarine/loader")
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
	--- How fast the particle slows down (3 when not set)
	drag: number?,
}

export type Effect = {
	kind: "text" | "sprite" | "dying",
	pos: vec.Vec2,
	age: number,
	duration: number,
	text: string?,
	color: vec4.Vec4?,
	--- For animated sprites: the frames, played once over the duration
	frames: { string }?,
	angle: number?,
	flipX: boolean?,
	size: number?,
}

module.list = module.list or {} :: { Effect }
module.particles = module.particles or {} :: { Particle }
--- Current screen shake strength, decays over time
module.shakeAmount = module.shakeAmount or 0
--- Color and strength of the full screen flash, decays over time
module.flashColor = module.flashColor or vec4.WHITE
module.flashAmount = module.flashAmount or 0

local function add(effect: Effect)
	table.insert(module.list, effect)
end

--- A text rising from a point, for example "-3".
function module.floatText(pos: vec.Vec2, text: string, color: vec4.Vec4?)
	add({ kind = "text", pos = pos, age = 0, duration = 0.9, text = text, color = color or vec4.WHITE })
end

--- An animation played once at a point (a weapon swing, a hit spark...). `pos` is the center of the sprite.
function module.animate(pos: vec.Vec2, frames: { string }, duration: number, angle: number?, size: number?)
	add({ kind = "sprite", pos = pos, age = 0, duration = duration, frames = frames, angle = angle, size = size })
end

--- A creature that just died: it spins, shrinks and fades. `pos` is the center of the sprite.
function module.dying(pos: vec.Vec2, sprite: string, flipX: boolean?, size: number?)
	add({ kind = "dying", pos = pos, age = 0, duration = 0.45, frames = { sprite }, flipX = flipX, size = size })
end

const BURSTS: { [string]: { colors: { vec4.Vec4 }, speed: number, size: number, gravity: number, duration: number } } =
	{
		dust = {
			colors = { vec4.createColor(0.59, 0.57, 0.45, 0.8) },
			speed = 0.8,
			size = 0.07,
			gravity = -0.5,
			duration = 0.35,
		},
		debris = {
			colors = { vec4.createColor(0.47, 0.45, 0.36, 1), vec4.createColor(0.31, 0.29, 0.23, 1) },
			speed = 3.5,
			size = 0.11,
			gravity = 9,
			duration = 0.6,
		},
		sparkle = {
			colors = { vec4.createColor(0.89, 0.87, 0.82, 1), vec4.createColor(0.65, 0.53, 0.29, 1) },
			speed = 2.5,
			size = 0.08,
			gravity = 1,
			duration = 0.6,
		},
		poof = {
			colors = { vec4.createColor(0.74, 0.73, 0.68, 0.9), vec4.createColor(0.47, 0.54, 0.57, 0.9) },
			speed = 2,
			size = 0.13,
			gravity = -1,
			duration = 0.5,
		},
		blood = {
			colors = { vec4.createColor(0.51, 0.28, 0.24, 1), vec4.createColor(0.38, 0.15, 0.22, 1) },
			speed = 3,
			size = 0.09,
			gravity = 8,
			duration = 0.6,
		},
		leaves = {
			colors = { vec4.createColor(0.45, 0.5, 0.29, 1), vec4.createColor(0.37, 0.48, 0.33, 1) },
			speed = 2.2,
			size = 0.1,
			gravity = 3,
			duration = 0.7,
		},
		water = {
			colors = { vec4.createColor(0.31, 0.45, 0.6, 1), vec4.createColor(0.53, 0.62, 0.64, 1) },
			speed = 2,
			size = 0.08,
			gravity = 6,
			duration = 0.5,
		},
		spark = {
			colors = { vec4.createColor(0.53, 0.62, 0.64, 1), vec4.createColor(0.89, 0.87, 0.82, 1) },
			speed = 4,
			size = 0.07,
			gravity = 0,
			duration = 0.35,
		},
		snow = {
			colors = { vec4.createColor(0.89, 0.87, 0.82, 1), vec4.createColor(0.53, 0.62, 0.64, 1) },
			speed = 2.4,
			size = 0.1,
			gravity = 5,
			duration = 0.55,
		},
		steam = {
			colors = { vec4.createColor(0.89, 0.87, 0.82, 0.6), vec4.createColor(0.74, 0.73, 0.68, 0.5) },
			speed = 0.6,
			size = 0.16,
			gravity = -1.5,
			duration = 1.2,
		},
	}

--- Throw particles from a point. `kind` is a key of BURSTS.
function module.burst(pos: vec.Vec2, kind: string, count: number)
	local style = BURSTS[kind]
	for i = 1, count do
		local angle = math.random() * math.pi * 2
		local speed = style.speed * (0.4 + math.random() * 0.6)
		table.insert(module.particles, {
			pos = pos + vec.V2(math.random() - 0.5, math.random() - 0.5):scale(0.3),
			velocity = vec.V2(math.cos(angle), math.sin(angle) - 0.4):scale(speed),
			color = style.colors[(i % #style.colors) + 1],
			size = style.size * (0.6 + math.random() * 0.8),
			age = 0,
			duration = style.duration * (0.6 + math.random() * 0.6),
			gravity = style.gravity,
		})
	end
end

--- A stream of particles flowing from `from` to `to` (water drunk by the Sun Warden), a few more every call
function module.stream(from: vec.Vec2, to: vec.Vec2, colors: { vec4.Vec4 }, count: number)
	local delta = to - from
	local distance = delta:length()
	if distance < 0.01 then
		return
	end
	local speed = 7
	for i = 1, count do
		local start = from + vec.V2(math.random() - 0.5, math.random() - 0.5):scale(0.5)
		table.insert(module.particles, {
			pos = start,
			velocity = (to - start):normalized():scale(speed),
			color = colors[(i % #colors) + 1],
			size = 0.08 + math.random() * 0.05,
			age = 0,
			duration = distance / speed,
			gravity = 0,
			drag = 0,
		})
	end
end

function module.shake(amount: number)
	module.shakeAmount = math.max(module.shakeAmount, amount)
end

function module.flash(color: vec4.Vec4, amount: number)
	module.flashColor = color
	module.flashAmount = math.max(module.flashAmount, amount)
end

function module.update(dt: number)
	for i = #module.list, 1, -1 do
		local effect = module.list[i]
		effect.age += dt
		if effect.kind == "text" then
			effect.pos = effect.pos + vec.V2(0, -dt * 0.8)
		end
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
			p.velocity = p.velocity:scale(math.exp(-dt * (p.drag or 3))) + vec.V2(0, p.gravity * dt)
			p.pos = p.pos + p.velocity:scale(dt)
		end
	end
end

--- The shake and the flash of the screen fade out, even while a menu is open (the world is paused, not the screen)
function module.updateScreen(dt: number)
	module.shakeAmount = math.max(0, module.shakeAmount - dt * 1.4)
	module.flashAmount = math.max(0, module.flashAmount - dt * 2.5)
end

function module.clear()
	table.clear(module.list)
	table.clear(module.particles)
end

return module
