--[[
Draws the world: terrain, props, characters, effects and darkness.

The camera is slightly tilted (like Stardew Valley): walls show their front face, and things lower on the screen
are drawn in front of things higher up. So the world is drawn in passes:
1. the terrain: ground, walls and their shadows (see terrain_view.luau)
2. flat things on the ground (ferns, teleporters), the shadows of everything standing, items to pick up
3. everything that stands up (props, characters, creatures, projectiles), sorted from top to bottom
4. effects, then darkness (a shader, see lighting.luau), then what shines through it (mending beams, sand)

World coordinates: 1 unit = 1 tile, y points down. `Viewport` says where the world is drawn on the screen.
]]
const blocks = require("./blocks")
const dream = require("./dream")
const effects = require("./effects")
const frost = require("./frost")
const graphics = require("@vectarine/graphics")
const interactions = require("./interactions")
const inventory = require("./inventory")
const items = require("./data/items")
const keyboard_icons = require("./keyboard_icons")
const level = require("./level")
const levels = require("./data/levels")
const lighting = require("./lighting")
const loader = require("@vectarine/loader")
const monsters = require("./data/monsters")
const npcs = require("./npcs")
const prompts = require("./prompts")
const props = require("./data/props")
const sprites = require("./sprites")
const monster_ai = require("./monster_ai")
const storm = require("./storm")
const swamp = require("./swamp")
const terrain_view = require("./terrain_view")
const theme = require("./theme")
const types = require("./types")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

const module = loader.init() or {}

export type Viewport = {
	--- Bottom-left corner of the area, in OpenGL coordinates
	pos: vec.Vec2,
	size: vec.Vec2,
	--- Size of a tile, in OpenGL units
	tileSize: vec.Vec2,
}

--- Top-left corner of the visible part of the world, in tiles
module.camera = module.camera or vec.V2(0, 0)

const WHITE_FLASH = vec4.createColor(3, 3, 3, 1)
--- Duration of the weapon swing animation, in seconds
const SWING_DURATION = 0.16
const FACING_ANGLES: { [types.Direction]: number } = { right = 0, down = math.pi / 2, left = math.pi, up = -math.pi / 2 }
const SHADOW = theme.withAlpha(theme.colors.darkness, 0.35)
const MUD = theme.hex("323230")
--- Things of the dream (phantoms, dream bridges) are pale and see-through
const DREAM_TINT = vec4.createColor(0.85, 0.95, 1.15, 0.72)

-- MARK: Camera

function module.visibleTiles(viewport: Viewport): vec.Vec2
	return vec.V2(viewport.size.x / viewport.tileSize.x, viewport.size.y / viewport.tileSize.y)
end

--- Where the camera wants to be: centered on the player, without showing outside of the level.
local function cameraTarget(game: types.Game, viewport: Viewport): vec.Vec2
	local visible = module.visibleTiles(viewport)
	local lvl = game.level
	local function axis(center: number, size: number, worldSize: number): number
		if worldSize <= size then
			return (worldSize - size) / 2
		end
		return math.clamp(center - size / 2, 0, worldSize - size)
	end
	local focus = game.player.pos + vec.V2(0, -0.5)
	return vec.V2(axis(focus.x, visible.x, lvl.width), axis(focus.y, visible.y, lvl.height))
end

function module.resetCamera(game: types.Game, viewport: Viewport)
	module.camera = cameraTarget(game, viewport)
end

function module.updateCamera(game: types.Game, viewport: Viewport, dt: number)
	module.camera = module.camera:lerp(cameraTarget(game, viewport), math.min(1, dt * 8))
end

--- Convert a world position to the screen (OpenGL coordinates)
function module.toScreen(viewport: Viewport, pos: vec.Vec2): vec.Vec2
	local relative = pos - module.camera
	return vec.V2(viewport.pos.x + relative.x * viewport.tileSize.x, viewport.pos.y + viewport.size.y - relative.y * viewport.tileSize.y)
end

-- MARK: Things

type Drawable = { y: number, draw: () -> () }

local function propSprite(game: types.Game, prop: types.Prop, kind: props.PropKind, time: number): string?
	local name: string = prop.kind
	if name == "chest" and interactions.isChestOpen(game, prop) then
		return "world/chest_open"
	elseif kind.fruit and not interactions.hasFruit(game, prop) then
		return kind.fruit.empty
	elseif name == "cistern" and interactions.isCisternFull(game, prop) then
		return "world/cistern_full"
	elseif name == "teleporter" and interactions.isTeleporterOn(game, prop) then
		return `world/teleporter_on_{math.floor(time * 4) % 2}`
	elseif name == "npc" then
		local npc = npcs.npcs[prop.data.npc]
		return `characters/{npc.sprite}_{math.floor(time * 1.6 + prop.cell.x) % 2}`
	elseif kind.frames then
		return kind.frames[math.floor(time * (kind.fps or 4)) % #kind.frames + 1]
	end
	return kind.sprite
end

--- The shadow of something standing at `pos`, `width` tiles wide
type Shadow = { pos: vec.Vec2, width: number, alpha: number? }

local function drawShadow(shadow: Shadow)
	local color = if shadow.alpha then theme.withAlpha(theme.colors.darkness, shadow.alpha) else SHADOW
	graphics.drawEllipse(shadow.pos, vec.V2(shadow.width, shadow.width * 0.35), color)
end

const MENDING = theme.hex("73804b")
const MENDING_CORE = theme.hex("e3ddd1")

--- The beams of the menders to the creatures they mend: a pulsing green glow with a bright core, and crosses
--- flowing along it. Drawn above the darkness: the mending is a light of its own, seen from afar.
local function drawMendingBeams(game: types.Game, time: number)
	for _, monster in game.monsters do
		local patient = (monster :: any).patient :: types.Monster?
		if monster.state == "heal" and patient and not patient.dead and monsters.species[monster.kind].behavior == "mender" then
			local from, to = monster.pos + vec.V2(0, -0.6), patient.pos + vec.V2(0, -0.5)
			local pulse = 0.75 + math.sin(time * 12) * 0.25
			graphics.drawLine(from, to, theme.withAlpha(MENDING, 0.35 * pulse), 0.3)
			graphics.drawLine(from, to, theme.withAlpha(MENDING, 0.8), 0.1)
			graphics.drawLine(from, to, theme.withAlpha(MENDING_CORE, pulse), 0.035)
			for i = 0, 3 do
				local t = (time * 1.4 + i / 4) % 1
				sprites.drawInWorld("characters/heal_plus", from:lerp(to, t) - vec.V2(0.35, 0.35), { size = 0.7 })
			end
		end
	end
end

--- The tongue of the Frog King: pieces from its mouth to the tip
local function drawTongue(base: vec.Vec2, tip: vec.Vec2)
	local delta = tip - base
	local length = delta:length()
	if length < 0.05 then
		return
	end
	local angle = delta:angle()
	local pieces = math.ceil(length / 0.8)
	for i = 1, pieces do
		local center = base + delta:scale((i - 0.5) / pieces)
		local sprite = if i == pieces then "characters/tongue_tip" else "characters/tongue"
		sprites.drawInWorld(sprite, center - vec.V2(0.5, 0.5), { angle = angle })
	end
end

--- Hazards of the marsh: a drop of ooze that grows on the wall and falls (its shadow grows on the ground), then its
--- splash; a puddle of poison
local function drawHazard(monster: types.Monster, species: monsters.Species, flat: { any }, shadows: { Shadow })
	local t = monster.stateTime
	local state = monster.state
	if species.behavior == "puddle" then
		table.insert(flat, { y = monster.pos.y, draw = function()
			sprites.drawInWorld("characters/ooze_1", monster.pos - vec.V2(0.7, 0.9), { size = 1.4 })
		end })
		return
	end
	if state == "form" or state == "fall" then
		local grow = if state == "form" then math.min(1, t / 1.4) else 1
		table.insert(shadows, { pos = monster.pos, width = 0.12 + 0.25 * grow, alpha = 0.25 + 0.3 * grow })
		local height = if state == "fall" then 1.4 * (1 - math.min(1, t / 0.3)) else 1.4
		table.insert(flat, { y = monster.pos.y, draw = function()
			sprites.drawInWorld("characters/ooze_0", monster.pos + vec.V2(-0.5 * grow, -0.4 - height - 0.5 * grow), { size = grow })
		end })
	elseif state == "splash" then
		table.insert(flat, { y = monster.pos.y, draw = function()
			sprites.drawInWorld("characters/ooze_1", monster.pos - vec.V2(0.5, 0.75))
		end })
	end
end

local function playerSprite(player: types.Player): (string, boolean)
	local view = if player.facing == "up" then "up" elseif player.facing == "down" then "down" else "side"
	local frame = 0
	if player.moving then
		frame = 1 + math.floor(player.walkTime * 7) % 2
	end
	return `characters/player_{view}_{frame}`, player.facing == "left"
end

local function monsterSprite(monster: types.Monster, species: monsters.Species, time: number): string
	local state = monster.state
	if species.behavior == "burrower" and (state == "hidden" or state == "rising") then
		return `characters/{species.mound or "mound"}_0`
	end
	if species.boss then
		local special = species.stateSprites and species.stateSprites[state]
		if special then
			return `bosses/{special}`
		end
		return `bosses/{species.sprite}_{math.floor(time * 3) % 2}`
	end
	-- Sentinels: their lens glows when they aim. Menders glow while they mend. Frogs stretch in the air.
	if species.behavior == "turret" then
		return `characters/{species.sprite}_{if state == "aim" then 1 else 0}`
	elseif species.behavior == "mender" then
		return `characters/{species.sprite}_{if state == "heal" or state == "aim" then 1 else 0}`
	elseif species.behavior == "hopper" then
		return `characters/{species.sprite}_{if state == "jump" then 2 elseif state == "crouch" then 1 else 0}`
	elseif species.behavior == "drip" or species.behavior == "puddle" then
		return `characters/{species.sprite}_{if state == "form" or state == "fall" then 0 else 1}`
	end
	local speed = if state == "coil" or state == "aim" or state == "crouch" then 12 else 5
	return `characters/{species.sprite}_{math.floor(time * speed + monster.id) % 2}`
end

--- What to draw, by pass: things flat on the ground, shadows, items to pick up, then everything standing.
--- Shadows are drawn together, so that drawing them doesn't break the batches of sprites.
type Drawables = { flat: { Drawable }, shadows: { Shadow }, pickups: { Drawable }, standing: { Drawable } }

local function collectDrawables(game: types.Game, low: vec.Vec2, high: vec.Vec2, time: number): Drawables
	local flat: { Drawable } = {}
	local standing: { Drawable } = {}
	local pickups: { Drawable } = {}
	local shadows: { Shadow } = {}
	local function visible(pos: vec.Vec2, margin: number): boolean
		return pos.x >= low.x - margin and pos.x <= high.x + margin and pos.y >= low.y - margin and pos.y <= high.y + margin
	end

	for _, prop in game.level.props do
		if visible(prop.cell, 2) and not game.flags[`broken:{prop.key}`] and interactions.isThere(game, prop) then
			local kind = props.kinds[prop.kind]
			local sprite = propSprite(game, prop, kind, time)
			if sprite then
				-- Ice blocks slide between two cells
				local cell = if kind.pushable then blocks.drawCell(prop) else prop.cell
				local tint = if kind.dream or (prop.kind == "npc" and npcs.npcs[prop.data.npc].dream) then DREAM_TINT else nil
				local entry: Drawable = {
					-- A snowdrift hides whoever walks behind it
					y = cell.y + (if kind.hides then 1.6 else 0.9),
					draw = function()
						sprites.drawInWorld(sprite, cell, { tint = tint })
						if kind.tall then
							sprites.drawInWorld(kind.tall, cell + vec.V2(0, -1))
						end
						if kind.wide then
							sprites.drawInWorld(kind.wide, cell + vec.V2(1, 0))
						end
					end,
				}
				table.insert(if kind.flat then flat else standing, entry)
			end
		end
	end

	for _, pickup in game.pickups do
		local bob = math.sin(time * 4 + pickup.pos.x * 2) * 0.06
		local icon = items.items[pickup.item].icon
		table.insert(shadows, { pos = pickup.pos + vec.V2(0, -0.05), width = 0.25 })
		table.insert(pickups, {
			y = pickup.pos.y,
			draw = function()
				sprites.drawInWorld(icon, pickup.pos + vec.V2(-0.35, -0.75 + bob), { size = 0.7 })
			end,
		})
	end

	local dreaming = dream.isDreaming(game)
	for _, monster in game.monsters do
		local species = monsters.species[monster.kind]
		-- Phantoms are only there for dreamers. Targets of the gallery are behind their wall, or knocked down.
		local hiddenTarget = species.prize ~= nil and (monster.state == "hidden" or monster.state == "down")
		if (species.dream and not dreaming) or hiddenTarget then
			continue
		end
		if visible(monster.pos, 3) and (species.behavior == "drip" or species.behavior == "puddle") then
			drawHazard(monster, species, flat, shadows)
		elseif visible(monster.pos, 3) then
			local size = species.size or 1
			local sprite = monsterSprite(monster, species, time)
			local telegraph = monster.state == "rising" or monster.state == "coil" or monster.state == "aim" or monster.state == "crouch"
				or monster.state == "raise" or monster.state == "charge" or monster.state == "drain" or monster.state == "flooded"
			local shake = if telegraph then math.sin(time * 60) * 0.04 else 0
			local tint = if monster.hitTime < 0.12 then WHITE_FLASH elseif species.dream then DREAM_TINT else nil
			-- Snowmen of the gallery rise from behind their wall
			local shown: number? = nil
			if species.behavior == "popup" then
				local t = math.min(1, monster.stateTime / 0.25)
				shown = if monster.state == "rise" then t elseif monster.state == "sink" then 1 - t else nil
			end
			local lift = (if species.flying then 0.35 + math.sin(time * 6 + monster.id) * 0.08 else 0) + monster_ai.lift(monster, species)
			-- The shadow of a leaping frog stays on the ground: it shows where it will land
			local shadow: Shadow = { pos = monster.pos + vec.V2(0, -0.02), width = species.radius }
			if lift > 3 then
				local landing = monster.state == "lock" or monster.state == "fall"
				shadow.width = species.radius * (if landing then 1.6 + math.sin(time * 20) * 0.1 else 1.2)
				shadow.alpha = if landing then 0.7 else 0.45
			end
			table.insert(shadows, shadow)
			local base, tip = monster_ai.tongue(monster)
			if base and tip then
				table.insert(standing, { y = monster.pos.y + 0.02, draw = function()
					drawTongue(base, tip)
				end })
			end
			if lift < 7 then
				table.insert(standing, {
					y = monster.pos.y,
					draw = function()
						local sunk = if shown then (1 - shown) * size else 0
						sprites.drawInWorld(
							sprite,
							monster.pos + vec.V2(-size / 2 + shake, 0.08 - size - lift + sunk),
							{ size = size, flipX = monster.facing == "left", tint = tint, visible = shown }
						)
					end,
				})
			end
		end
	end

	local player = game.player
	local blink = player.invincible > 0 and math.floor(player.invincible * 14) % 2 == 0
	if not blink then
		local sprite, flip = playerSprite(player)
		local sailing = swamp.isSailing(game)
		-- Sinking in a bog: the player goes down into the mud
		local sink = swamp.sinking() * 0.45
		if not sailing then
			table.insert(shadows, { pos = player.pos + vec.V2(0, -0.02), width = 0.32 })
		end
		table.insert(standing, {
			y = player.pos.y,
			draw = function()
				if sailing then
					sprites.drawInWorld("characters/boat_0", player.pos + vec.V2(-0.5, -0.75))
				end
				-- In the mud, the legs disappear below its surface
				local visible = if sink > 0 then 1 - sink else nil
				sprites.drawInWorld(sprite, player.pos + vec.V2(-0.5, 0.08 - 1 + sink - (if sailing then 0.15 else 0)), { flipX = flip, visible = visible })
				if sink > 0.05 then
					graphics.drawEllipse(player.pos + vec.V2(0, 0.02), vec.V2(0.42, 0.16), MUD)
				end
			end,
		})
		-- The weapon sweeps in front of the player during a melee attack
		local weaponId = inventory.weapon(player)
		local weapon = items.items[weaponId]
		local swing = player.swingTime / SWING_DURATION
		if swing < 1 and not weapon.shot and not weapon.bomb and weaponId ~= "fist" then
			local angle = FACING_ANGLES[player.facing] - 1.1 + swing * 2.2
			local center = player.pos + vec.V2(0, -0.4) + vec.fromAngle(angle, 0.55)
			table.insert(standing, {
				-- In front of the player, except when attacking upwards
				y = player.pos.y + (if player.facing == "up" then -0.01 else 0.01),
				draw = function()
					sprites.drawInWorld(weapon.icon, center - vec.V2(0.35, 0.35), { size = 0.7, angle = angle + math.pi / 4 })
				end,
			})
		end
	end

	-- Lit bombs: the fuse sparks faster as it burns
	for _, bomb in game.bombs do
		local frame = math.floor(time * (if bomb.fuse < 0.6 then 16 else 6)) % 2
		table.insert(standing, {
			y = bomb.pos.y,
			draw = function()
				sprites.drawInWorld(`world/bomb_{frame}`, bomb.pos + vec.V2(-0.5, -0.85))
			end,
		})
	end

	for _, projectile in game.projectiles do
		local angle = projectile.velocity:angle()
		table.insert(standing, {
			y = projectile.pos.y + 0.35,
			draw = function()
				sprites.drawInWorld(projectile.sprite, projectile.pos + vec.V2(-0.5, -0.5), { angle = angle })
			end,
		})
	end
	return { flat = flat, shadows = shadows, pickups = pickups, standing = standing }
end

-- MARK: Effects

const SUNLIGHT = theme.hex("e3ddd1")

--- A beam of daylight falling from the ceiling, with dust floating in it
local function drawSunbeam(game: types.Game, time: number)
	local data = levels.levels[game.level.id]
	local beam = data.sunbeam
	if not beam then
		return
	end
	local center = vec.V2(beam[1] + 0.5, beam[2] + 0.5)
	-- The beam: vertical bands, brighter in the middle, fading towards the ceiling
	for i = 0, 11 do
		local y = center.y - i
		local alpha = 0.07 * (1 - i / 12)
		for _, band in { { width = 3.2, alpha = 0.6 }, { width = 2, alpha = 1 } } do
			graphics.drawRect(vec.V2(center.x - band.width / 2, y - 1), vec.V2(band.width, 1), theme.withAlpha(SUNLIGHT, alpha * band.alpha))
		end
	end
	graphics.drawEllipse(center + vec.V2(0, 0.3), vec.V2(2.4, 1.1), theme.withAlpha(SUNLIGHT, 0.08))
	-- Dust: specks drifting slowly down through the light
	for i = 1, 18 do
		local seed = i * 12.9898
		local x = center.x + (math.sin(seed) * 1.4) + math.sin(time * 0.7 + seed) * 0.15
		local fall = (time * 0.25 + (seed % 1)) % 1
		local y = center.y - 9 + fall * 9.5
		local alpha = 0.5 * math.sin(fall * math.pi)
		graphics.drawRect(vec.V2(x, y), vec.V2(0.06, 0.06), theme.withAlpha(SUNLIGHT, alpha))
	end
end

local function drawEffects()
	for _, effect in effects.list do
		local t = effect.age / effect.duration
		if effect.kind == "sprite" and effect.frames then
			local frame = effect.frames[math.min(#effect.frames, math.floor(t * #effect.frames) + 1)]
			local size = effect.size or 1
			sprites.drawInWorld(frame, effect.pos - vec.V2(size / 2, size / 2), { angle = effect.angle, size = size })
		elseif effect.kind == "dying" and effect.frames then
			local size = (effect.size or 1) * (1 - t * 0.7)
			sprites.drawInWorld(effect.frames[1], effect.pos - vec.V2(size / 2, size / 2), {
				size = size,
				angle = t * 6,
				flipX = effect.flipX,
				tint = vec4.createColor(1, 1, 1, 1 - t),
			})
		end
	end
	for _, p in effects.particles do
		local alpha = 1 - p.age / p.duration
		graphics.drawRect(p.pos - vec.V2(p.size / 2, p.size / 2), vec.V2(p.size, p.size), theme.withAlpha(p.color, alpha))
	end
end

-- MARK: Miasma

const MIASMA_TINT = vec4.createColor(1, 1, 1, 0.4)

--- Clouds of miasma drifting over their cells, over everything that stands
local function drawMiasma(game: types.Game, low: vec.Vec2, high: vec.Vec2, time: number)
	local lvl = game.level :: level.Level
	for y = low.y, high.y do
		for x = low.x, high.x do
			local ground = level.groundAt(lvl, x, y)
			if ground and ground.miasma then
				-- Two puffs per cell, drifting at their own pace, so that the clouds don't look like tiles
				for puff = 0, 1 do
					local phase = time * (0.5 + puff * 0.3) + x * 1.3 + y * 0.7 + puff * 2.1
					local frame = (math.floor(phase) + puff) % 2
					local offset = vec.V2((((x * 7 + y * 3 + puff * 5) % 5) - 2) * 0.15, (((x * 3 + y * 5 + puff) % 3) - 1) * 0.2)
					local sway = vec.V2(math.sin(phase) * 0.2, math.cos(phase * 0.8) * 0.12 - 0.45)
					sprites.drawInWorld(`world/miasma_{frame}`, vec.V2(x, y) + offset + sway, { size = 1.1 - puff * 0.25, tint = MIASMA_TINT })
				end
			end
		end
	end
end

-- MARK: Veils

--- The veils: mist drifting over their cells. Thick for the waking, thin for dreamers (they walk through it).
local function drawVeils(game: types.Game, low: vec.Vec2, high: vec.Vec2, time: number)
	local lvl = game.level :: level.Level
	local tint = vec4.createColor(1, 1, 1, if dream.isDreaming(game) then 0.18 else 0.7)
	for y = low.y, high.y do
		for x = low.x, high.x do
			local ground = level.groundAt(lvl, x, y)
			if ground and ground.veil then
				for puff = 0, 1 do
					local phase = time * (0.35 + puff * 0.2) + x * 1.1 + y * 0.9 + puff * 1.7
					local frame = (math.floor(phase) + puff) % 2
					local sway = vec.V2(math.sin(phase) * 0.12, math.cos(phase * 0.7) * 0.1 - 0.35 - puff * 0.3)
					sprites.drawInWorld(`world/veil_{frame}`, vec.V2(x, y) + sway, { size = 1.05, tint = tint })
				end
			end
		end
	end
end

-- MARK: Snow

const SNOW = theme.hex("e3ddd1")
const FLAKES = 110

--- Snow falling over the level (and the blizzard of the Last Watchman), over everything
local function drawSnow(game: types.Game, low: vec.Vec2, high: vec.Vec2, time: number)
	local amount = if frost.isBlizzard(game) then 1 else game.level.snowfall
	if amount <= 0.01 then
		return
	end
	local size = high - low + vec.V2(2, 2)
	local wind = if amount > 0.6 then 2.5 else 0.6
	for i = 1, math.floor(FLAKES * amount) do
		local seed = i * 7.31
		local fall = 1.2 + (i % 4) * 0.4
		local start = vec.V2((seed * 0.618) % 1, (seed * 0.377) % 1)
		local x = low.x - 1 + ((start.x * size.x + time * wind + math.sin(time + seed) * 0.3) % size.x)
		local y = low.y - 1 + ((start.y * size.y + time * fall) % size.y)
		local flake = if i % 3 == 0 then 0.09 else 0.06
		graphics.drawRect(vec.V2(x, y), vec.V2(flake * (1 + wind * 0.4), flake), theme.withAlpha(SNOW, 0.75))
	end
end

-- MARK: Sandstorms

const SAND = theme.hex("a29f7e")
const STREAKS = 90

--- Streaks of sand blowing with the wind, over everything (the haze itself is drawn by the darkness shader)
local function drawStorm(game: types.Game, low: vec.Vec2, high: vec.Vec2, time: number)
	local stormData = game.level.storm
	local intensity = storm.intensity(game)
	-- Blizzards (snow, see drawSnow) have no sand
	if not stormData or intensity <= 0.01 or game.level.snowfall > 0 then
		return
	end
	local alpha = intensity * (if storm.isCalmed(game) then 0.25 else 0.6)
	local size = high - low + vec.V2(2, 2)
	local wind = stormData.wind
	for i = 1, STREAKS do
		local seed = i * 7.31
		local speed = 8 + (i % 5)
		local start = vec.V2((seed * 0.618) % 1, (seed * 0.377) % 1)
		local travel = wind:scale(time * speed)
		local x = low.x - 1 + ((start.x * size.x + travel.x) % size.x)
		local y = low.y - 1 + ((start.y * size.y + travel.y) % size.y)
		local length = 0.4 + (i % 3) * 0.2
		graphics.drawRect(vec.V2(x, y), vec.V2(length, 0.06), theme.withAlpha(SAND, alpha))
	end
end

-- MARK: Key prompts

--- Distance (in tiles) at which things the player can use show the key to press above them
const PROMPT_RANGE = 3

--- The key to press above everything the player can use nearby, so that they don't look like decoration.
--- The thing [E] would use right now has a larger, bobbing key. Drawn in screen space, above the darkness.
local function drawKeyPrompts(game: types.Game, viewport: Viewport, time: number)
	local target = interactions.find(game)
	local targetProp = if target then target.prop else nil
	local key = prompts.keyIcon("E")
	for _, prop in game.level.props do
		local center = prop.cell + vec.V2(0.5, 0.5)
		if (center - game.player.pos):length() < PROMPT_RANGE and interactions.canInteract(game, prop) then
			local kind = props.kinds[prop.kind]
			local isTarget = prop == targetProp
			-- Square on the screen: OpenGL units are not square, but tiles are
			local scale = if isTarget then 0.62 else 0.45
			local size = vec.V2(viewport.tileSize.x * scale, viewport.tileSize.y * scale)
			-- Above the sprite: tall props (with a top sprite) are 2 tiles high, flat ones lie on the ground
			local top = prop.cell.y + (if kind.tall then -1.05 elseif kind.flat then 0.35 else -0.05)
			local bob = if isTarget then math.sin(time * 5) * 0.06 else 0
			local bottom = module.toScreen(viewport, vec.V2(center.x, top + bob))
			local pos = vec.V2(bottom.x - size.x / 2, bottom.y)
			local tint = if isTarget then nil else vec4.createColor(1, 1, 1, 0.55)
			keyboard_icons.keyboardInputIcons:drawTileQuad(
				key,
				pos,
				pos + vec.V2(size.x, 0),
				pos + size,
				pos + vec.V2(0, size.y),
				tint
			)
		end
	end
end

-- MARK: Drawing

export type DrawOptions = {
	time: number,
	--- Draw the darkness (the menus show the world without it)
	darkness: boolean?,
}

--- Draw the world in the viewport. Texts (floating damage numbers) are drawn after, in screen space.
function module.drawWorld(game: types.Game, viewport: Viewport, options: DrawOptions)
	local time = options.time
	local visible = module.visibleTiles(viewport)
	local low = module.camera:floor()
	local high = (module.camera + visible):ceil()
	local lvl = game.level :: level.Level
	local origin = viewport.pos + vec.V2(0, viewport.size.y)

	graphics.withTransformation({ translation = origin }, function()
		graphics.withTransformation({
			translation = module.camera:scale(-1),
			scale = vec.V2(viewport.tileSize.x, -viewport.tileSize.y),
		}, function()
			terrain_view.draw(lvl, low, high, time)
			local drawables = collectDrawables(game, low, high, time)
			local function byY(a: Drawable, b: Drawable): boolean
				return a.y < b.y
			end
			table.sort(drawables.flat, byY)
			table.sort(drawables.standing, byY)
			for _, entry in drawables.flat do
				entry.draw()
			end
			for _, shadow in drawables.shadows do
				drawShadow(shadow)
			end
			for _, entry in drawables.pickups do
				entry.draw()
			end
			for _, entry in drawables.standing do
				entry.draw()
			end
			drawMiasma(game, low, high, time)
			drawVeils(game, low, high, time)
			drawEffects()
			drawSunbeam(game, time)
		end)
	end)
	if options.darkness ~= false then
		lighting.drawDarkness(game, viewport.pos, viewport.size, module.camera, visible, time)
	end
	-- What shines through the darkness
	graphics.withTransformation({ translation = origin }, function()
		graphics.withTransformation({
			translation = module.camera:scale(-1),
			scale = vec.V2(viewport.tileSize.x, -viewport.tileSize.y),
		}, function()
			drawMendingBeams(game, time)
			if options.darkness ~= false then
				drawStorm(game, low, high, time)
				drawSnow(game, low, high, time)
			end
		end)
	end)
	if options.darkness ~= false then
		drawKeyPrompts(game, viewport, time)
	end

	-- Floating texts, in screen space so that they are not drawn upside down
	for _, effect in effects.list do
		if effect.kind == "text" and effect.text then
			local alpha = 1 - math.max(0, effect.age / effect.duration - 0.6) / 0.4
			local size = viewport.tileSize.y * 0.45
			local width = theme.font:measureText(effect.text, size).width
			local pos = module.toScreen(viewport, effect.pos) - vec.V2(width / 2, 0)
			theme.font:drawText(effect.text, pos + vec.V2(0.004, -0.006), size, theme.withAlpha(theme.colors.darkness, alpha))
			theme.font:drawText(effect.text, pos, size, theme.withAlpha(effect.color or theme.colors.text, alpha))
		end
	end
end

return module
