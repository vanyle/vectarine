--[[
Draws the world: the tilemap, the entities, the player, the particles and the other effects.

Drawing happens in "world space": 1 unit is 1 tile and y points down, like in the room maps.
`withWorldTransform` sets up a graphics transformation from world space to the screen, so the rest of
the code never has to think about screen coordinates.

Everything here is visual only: positions are smoothed (`drawPos`) and moves hop a little so that the
turn-based movements feel alive, but the game logic only knows about whole tiles.
]]
const effects = require("./effects")
const game_state = require("./game_state")
const graphics = require("@vectarine/graphics")
const loader = require("@vectarine/loader")
const monsters = require("./data/monsters")
const rooms = require("./data/rooms")
const rules = require("./rules")
const sprites = require("./sprites")
const terrain = require("./data/terrain")
const theme = require("./theme")
const types = require("./types")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")
const world = require("./world")

const module = loader.init() or {}

const ROOM_SIZE = vec.V2(rooms.ROOM_WIDTH, rooms.ROOM_HEIGHT)
--- How long the camera takes to scroll to the next room, in seconds
const SCROLL_DURATION = 0.45
--- How fast things slide to their new tile. Higher is faster.
const MOVE_SPEED = 16

--- A rectangle of the screen where the world is drawn, in OpenGL coordinates.
export type Viewport = {
	pos: vec.Vec2,
	size: vec.Vec2,
	--- Size of one tile on screen
	tileSize: vec.Vec2,
}

--- Colors used to tell gates apart: plate gates are plain, target gates are red.
const GATE_TINTS: { [string]: vec4.Vec4 } = {
	plates = vec4.WHITE,
	targets = vec4.createColor(1, 0.55, 0.5, 1),
	monsters = vec4.WHITE,
	levels = vec4.createColor(1, 0.85, 0.5, 1),
}
const EFFECT_COLORS: { [string]: vec4.Vec4 } = {
	leap = vec4.createColor(1, 0.4, 0.35, 1),
	slant = vec4.createColor(0.35, 0.65, 1, 1),
}

-- MARK: Coordinates

--- Run `draw` with a transformation where 1 unit = 1 tile, y points down and `camera` is the top-left of the viewport.
local function withWorldTransform(viewport: Viewport, camera: vec.Vec2, draw: () -> ())
	local top = viewport.pos.y + viewport.size.y
	-- The transformation computes: scale * (p + translation)
	graphics.withTransformation({
		scale = vec.V2(viewport.tileSize.x, -viewport.tileSize.y),
		translation = vec.V2(viewport.pos.x / viewport.tileSize.x - camera.x, -top / viewport.tileSize.y - camera.y),
	}, draw)
end

--- Convert a world position to the screen. Useful to draw text, which should not be flipped.
function module.toScreen(viewport: Viewport, camera: vec.Vec2, pos: vec.Vec2): vec.Vec2
	local top = viewport.pos.y + viewport.size.y
	return vec.V2(
		viewport.pos.x + (pos.x - camera.x) * viewport.tileSize.x,
		top - (pos.y - camera.y) * viewport.tileSize.y
	)
end

--- Fill a rectangle in world space. The y axis is flipped, so the rectangle starts from its bottom.
local function fillRect(pos: vec.Vec2, size: vec.Vec2, color: vec4.Vec4)
	graphics.drawRect(pos + vec.V2(0, size.y), vec.V2(size.x, -size.y), color)
end

-- MARK: Terrain

--- A number that looks random but is always the same for a given tile, to pick sprite variants.
local function tileHash(x: number, y: number): number
	local h = bit32.bxor(x * 374761393, y * 668265263)
	h = bit32.bxor(h, bit32.rshift(h, 13))
	return h
end

--- The terrain ids that count as "the same" as `t` for autotiling (itself and the ones it `connects` to).
--- Keyed by terrain so that editing terrain.luau doesn't keep stale ids.
local connectedIds: { [types.Terrain]: { [number]: boolean } } = {}
local function connectedTo(t: types.Terrain): { [number]: boolean }
	local ids = connectedIds[t]
	if not ids then
		ids = { [t.id] = true }
		for _, name in (t.autotile and t.autotile.connects) or {} do
			ids[terrain.byName[name].id] = true
		end
		connectedIds[t] = ids
	end
	return ids
end

--- Pick the sprite of a terrain tile, looking at its neighbors when the terrain is autotiled.
local function terrainSprite(level: types.World, layer: number, t: types.Terrain, x: number, y: number): string?
	local autotile = t.autotile
	if not autotile then
		return if t.variants then t.variants[tileHash(x, y) % #t.variants + 1] else t.sprite
	end
	local ids = connectedTo(t)
	local function same(dx: number, dy: number): boolean
		return ids[level.tilemap:getTile(layer, x + dx, y + dy)] == true
	end
	local up, right, down, left = same(0, -1), same(1, 0), same(0, 1), same(-1, 0)
	if autotile.row then
		local index = if left and right then 3 elseif right then 2 elseif left then 4 else 1
		return autotile.row[index]
	elseif autotile.nine then
		local inner = autotile.inner
		if inner and up and down and left and right then
			-- Inside corner: all 4 neighbors are the same, but a diagonal one isn't
			if not same(-1, -1) then
				return inner[1]
			elseif not same(1, -1) then
				return inner[2]
			elseif not same(1, 1) then
				return inner[3]
			elseif not same(-1, 1) then
				return inner[4]
			end
		end
		local row = if not up then 0 elseif not down then 2 else 1
		local column = if not left then 0 elseif not right then 2 else 1
		return autotile.nine[row * 3 + column + 1]
	end
	local mask = (if up then 1 else 0) + (if right then 2 else 0) + (if down then 4 else 0) + (if left then 8 else 0)
	return (autotile.mask or {})[mask] or autotile.fallback
end

local function drawTerrainLayer(game: types.Game, layer: number, low: vec.Vec2, high: vec.Vec2)
	local level = game.world
	level.tilemap:getTilemapPart(layer, low.x, low.y, high.x, high.y, function(tileId: number, x: number, y: number)
		local t = terrain.byId[tileId]
		if not t then
			return
		end
		local sprite = terrainSprite(level, layer, t, x, y)
		if t.plate and game_state.isPressed(game, vec.V2(x, y)) then
			sprite = "extra/plate_down"
		end
		if sprite then
			sprites.drawInWorld(sprite, vec.V2(x, y), { tint = t.tint })
		end
	end)
end

-- MARK: Animation

--- Visual state that is only needed for drawing, stored aside so that it never ends up in saved levels.
type Animation = { lastPos: vec.Vec2, moveLength: number, open: number }
module.animations = setmetatable({} :: { [any]: Animation }, { __mode = "k" })

--- The turn clock, smoothed. Ambient particles use it so that the world drifts a bit after each step.
module.displayedTurn = module.displayedTurn or 0

local function animationOf(thing: any): Animation
	local animation = module.animations[thing]
	if not animation then
		animation = { lastPos = thing.pos, moveLength = 1, open = -1 }
		module.animations[thing] = animation
	end
	return animation
end

--- Smoothly move the drawn position towards the logical position and decay the bumps.
local function animate(thing: { pos: vec.Vec2, drawPos: vec.Vec2, bump: vec.Vec2 }, dt: number)
	local animation = animationOf(thing)
	if animation.lastPos.x ~= thing.pos.x or animation.lastPos.y ~= thing.pos.y then
		animation.lastPos = thing.pos
		animation.moveLength = math.max(0.01, (thing.pos - thing.drawPos):length())
	end
	if (thing.pos - thing.drawPos):length() > 4 then
		thing.drawPos = thing.pos -- Teleported: no need to animate
	else
		thing.drawPos = thing.drawPos:lerp(thing.pos, 1 - math.exp(-dt * MOVE_SPEED))
	end
	thing.bump = thing.bump:scale(math.exp(-dt * 14))
end

--- How high something hops while moving between two tiles (0 when not moving)
local function hopHeight(thing: { pos: vec.Vec2, drawPos: vec.Vec2 }): number
	local animation = animationOf(thing)
	local remaining = math.clamp((thing.pos - thing.drawPos):length() / animation.moveLength, 0, 1)
	return math.sin(remaining * math.pi) * 0.22 * math.min(animation.moveLength, 2)
end

--- Returns true while the player is moving on screen. Player input waits for animations to finish.
function module.isAnimating(game: types.Game): boolean
	return (game.player.pos - game.player.drawPos):length() > 0.15 or module.camera.scroll ~= nil
end

--- Update the smooth positions of everything that moves.
function module.animate(game: types.Game, dt: number)
	animate(game.player, dt)
	for _, entity in game.entities[game.currentRoom] do
		animate(entity, dt)
		if entity.kind == "gate" then
			local animation = animationOf(entity)
			local target = if game_state.isGateOpen(game, entity) then 1 else 0
			if animation.open < 0 then
				animation.open = target
			end
			animation.open += (target - animation.open) * (1 - math.exp(-dt * 14))
		end
	end
	module.displayedTurn += (effects.turnClock - module.displayedTurn) * (1 - math.exp(-dt * 10))
end

-- MARK: Entities

export type Label = { pos: vec.Vec2, text: string, color: vec4.Vec4, size: number }

local function drawGate(game: types.Game, gate: types.Entity)
	local open = math.clamp(animationOf(gate).open, 0, 1)
	if gate.spikes then
		sprites.drawInWorld("extra/spikes_down", gate.pos)
		if open < 0.99 then
			-- Spikes rise from the floor
			sprites.drawInWorld("extra/spikes", gate.pos + vec.V2(0, open * 0.5), { scale = 1 - open * 0.6 })
		end
		return
	end
	sprites.drawInWorld("extra/gate_open", gate.pos)
	if open < 0.99 then
		local tint = GATE_TINTS[gate.condition or "plates"]
		sprites.drawInWorld(gate.sprite, gate.pos + vec.V2(0, open * 0.45), { tint = tint, scale = 1 - open * 0.8 })
	end
end

local function drawMonster(entity: types.Entity, time: number)
	local definition = monsters.monsters[entity.monster or ""]
	local pos = entity.drawPos + entity.bump - vec.V2(0, hopHeight(entity))
	if definition.flying then
		pos += vec.V2(0, math.sin(time * 8 + entity.pos.x) * 0.06 - 0.08)
	end
	local tint = if entity.effect then EFFECT_COLORS[entity.effect]:lerp(vec4.WHITE, 0.45) else nil
	sprites.drawInWorld(entity.sprite, pos, { flipX = entity.facingLeft, tint = tint })
	if entity.effect then
		-- A small potion above the monster shows that it moves differently now
		local potion = if entity.effect == "leap" then "dungeon/potion_red" else "dungeon/potion_blue"
		sprites.drawInWorld(potion, pos + vec.V2(0.3, -0.45), { scale = 0.45 })
	end
end

local function drawEntity(game: types.Game, entity: types.Entity, time: number, labels: { Label })
	local kind = entity.kind
	local pos = entity.drawPos + entity.bump
	if kind == "monster" then
		drawMonster(entity, time)
	elseif kind == "gate" then
		drawGate(game, entity)
	elseif kind == "pickup" then
		local bob = math.sin(time * 3 + entity.pos.x * 1.7) * 0.06
		if entity.pickup == "gear" then
			-- The gear is the goal of the level: make it pulse
			local pulse = 1 + math.sin(time * 4) * 0.08
			sprites.drawInWorld("extra/gear", pos + vec.V2(0, bob - 0.05), { scale = pulse * 0.9 })
		else
			sprites.drawInWorld(entity.sprite, pos + vec.V2(0, bob - 0.05), { scale = 0.8 })
		end
	elseif kind == "bomb" then
		local fuse = entity.fuse or 0
		local wobble = math.sin(time * (10 + (4 - fuse) * 6)) * 0.04
		local blink = math.floor(time * (3 + (4 - fuse) * 3)) % 2 == 0
		local tint = if blink then vec4.createColor(1, 0.55, 0.5, 1) else nil
		sprites.drawInWorld(entity.sprite, pos + vec.V2(wobble, 0), { tint = tint, scale = 0.9 + (3 - fuse) * 0.05 })
		table.insert(labels, { pos = pos + vec.V2(0.75, 0.1), text = `{fuse}`, color = theme.colors.danger, size = 0.55 })
	elseif kind == "target" then
		sprites.drawInWorld(entity.sprite, pos)
		if entity.hit then
			sprites.drawInWorld("extra/arrow_fly", pos + vec.V2(0.1, -0.1), { quarterTurns = 1, scale = 0.8 })
		end
	elseif kind == "ladder" then
		local finished = entity.level and game.finished[entity.level]
		sprites.drawInWorld(entity.sprite, pos, { tint = if finished then theme.colors.gold else nil })
		table.insert(labels, {
			pos = pos + vec.V2(0.5, -0.1),
			text = `{entity.level}`,
			color = if finished then theme.colors.gold else theme.colors.text,
			size = 0.45,
		})
	elseif kind == "npc" then
		local breathe = math.sin(time * 2 + entity.pos.x) * 0.02
		sprites.drawInWorld(entity.sprite, pos + vec.V2(0, -breathe), { scale = 1 + breathe })
	else
		sprites.drawInWorld(entity.sprite, pos)
	end
end

local function drawPlayer(game: types.Game, time: number)
	local player = game.player
	if player.dead then
		return
	end
	if player.facing == "left" then
		module.playerFacingLeft = true
	elseif player.facing == "right" then
		module.playerFacingLeft = false
	end
	local hop = hopHeight(player)
	-- Squash and stretch: bigger in the air, and a slow "breathing" when idle
	local scale = 1 + hop * 0.25 + math.sin(time * 2.5) * 0.015
	local pos = player.drawPos + player.bump - vec.V2(0, hop)
	local tint = if player.effect then EFFECT_COLORS[player.effect]:lerp(vec4.WHITE, 0.55) else nil
	sprites.drawInWorld("dungeon/hero", pos, { flipX = module.playerFacingLeft, scale = scale, tint = tint })
end

--- Brackets on the tile in front of the player: they show where items will be used.
--- With a potion, colored markers show where each arrow key would take the player.
local function drawCursor(game: types.Game, time: number)
	local player = game.player
	if player.dead then
		return
	end
	local facing = player.pos + types.directions[player.facing]
	sprites.drawInWorld("dungeon/cursor", facing, { tint = theme.withAlpha(vec4.WHITE, 0.22) })
	if player.effect then
		local color = EFFECT_COLORS[player.effect]
		for _, dir in types.directionList do
			local offset = rules.moveFor(player.effect, dir)
			local alpha = 0.35 + math.sin(time * 5) * 0.1
			sprites.drawInWorld("dungeon/cursor", player.pos + offset, { tint = theme.withAlpha(color, alpha), scale = 0.7 })
		end
	end
end

-- MARK: Effects

local function drawParticles()
	for _, p in effects.particles do
		local alpha = 1 - p.age / p.duration
		local half = p.size * 0.5
		fillRect(p.pos - vec.V2(half, half), vec.V2(p.size, p.size), theme.withAlpha(p.color, alpha))
	end
end

local function drawEffects(labels: { Label })
	for _, effect in effects.list do
		local progress = effect.age / effect.duration
		if effect.kind == "sprite" and effect.sprite then
			sprites.drawInWorld(effect.sprite, effect.pos, {
				quarterTurns = effect.quarterTurns,
				tint = theme.withAlpha(vec4.WHITE, 1 - progress * progress),
				scale = 0.8 + progress * 0.2,
			})
		elseif effect.kind == "arrow" and effect.to then
			local pos = effect.pos:lerp(effect.to, math.min(1, progress * 1.1))
			sprites.drawInWorld("extra/arrow_fly", pos, { quarterTurns = effect.quarterTurns })
		elseif effect.kind == "dying" and effect.sprite then
			-- Jump up, spin and shrink
			local lift = math.sin(progress * math.pi) * 0.4
			sprites.drawInWorld(effect.sprite, effect.pos - vec.V2(0, lift), {
				scale = 1 - progress,
				quarterTurns = math.floor(progress * 8) % 4,
				flipX = effect.flipX,
				tint = vec4.createColor(1, 1 - progress, 1 - progress, 1),
			})
		elseif effect.kind == "ring" then
			-- Explosion shockwave: an expanding square outline
			local size = 1 + progress * 2.5
			local center = effect.pos + vec.V2(0.5, 0.5)
			local color = vec4.createColor(1, 0.9, 0.6, (1 - progress) * 0.8)
			local thickness = 0.12 * (1 - progress) + 0.02
			local corner = center - vec.V2(size / 2, size / 2)
			fillRect(corner, vec.V2(size, thickness), color)
			fillRect(corner + vec.V2(0, size - thickness), vec.V2(size, thickness), color)
			fillRect(corner, vec.V2(thickness, size), color)
			fillRect(corner + vec.V2(size - thickness, 0), vec.V2(thickness, size), color)
		elseif effect.kind == "text" and effect.text then
			table.insert(labels, {
				pos = effect.pos + vec.V2(0.5, -0.1 - progress * 0.9),
				text = effect.text,
				color = theme.withAlpha(effect.color or vec4.WHITE, math.min(1, (1 - progress) * 2.5)),
				size = 0.5,
			})
		end
	end
end

--- Leaves (outside) or dust (in the mine) floating in the air. They only drift when turns pass,
--- which shows that the world moves only when the player does.
local function drawAmbient(game: types.Game, low: vec.Vec2, high: vec.Vec2)
	local outdoors = world.roomData(game.currentRoom).floor == "grass"
	local turn = module.displayedTurn
	local span = high - low
	for i = 1, 14 do
		local seedX = (i * 0.6180339) % 1
		local seedY = (i * 0.4142135) % 1
		local x = low.x + ((seedX * span.x + turn * (0.13 + seedY * 0.1)) % span.x)
		local y = low.y + ((seedY * span.y + turn * (0.07 + seedX * 0.05) + math.sin(turn * 0.5 + i) * 0.3) % span.y)
		if outdoors then
			sprites.drawInWorld("extra/leaf", vec.V2(x, y), {
				scale = 0.55,
				quarterTurns = math.floor(turn * 0.5 + i) % 4,
				tint = theme.withAlpha(vec4.WHITE, 0.8),
			})
		else
			fillRect(vec.V2(x, y), vec.V2(0.06, 0.06), vec4.createColor(1, 0.9, 0.7, 0.35))
		end
	end
end

local function drawLabels(viewport: Viewport, camera: vec.Vec2, labels: { Label })
	for _, label in labels do
		local size = label.size * viewport.tileSize.y
		local width = theme.font:measureText(label.text, size).width
		local center = module.toScreen(viewport, camera, label.pos)
		-- Keep labels inside the viewport so that they can always be read
		local minX = viewport.pos.x + 0.01
		local maxX = viewport.pos.x + viewport.size.x - width - 0.01
		local bottomLeft = vec.V2(
			math.clamp(center.x - width / 2, minX, math.max(minX, maxX)),
			math.clamp(center.y - size / 2, viewport.pos.y + 0.01, viewport.pos.y + viewport.size.y - size)
		)
		theme.font:drawText(label.text, bottomLeft + vec.V2(0.004, -0.006), size, theme.withAlpha(vec4.BLACK, label.color.a))
		theme.font:drawText(label.text, bottomLeft, size, label.color)
	end
end

-- MARK: Main drawing function

export type DrawOptions = {
	time: number,
	showPlayer: boolean,
}

--- Draw the world as seen from `camera` (the world position of the top-left corner of the viewport).
function module.drawWorld(game: types.Game, viewport: Viewport, camera: vec.Vec2, options: DrawOptions)
	local labels: { Label } = {}
	local visibleTiles = viewport.size / viewport.tileSize
	local low = camera:floor()
	local high = (camera + visibleTiles):ceil()

	-- The rooms on screen: the current one, and the next one while the camera scrolls
	local visibleRooms = { game.currentRoom }
	for _, corner in { low, high - vec.V2(1, 1) } do
		local room = world.roomAt(game.world, corner)
		if room and not table.find(visibleRooms, room.id) then
			table.insert(visibleRooms, room.id)
		end
	end

	-- Everything is clipped to the viewport, so that rooms don't spill over the HUD while scrolling.
	graphics.drawWithMask(function()
		withWorldTransform(viewport, camera, function()
			drawTerrainLayer(game, world.GROUND, low, high)
			drawTerrainLayer(game, world.OBJECTS, low, high)
			-- Gates and ladders are on the floor, so they are drawn first
			for pass = 1, 2 do
				for _, roomId in visibleRooms do
					for _, entity in game.entities[roomId] do
						local isFloor = entity.kind == "gate" or entity.kind == "ladder"
						if (pass == 1) == isFloor and game_state.isPresent(game, entity) then
							drawEntity(game, entity, options.time, labels)
						end
					end
				end
			end
			if options.showPlayer then
				drawCursor(game, options.time)
				drawPlayer(game, options.time)
			end
			drawEffects(labels)
			drawParticles()
			drawAmbient(game, low, high)
		end)
		drawLabels(viewport, camera, labels)
	end, function()
		graphics.drawRect(viewport.pos, viewport.size)
	end)
end

-- MARK: Camera

--- The camera shows one room at a time and scrolls when the player walks to another room.
module.camera = module.camera
	or {
		pos = vec.V2(0, 0),
		room = "",
		scroll = nil :: { from: vec.Vec2, to: vec.Vec2, t: number }?,
	}

--- Place the camera on the current room without scrolling.
function module.resetCamera(game: types.Game)
	module.camera.pos = game.world.rooms[game.currentRoom].origin
	module.camera.room = game.currentRoom
	module.camera.scroll = nil
end

function module.updateCamera(game: types.Game, dt: number)
	local camera = module.camera
	if camera.room ~= game.currentRoom then
		local target = game.world.rooms[game.currentRoom].origin
		local distance = target - camera.pos
		-- Scroll to neighboring rooms, but jump directly when changing to a far away room (a level)
		local isNeighbor = math.abs(distance.x) <= ROOM_SIZE.x and math.abs(distance.y) <= ROOM_SIZE.y
		if isNeighbor then
			camera.scroll = { from = camera.pos, to = target, t = 0 }
		else
			camera.pos = target
			camera.scroll = nil
		end
		camera.room = game.currentRoom
	end

	local scroll = camera.scroll
	if scroll then
		scroll.t = math.min(1, scroll.t + dt / SCROLL_DURATION)
		local eased = scroll.t * scroll.t * (3 - 2 * scroll.t) -- smoothstep
		camera.pos = scroll.from:lerp(scroll.to, eased)
		if scroll.t >= 1 then
			camera.scroll = nil
		end
	end
end

return module
