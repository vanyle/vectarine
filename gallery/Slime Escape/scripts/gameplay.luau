local module = {}

local Coord = require("@vectarine/coord")
local Debug = require("@vectarine/debug")
local Event = require("@vectarine/event")
local Graphics = require("@vectarine/graphics")
local Io = require("@vectarine/io")
local Resources = require("@vectarine/resources")
local Screen = require("@vectarine/screen")
local Vec = require("@vectarine/vec")
local entity = require("./entity")

local logoResource = Resources.loadImage("textures/logo.png")
local fontResource = Resources.loadFont("fonts/arial.ttf")
local _, utils = Resources.loadScript("scripts/utils.luau", require("utils.luau"))
local _, entity = Resources.loadScript("scripts/entity.luau", require("entity.luau"))

type Player = {
	position: Vec.Vec2,
	health: number,
	attackDamage: number,
}
type Projectile = {
	type: string,
	position: Vec.Vec2,
	speed: Vec.Vec2,
	damage: number,
}

local defaultPlayer = {
	position = Vec.V2(0, 0),
	health = 100,
	projectileCooldown = 0, -- nbr of seconds until next shot
	projectileRate = 1, -- in Hz
	attackDamage = 20,
}

local camera = utils.initFromGlobal({ position = Vec.V2(0, 0), zoom = 50 }, "camera")
local player = utils.initFromGlobal(defaultPlayer, "player")
local projectiles: { Projectile } = utils.initFromGlobal({}, "projectiles")

local function getPlayerSize(): Vec.Vec2
	return Vec.V2(1, 1)
end

local level = {
	{ 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 },
	{ 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1 },
	{ 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1 },
	{ 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1 },
	{ 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1 },
	{ 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1 },
	{ 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1 },
	{ 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1 },
	{ 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1 },
	{ 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 },
}

--- Converts world coordinates to camera/screen coordinates
local function getCamPos(world_pos: Vec.Vec2): Vec.Vec2
	return (world_pos - camera.position):scale(camera.zoom)
end

local function getCamSize(world_size: Vec.Vec2): Vec.Vec2
	return world_size:scale(camera.zoom)
end

local function handlePlayerInput()
	local delta: number = _G.deltaTime
	local pos: Vec.Vec2 = player.position

	local playerMovementSpeed = 5
	local dir = Vec.V2(0, 0)

	if Io.isKeyDown("z") then
		dir.y = dir.y - 1
	elseif Io.isKeyDown("s") then
		dir.y = dir.y + 1
	end
	if Io.isKeyDown("q") then
		dir.x = dir.x - 1
	elseif Io.isKeyDown("d") then
		dir.x = dir.x + 1
	end

	if dir:length() > 0.01 then
		dir = dir:scale(delta * playerMovementSpeed / dir:length())
		pos = pos + dir
	end

	player.position = pos
end

local function getClosestEntity(pos: Vec.Vec2, considerCenters: boolean, withType: string?): entity.Entity?
	local closestEntity: entity.Entity? = nil
	local closestDist = math.huge
	local entities: { entity.Entity } = entity.getEntities()
	for _, entity in pairs(entities) do
		local entityPos = entity.position + (considerCenters and Vec.V2(0.5, 0.5) or Vec.V2(0, 0))
		local dist = (entityPos - pos):length()
		if dist < closestDist and (not withType or entity.type == withType) then
			closestDist = dist
			closestEntity = entity
		end
	end
	return closestEntity
end

local function handlePlayerAutoAttack()
	-- Spawn projectiles if possible
	player.projectileCooldown = player.projectileCooldown - _G.deltaTime
	if player.projectileCooldown <= 0 then
		player.projectileCooldown = player.projectileCooldown + (1 / player.projectileRate)
		-- Fire one (or more) projectiles.
		if #projectiles < 50 then
			-- TODO: multiple projectiles might be shot per frame.
			local closestEntity = getClosestEntity(player.position, "slime")
			if closestEntity ~= nil then
				local towardsEntity = (closestEntity.position - player.position)
				towardsEntity = towardsEntity:scale(1 / towardsEntity:length())
				local projectile = {
					type = "bullet",
					position = player.position,
					speed = towardsEntity:scale(10),
					damage = player.attackDamage,
				}
				table.insert(projectiles, projectile)
			end
		end
	end

	-- Move and clean projectiles
	local delta: number = _G.deltaTime
	for i = #projectiles, 1, -1 do
		local projectile = projectiles[i]
		projectile.position = projectile.position + projectile.speed:scale(delta)

		local closestEntity = getClosestEntity(projectile.position, "slime")
		if closestEntity then
			local distanceToClosest = (projectile.position - closestEntity.position):length()
			if distanceToClosest < 0.5 then
				closestEntity.health = closestEntity.health - projectile.damage
				-- Piercing
				-- table.remove(projectiles, i)
			end
		end

		-- Clean if far
		local distToPlayer = (projectile.position - player.position):length()
		if distToPlayer > 30 then
			table.remove(projectiles, i)
		end
	end
end

local function drawHealthBar()
	local total_health_bar_width = camera.zoom * 3
	local health_bar_width = total_health_bar_width * (player.health / 100 * 0.7)
	Graphics.drawRect(
		Coord.gl(1, 1) + Coord.pxDelta(-200, 50) - Coord.pxDelta(5, 5),
		Coord.pxDelta(total_health_bar_width + 10, 30),
		{ r = 0, g = 0, b = 0, a = 1 }
	)
	Graphics.drawRect(
		Coord.gl(1, 1) + Coord.pxDelta(-200, 50),
		Coord.pxDelta(total_health_bar_width, 20),
		{ r = 0.5, g = 0.5, b = 0.5, a = 1 }
	)
	Graphics.drawRect(
		Coord.gl(1, 1) + Coord.pxDelta(-200, 50),
		Coord.pxDelta(health_bar_width, 20),
		{ r = 1, g = 0, b = 0, a = 1 }
	)
	Graphics.drawText(
		tostring(player.health),
		fontResource,
		Coord.gl(1, 1) + Coord.pxDelta(-200, 67),
		0.07,
		{ r = 0, g = 0, b = 0, a = 1 }
	)
end

function renderGameplay()
	handlePlayerInput()
	entity.entityBehavior()
	handlePlayerAutoAttack()

	Debug.fprint("Player Health: ", player.health)

	-- Move camera to center player:
	local cameraSpeed = 0.01
	camera.position = camera.position + (player.position - Vec.V2(8, 5) - camera.position):scale(cameraSpeed)

	Graphics.drawRect(Coord.gl(-1, -1), Coord.glDelta(2, 2), { r = 0.2, g = 0, b = 0, a = 1 })

	-- Draw level
	for y = 1, #level do
		for x = 1, #level[y] do
			local tile = level[y][x]
			if tile ~= 0 then
				local color = tile == 1 and { r = 0.5, g = 0.5, b = 0.5, a = 1 } or { r = 0.8, g = 0.8, b = 0.8, a = 1 }
				local tile_pos = getCamPos(Vec.V2((x - 1), (y - 1)))
				local tile_size = getCamSize(Vec.V2(1, 1))
				Graphics.drawRect(Coord.px(tile_pos.x, tile_pos.y), Coord.pxDelta(tile_size.x, tile_size.y), color)
			end
		end
	end

	-- Draw entities
	local entities: { entity.Entity } = entity.getEntities()
	for _, entity in pairs(entities) do
		local entity_screen_pos = getCamPos(entity.position)
		Graphics.drawRect(
			Coord.px(entity_screen_pos.x, entity_screen_pos.y),
			Coord.pxDelta(camera.zoom, camera.zoom),
			{ r = 0, g = 1, b = 0, a = 1 }
		)
		if entity.isHealthShown then
			local health_bar_width = camera.zoom * (entity.health / 100)
			Graphics.drawRect(
				Coord.px(
					entity_screen_pos.x - (camera.zoom / 2) + (health_bar_width / 2),
					entity_screen_pos.y - camera.zoom * 0.6
				),
				Coord.pxDelta(health_bar_width, camera.zoom * 0.2),
				{ r = 1, g = 0, b = 0, a = 1 }
			)
		end
	end

	-- Draw player
	local player_screen_pos = getCamPos(player.position)
	local player_screen_size = getCamSize(Vec.V2(1, 1))
	Graphics.drawRect(
		Coord.px(player_screen_pos.x, player_screen_pos.y),
		Coord.pxDelta(player_screen_size.x, player_screen_size.y),
		{ r = 0, g = 0, b = 1, a = 1 }
	)

	-- Player health bar
	drawHealthBar()

	-- Draw projectiles
	for _, projectile in pairs(projectiles) do
		local projectile_size = getCamSize(Vec.V2(0.2, 0.2))
		local projectile_screen_pos = getCamPos(projectile.position + Vec.V2(0.5, 0.5) - Vec.V2(0.1, 0.1))
		Graphics.drawRect(
			Coord.px(projectile_screen_pos.x, projectile_screen_pos.y),
			Coord.pxDelta(projectile_size.x, projectile_size.y),
			{ r = 1, g = 1, b = 0, a = 1 }
		)
	end
end

module.screen = Screen.newScreen("Game", function()
	renderGameplay()
end)

return module
