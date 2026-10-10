--[[
Light and darkness. The explorium is dark: the player only sees around light sources.

- Each level has a `darkness` (0 = bright, 1 = pitch black). Some parts of a level are darker (`darkAreas`): the
  darkness changes smoothly when the player walks in and out of them (`update`).
- Light sources: campfires, glowing mushrooms, active teleporters... (the `light` radius of props), and the
  `lights` of the level (the sunlight of the sinkhole, for example).
- The player carries a flashlight: a small halo around them and a cone in front. The eternal lantern gives a large
  halo instead, and the firefly jar a bit more.
- Sanity recovers faster in the light: near a light source, or anywhere with the lantern.

The darkness is drawn by a shader (shaders/darkness.glsl) that computes the light of every pixel on the GPU.
`lightAt` computes the same light in Lua, for the rules: keep the two in sync.
]]
const canvas = require("@vectarine/canvas")
const inventory = require("./inventory")
const items = require("./data/items")
const levels = require("./data/levels")
const loader = require("@vectarine/loader")
const props = require("./data/props")
const storm = require("./storm")
const theme = require("./theme")
const types = require("./types")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

const module = loader.init() or {}

--- The darkness around the player right now: it follows the dark areas of the level smoothly (not saved)
module.state = module.state or { levelId = "", darkness = 0 }

--- How dark the place of the player is: the darkness of the level, or of the dark area they are in
function module.targetDarkness(game: types.Game): number
	local cell = vec.V2(math.floor(game.player.pos.x), math.floor(game.player.pos.y))
	for _, area in game.level.darkAreas do
		if cell.x >= area[1] and cell.y >= area[2] and cell.x <= area[3] and cell.y <= area[4] then
			return area[5]
		end
	end
	return game.level.darkness
end

--- Follow the darkness of the place of the player, smoothly (instantly when entering a level)
function module.update(game: types.Game, dt: number)
	local state = module.state
	local target = module.targetDarkness(game)
	if state.levelId ~= game.level.id then
		state.levelId = game.level.id
		state.darkness = target
	end
	state.darkness += (target - state.darkness) * math.min(1, dt * 2.5)
end

--- The darkness drawn right now
function module.darkness(game: types.Game): number
	return if module.state.levelId == game.level.id then module.state.darkness else game.level.darkness
end

module.shader = loader.loadShader("shaders/darkness.glsl")

--- The shader handles this many light sources at once (the ones closest to the view)
const MAX_LIGHTS = 10
--- Length of the beam of the flashlight, in tiles
const FLASHLIGHT_RANGE = 6.5
--- The color of the sand of the storms
const HAZE_COLOR = theme.hex("969372")
--- The blizzards of the frontier are white
const SNOW_HAZE_COLOR = theme.hex("bdbbae")

export type Light = { pos: vec.Vec2, radius: number }

const DIRECTIONS: { [types.Direction]: vec.Vec2 } = {
	up = vec.V2(0, -1),
	down = vec.V2(0, 1),
	left = vec.V2(-1, 0),
	right = vec.V2(1, 0),
}

--- Light sources of the level (not the player's)
function module.sources(game: types.Game): { Light }
	local list: { Light } = {}
	for _, prop in game.level.props do
		local kind = props.kinds[prop.kind]
		local radius = kind.light
		if prop.kind == "teleporter" and game.flags[`teleporter:{prop.key}`] then
			radius = 2.5
		end
		if radius and not game.flags[`broken:{prop.key}`] then
			table.insert(list, { pos = prop.cell + vec.V2(0.5, 0.5), radius = radius })
		end
	end
	local data = levels.levels[game.level.id]
	for _, light in data.lights or {} do
		table.insert(list, { pos = vec.V2(light[1] + 0.5, light[2] + 0.5), radius = light[3] })
	end
	return list
end

--- Radius of the halo around the player
function module.playerRadius(player: types.Player): number
	local radius = if player.artefacts.eternal_lantern then 5.5 else 2.2
	-- A torch in hand lights the way too
	local weapon = items.items[inventory.weapon(player)]
	return radius + inventory.charmBonus(player, "light") + (weapon.light or 0)
end

--- Where the light carried by the player comes from (their hands, above their feet)
function module.haloCenter(player: types.Player): vec.Vec2
	return player.pos + vec.V2(0, -0.4)
end

local function falloff(distance: number, radius: number): number
	local t = distance / radius
	if t >= 1 then
		return 0
	end
	return 1 - t * t
end

--- How much light reaches a point, from 0 (dark) to 1 (lit). `sources` comes from `module.sources`.
function module.lightAt(game: types.Game, sources: { Light }, pos: vec.Vec2): number
	local light = 0
	for _, source in sources do
		light = math.max(light, falloff((pos - source.pos):length(), source.radius))
	end
	local player = game.player
	local center = module.haloCenter(player)
	local offset = pos - center
	local distance = offset:length()
	light = math.max(light, falloff(distance, module.playerRadius(player)))
	-- The flashlight: a cone in front of the player
	if not player.artefacts.eternal_lantern and distance > 0.01 then
		local forward = DIRECTIONS[player.facing]
		local alignment = offset:dot(forward) / distance
		if alignment > 0.8 then
			local cone = falloff(distance, FLASHLIGHT_RANGE) * math.min(1, (alignment - 0.8) / 0.1)
			light = math.max(light, cone * 0.95)
		end
	end
	return light
end

--- Is the player in the light (where sanity recovers)?
function module.isSafe(game: types.Game, sources: { Light }): boolean
	if module.darkness(game) < 0.35 or game.player.artefacts.eternal_lantern then
		return true
	end
	for _, source in sources do
		if (game.player.pos - source.pos):length() < source.radius * 0.8 then
			return true
		end
	end
	return false
end

-- MARK: Drawing

--- The shader is drawn through a tiny canvas that is never painted: the shader only needs the position of each pixel
local overlay: canvas.Canvas? = nil

--- Draw the darkness over the view at `pos` (bottom-left, OpenGL coordinates) of size `size`, which shows the
--- `visible` tiles starting from `camera` (the top-left tile). The light of fires flickers with `time`.
function module.drawDarkness(game: types.Game, pos: vec.Vec2, size: vec.Vec2, camera: vec.Vec2, visible: vec.Vec2, time: number)
	local darkness = module.darkness(game)
	if darkness <= 0 or not module.shader:isReady() then
		return
	end
	if not overlay then
		overlay = canvas.createCanvas(4, 4)
	end
	local target = overlay :: canvas.Canvas
	target:setShader(module.shader)

	-- The light sources closest to the middle of the view
	local middle = camera + visible:scale(0.5)
	local sources = module.sources(game)
	table.sort(sources, function(a: Light, b: Light): boolean
		return (a.pos - middle):length() - a.radius < (b.pos - middle):length() - b.radius
	end)
	local count = math.min(#sources, MAX_LIGHTS)
	for i = 1, count do
		local source = sources[i]
		-- Fires flicker
		local radius = source.radius * (1 + math.sin(time * 9 + source.pos.x) * 0.03)
		target:setUniform(`lights[{i - 1}]`, vec4.createColor(source.pos.x, source.pos.y, radius, 0))
	end
	target:setUniform("lightCount", count)

	local player = game.player
	local center = module.haloCenter(player)
	local flashlight = if player.artefacts.eternal_lantern then 0 else 1
	target:setUniform("halo", vec4.createColor(center.x, center.y, module.playerRadius(player), flashlight))
	target:setUniform("facing", DIRECTIONS[player.facing])
	target:setUniform("camera", camera)
	target:setUniform("visible", visible)
	target:setUniform("darkness", darkness)
	target:setUniform("shadowColor", theme.colors.darkness)
	target:setUniform("haze", storm.haze(game))
	local frozen = levels.levels[game.level.id].style == "frost"
	target:setUniform("hazeColor", if frozen then SNOW_HAZE_COLOR else HAZE_COLOR)
	target:setUniform("hazeRadius", if storm.isCalmed(game) then 4 else 2.6)
	target:draw(pos, size)
end

return module
