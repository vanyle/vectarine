--[[

This example shows you how to use the Physics and Tile modules
to create a platformer with a potentially infinite level.

You can check "world.luau" to see how the world is created, updated and drawn.

The assets come from:
- Kenney's 1-bit platformer pack: https://kenney.nl/assets/1-bit-platformer-pack
- Kenney's topdown icons: https://kenney.nl/assets/game-icons

You can open the level.tmx file with Tiled to edit it while the game is running!

]]

-- Modules
local camera = require("@vectarine/camera")
local graphics = require("@vectarine/graphics")
local io = require("@vectarine/io")
local loader = require("@vectarine/loader")
local persist = require("@vectarine/persist")
local physics = require("@vectarine/physics")
local vec = require("@vectarine/vec")
local vec4 = require("@vectarine/vec4")

-- Script resources
local f = require("./f")
local interactableModule = require("./interactable")
local worldModule = require("./world")

-- Constants
local FOREGROUND_LAYER = 2
local COLLISION_LAYER = 1
local FAR_BACKGROUND_LAYER = 0
local SOLID_BLOCKS =
	{ 84, 85, 86, 87, 88, 89, 270, 272, 271, 127, 128, 129, 107, 108, 109, 110, 111, 112, 160, 161, 162 }
local LEVEL_SIZE = 16

local PLAYER_MOVE_SPEED = 40
local PLAYER_JUMP_SPEED = 25

-- Resources

local tiles_tiles = loader.loadTileset("textures/1bit_platformer.tsx")
local tiles_image = loader.loadImage("textures/1bit_platformer.png", false)
local level = loader.loadTilemap("textures/level.tmx")
local tiles = tiles_image:withTileset(tiles_tiles)

local icons_image = loader.loadImage("textures/icons_sheet.png", true)
local icons_tiles = loader.loadTileset("textures/icons_sheet.tsx")
local icons = icons_image:withTileset(icons_tiles)

-- Persisted state
local gameCamera = persist.onReload(camera.new(), "camera")

local player = persist.onReload({
	o = (nil :: any) :: physics.Object2,
}, "player")

local movement = persist.onReload({
	jumpSpeed = PLAYER_JUMP_SPEED,
	moveSpeed = PLAYER_MOVE_SPEED,
}, "movement")

local colliders = persist.onReload({
	show = false,
	isSmallPlayer = false,
}, "colliders")
local isSmallPlayer = false

local debugging = persist.onReload({
	noclip = false,
	zoomLevel = 100,
}, "debugging")

local areResourcesLoaded = persist.onReload({ loaded = false }, "areResourcesLoaded")

if _G.isFirstLaunch == nil then
	_G.isFirstLaunch = true
	io.centerWindow() -- Center the window once.
end

-- Non persisted state

local world = physics.newWorld2(vec.ZERO2, gameCamera)
local factor = vec.V2(0.1, -0.1)
local offset = vec.V2(-1, 1)

local interactibleManager: interactableModule.InteractableManager

function makePlayer()
	-- By default, we make the player collider way too big.
	local newSize = colliders.isSmallPlayer and vec.V2(0.4, 0.4) or vec.V2(0.4, 0.7)
	local newCollider = physics.newRectangleCollider(newSize)
	local playerPosition = player.o and player.o.position or vec.V2(5, 6)
	if (player.o :: physics.Object2?) ~= nil then
		world:removeObject(player.o)
	end
	local o = world:createObject(playerPosition, 1.0, newCollider, { "player" }, "dynamic")
	o.linearDamping = 4
	o:setLockRotation(true)
	-- Set friction to 0 so that running on the ground is as fast as in the air.
	o:setFriction(0.0)
	o:setRestitution(0.0)
	return o
end

-- Initialization function

local infiniteWorld: worldModule.IWorld =
	worldModule.createInfiniteWorld(world, level, COLLISION_LAYER, SOLID_BLOCKS, LEVEL_SIZE, tiles, tiles_tiles, {
		{ index = FAR_BACKGROUND_LAYER, tint = vec4.GRAY },
		{ index = COLLISION_LAYER, tint = vec4.WHITE },
		{ index = FOREGROUND_LAYER, tint = vec4.WHITE },
	})

player.o = makePlayer()
gameCamera.position = vec.ZERO2 - player.o.position
gameCamera.zoom = debugging.zoomLevel

interactibleManager = interactableModule.newInteractibleManager(tiles, icons)
interactableModule.newInteractible(interactibleManager, "npc", vec.V2(6, 6), {
	"Hello! (Press B to continue)",
	"Welcome the Vectarine physics demo",
	"This demo will show you how to use Vectarine's physics system",
	"It has also uses tilemaps to represent an infinite level.",
	"Use the arrow keys to move.",
})
interactableModule.newInteractible(
	interactibleManager,
	"npc",
	vec.V2(12, 6),
	{ "Use the up arrow to jump", "You can restart this demo with Ctrl+R" }
)

interactableModule.newInteractible(interactibleManager, "npc", vec.V2(23, 6), {
	"You are too big to fit inside this gap.",
	"Vectarine has a watcher tool that you can open using Ctrl+3",
	"You can also open it from the Tools menu at the top.",
	"Using the watcher tool, search for the colliders variable.",
	"Try to check the show checkbox",
	"You will be able to change your collider size from there too.",
})

interactableModule.newInteractible(interactibleManager, "npc", vec.V2(33, 4), {
	"Doors are currently disabled",
	"Using the watcher tool, search for doors to enable them",
})

interactableModule.newInteractible(interactibleManager, "open_door", vec.V2(30, 4), {}, vec.V2(30, -1))

interactableModule.newInteractible(interactibleManager, "open_door", vec.V2(30, -1), {}, vec.V2(30, 4))

interactableModule.newInteractible(interactibleManager, "npc", vec.V2(48, 8), {
	"Looks like you're stuck in a hole.",
	"You can search for 'movement' in watcher to change your jump settings",
})

interactableModule.newInteractible(interactibleManager, "npc", vec.V2(54, -1), {
	"This is the end of the level",
	"You can explore it in no-clip mode!",
	"Enable it using the debugging object in watcher",
	"Have fun testing and modifying the demo! :)",
})

interactableModule.newInteractible(interactibleManager, "npc", vec.V2(49, -3), {
	"Check the resources with Ctrl+2",
	"You can open level.tmx with Tiled or another level editor",
	"You can edit the level while playing the game!",
	"These signs and other interactible objects are not part of the level!",
})

world.gravity = vec.V2(0, 60)

function Update(deltaTime: number)
	if graphics.drawSplashScreenIfNeeded({
		tiles_tiles,
		tiles_image,
		icons_tiles,
		icons_image,
	}) then
		return
	end

	if isSmallPlayer ~= colliders.isSmallPlayer then
		isSmallPlayer = colliders.isSmallPlayer
		player.o = makePlayer()
	end

	infiniteWorld:update(player.o.position, deltaTime)

	gameCamera.zoom = debugging.zoomLevel / 100

	local groundDetectWidth = 0.6 -- Larger value = coyote time kind of.
	local groundDetectYOffset = colliders.isSmallPlayer and -0.3 or -0.55
	local groundDetectHeight = 0.3
	local onGroundZone = {
		pos = player.o.position - vec.V2(groundDetectWidth / 2, groundDetectYOffset),
		size = vec.V2(groundDetectWidth, groundDetectHeight),
	}

	if debugging.noclip then
		player.o.position = player.o.position
			+ vec.V2(
				((io.isKeyDown("Left") and -10 or 0) + (io.isKeyDown("Right") and 10 or 0)),
				((io.isKeyDown("Up") and -10 or 0) + (io.isKeyDown("Down") and 10 or 0))
			):scale(deltaTime)
	else
		-- Cannot move while speaking (this is impolite)
		if interactibleManager.interactingObject == nil then
			player.o.speed = player.o.speed
				+ vec.V2(
					(
						(io.isKeyDown("Left") and -movement.moveSpeed or 0)
						+ (io.isKeyDown("Right") and movement.moveSpeed or 0)
					),
					0
				):scale(deltaTime)
		end
		if not io.isKeyDown("Left") and not io.isKeyDown("Right") then
			-- Make the player faster to stop for nicer controls.
			player.o.speed = player.o.speed * vec.V2(0.95, 1)
		end

		local objectsBelowPlayer = world:getObjectsInArea(onGroundZone.pos, onGroundZone.size)
		local isAboveSolidGround = false
		for _, o in objectsBelowPlayer do
			if f.contains(o.tags, "ground") then
				isAboveSolidGround = true
				break
			end
		end

		-- You cannot jump if you are already going up fast (prevents tapping the button twice in a row super quickly)
		if io.isKeyJustPressed("Up") and isAboveSolidGround and player.o.speed.y > -0.1 then
			player.o.speed = player.o.speed + vec.V2(0, -movement.jumpSpeed)
		end
	end

	-- We need to more clearly distinguish between world pos and screen pos and coordinate system and stuff.
	gameCamera:moveTowards(vec.ZERO2 - player.o.position, 0.02)

	local windowSize = io.getWindowSize()
	local yScale = windowSize.x / windowSize.y
	factor = vec.V2(0.1, -0.1 * yScale)
	offset = vec.V2(-1, 1)

	-- The player can be modeled as a square with 4 corners.
	-- local integerPlayerPos = ((player.o.position - offset) / factor):floor() -- Check the bottom left corder of the player.
	-- local t = level:getTile(COLLISION_LAYER, integerPlayerPos.x, integerPlayerPos.y + 1) -- +y = down

	graphics.clear(vec4.BLACK)
	graphics.withTransformation({ translation = gameCamera.position, scale = factor:scale(gameCamera.zoom) }, function()
		graphics.withTransformation({ translation = offset }, function()
			infiniteWorld:draw()
			interactableModule.drawInteractibles(interactibleManager, deltaTime, player.o.position, player)
			local playerDrawOffset = colliders.isSmallPlayer and -0.41 or -0.7
			tiles:drawTile("player", (player.o.position - vec.V2(0.5, playerDrawOffset)), vec.V2(1, -1))
			drawColliders(onGroundZone)
		end)
	end)

	if not debugging.noclip then
		world:step(deltaTime)
	end
end

function drawColliders(onGroundZone: { pos: vec.Vec2, size: vec.Vec2 })
	if not colliders.show then
		return
	end

	-- Marker for the origin
	graphics.drawCircle(vec.V2(0, 0), 0.2, vec4.GREEN)

	-- Player collider
	graphics.drawPolygon(player.o:getPoints(), vec4.V4(1, 0, 0, 0.5))

	-- Draw ground colliders for debug
	infiniteWorld:drawColliders()

	graphics.drawRect(onGroundZone.pos, onGroundZone.size, vec4.BLUE)
end
