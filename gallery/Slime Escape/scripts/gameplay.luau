local module = {}

local Coord = require("@vectarine/coord")
local Debug = require("@vectarine/debug")
local Graphics = require("@vectarine/graphics")
local Io = require("@vectarine/io")
local Loader = require("@vectarine/loader")
local Persist = require("@vectarine/persist")
local Screen = require("@vectarine/screen")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")
local entity = require("./entity")

-- local logoResource = Loader.loadImage("textures/logo.png")
local fontResource = Loader.loadFont("fonts/arial.ttf")
local _, entity = Loader.loadScript("scripts/entity.luau", require("entity.luau"))

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

local camera = Persist.onReload({ position = Vec.V2(0, 0), zoom = 50 }, "camera")
local player = Persist.onReload(defaultPlayer, "player")
local projectiles = Persist.onReload({}, "projectiles")

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
			local closestEntity = getClosestEntity(player.position, true, "slime")
			if closestEntity ~= nil then
				local towardsEntity = (closestEntity.position - player.position)
				towardsEntity = towardsEntity:normalized()
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

		local closestEntity = getClosestEntity(projectile.position, true, "slime")
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
		Coord.gl(Vec.V2(1, 1)) + Coord.pxVec(Vec.V2(-200, 50)) - Coord.pxVec(Vec.V2(5, 5)),
		Coord.pxVec(Vec.V2(total_health_bar_width + 10, 30)),
		Vec4.BLACK
	)
	Graphics.drawRect(
		Coord.gl(Vec.V2(1, 1)) + Coord.pxVec(Vec.V2(-200, 50)),
		Coord.pxVec(Vec.V2(total_health_bar_width, 20)),
		Vec4.GRAY
	)
	Graphics.drawRect(
		Coord.gl(Vec.V2(1, 1)) + Coord.pxVec(Vec.V2(-200, 50)),
		Coord.pxVec(Vec.V2(health_bar_width, 20)),
		Vec4.RED
	)
	fontResource:drawText(
		tostring(player.health),
		Coord.gl(Vec.V2(1, 1)) + Coord.pxVec(Vec.V2(-200, 67)),
		0.07,
		Vec4.BLACK
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

	Graphics.drawRect(Coord.gl(Vec.V2(-1, -1)), Coord.glVec(Vec.V2(2, 2)), Vec4.createColor(0.2, 0, 0, 1))

	-- Draw level
	for y = 1, #level do
		for x = 1, #level[y] do
			local tile = level[y][x]
			if tile ~= 0 then
				local color = tile == 1 and Vec4.GRAY or Vec4.LIGHT_GRAY
				local tile_pos = getCamPos(Vec.V2((x - 1), (y - 1)))
				local tile_size = getCamSize(Vec.V2(1, 1))
				Graphics.drawRect(
					Coord.px(Vec.V2(tile_pos.x, tile_pos.y)),
					Coord.pxVec(Vec.V2(tile_size.x, tile_size.y)),
					color
				)
			end
		end
	end

	-- Draw entities
	local entities: { entity.Entity } = entity.getEntities()
	for _, entity in pairs(entities) do
		local entity_screen_pos = getCamPos(entity.position)
		Graphics.drawRect(
			Coord.px(Vec.V2(entity_screen_pos.x, entity_screen_pos.y)),
			Coord.pxVec(Vec.V2(camera.zoom, camera.zoom)),
			Vec4.GREEN
		)
		if entity.isHealthShown then
			local health_bar_width = camera.zoom * (entity.health / 100)
			Graphics.drawRect(
				Coord.px(
					Vec.V2(
						entity_screen_pos.x - (camera.zoom / 2) + (health_bar_width / 2),
						entity_screen_pos.y - camera.zoom * 0.6
					)
				),
				Coord.pxVec(Vec.V2(health_bar_width, camera.zoom * 0.2)),
				Vec4.RED
			)
		end
	end

	-- Draw player
	local player_screen_pos = getCamPos(player.position)
	local player_screen_size = getCamSize(Vec.V2(1, 1))
	Graphics.drawRect(
		Coord.px(Vec.V2(player_screen_pos.x, player_screen_pos.y)),
		Coord.pxVec(Vec.V2(player_screen_size.x, player_screen_size.y)),
		Vec4.BLUE
	)

	-- Player health bar
	drawHealthBar()

	-- Draw projectiles
	for _, projectile in pairs(projectiles) do
		local projectile_size = getCamSize(Vec.V2(0.2, 0.2))
		local projectile_screen_pos = getCamPos(projectile.position + Vec.V2(0.5, 0.5) - Vec.V2(0.1, 0.1))
		Graphics.drawRect(
			Coord.px(Vec.V2(projectile_screen_pos.x, projectile_screen_pos.y)),
			Coord.pxVec(Vec.V2(projectile_size.x, projectile_size.y)),
			Vec4.YELLOW
		)
	end
end

module.screen = Screen.newScreen("Game", function()
	renderGameplay()
end)

return module
