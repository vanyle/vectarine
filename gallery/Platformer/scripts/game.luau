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
local Camera = require("@vectarine/camera")
local Graphics = require("@vectarine/graphics")
local Io = require("@vectarine/io")
local Loader = require("@vectarine/loader")
local Persist = require("@vectarine/persist")
local Physics = require("@vectarine/physics")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")

-- Script resources
local fRes, f = Loader.loadScript("./scripts/f.luau", require("./f"))
local InteractableRes, Interactable = Loader.loadScript("./scripts/interactable.luau", require("./interactable"))
local InteractableTypes = require("./interactable")
local WorldRes, World = Loader.loadScript("./scripts/world.luau", require("./world"))
local WorldTypes = require("./world")

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

local tiles_tiles = Loader.loadTileset("textures/1bit_platformer.tsx")
local tiles_image = Loader.loadImage("textures/1bit_platformer.png", false)
local level = Loader.loadTilemap("textures/level.tmx")
local tiles = tiles_image:withTileset(tiles_tiles)

local icons_image = Loader.loadImage("textures/icons_sheet.png", true)
local icons_tiles = Loader.loadTileset("textures/icons_sheet.tsx")
local icons = icons_image:withTileset(icons_tiles)

-- Persisted state
local camera = Persist.onReload(Camera.new(), "camera")

local player = Persist.onReload({
	o = (nil :: any) :: Physics.Object2,
}, "player")

local movement = Persist.onReload({
	jumpSpeed = PLAYER_JUMP_SPEED,
	moveSpeed = PLAYER_MOVE_SPEED,
}, "movement")

local colliders = Persist.onReload({
	show = false,
	isSmallPlayer = false,
}, "colliders")
local isSmallPlayer = false

local debugging = Persist.onReload({
	noclip = false,
	zoomLevel = 100,
}, "debugging")

local areResourcesLoaded = Persist.onReload({ loaded = false }, "areResourcesLoaded")

if _G.isFirstLaunch == nil then
	_G.isFirstLaunch = true
	Io.centerWindow() -- Center the window once.
end

-- Non persisted state

local world = Physics.newWorld2(Vec.ZERO2, camera)
local factor = Vec.V2(0.1, -0.1)
local offset = Vec.V2(-1, 1)

local infiniteWorld: WorldTypes.IWorld

local interactibleManager: InteractableTypes.InteractableManager

function makePlayer()
	-- By default, we make the player collider way too big.
	local newSize = colliders.isSmallPlayer and Vec.V2(0.4, 0.4) or Vec.V2(0.4, 0.7)
	local newCollider = Physics.newRectangleCollider(newSize)
	local playerPosition = player.o and player.o.position or Vec.V2(5, 6)
	if (player.o :: Physics.Object2?) ~= nil then
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
function Init()
	infiniteWorld =
		World.createInfiniteWorld(world, level, COLLISION_LAYER, SOLID_BLOCKS, LEVEL_SIZE, tiles, tiles_tiles, {
			{ index = FAR_BACKGROUND_LAYER, tint = Vec4.GRAY },
			{ index = COLLISION_LAYER, tint = Vec4.WHITE },
			{ index = FOREGROUND_LAYER, tint = Vec4.WHITE },
		})

	player.o = makePlayer()
	camera.position = Vec.ZERO2 - player.o.position
	camera.zoom = debugging.zoomLevel

	interactibleManager = Interactable.newInteractibleManager(tiles, icons)
	Interactable.newInteractible(interactibleManager, "npc", Vec.V2(6, 6), {
		"Hello! (Press B to continue)",
		"Welcome the Vectarine physics demo",
		"This demo will show you how to use Vectarine's physics system",
		"It has also uses tilemaps to represent an infinite level.",
		"Use the arrow keys to move.",
	})
	Interactable.newInteractible(
		interactibleManager,
		"npc",
		Vec.V2(12, 6),
		{ "Use the up arrow to jump", "You can restart this demo with Ctrl+R" }
	)

	Interactable.newInteractible(interactibleManager, "npc", Vec.V2(23, 6), {
		"You are too big to fit inside this gap.",
		"Vectarine has a watcher tool that you can open using Ctrl+3",
		"You can also open it from the Tools menu at the top.",
		"Using the watcher tool, search for the colliders variable.",
		"Try to check the show checkbox",
		"You will be able to change your collider size from there too.",
	})

	Interactable.newInteractible(interactibleManager, "npc", Vec.V2(33, 4), {
		"Doors are currently disabled",
		"Using the watcher tool, search for doors to enable them",
	})

	Interactable.newInteractible(interactibleManager, "open_door", Vec.V2(30, 4), {}, Vec.V2(30, -1))

	Interactable.newInteractible(interactibleManager, "open_door", Vec.V2(30, -1), {}, Vec.V2(30, 4))

	Interactable.newInteractible(interactibleManager, "npc", Vec.V2(48, 8), {
		"Looks like you're stuck in a hole.",
		"You can search for 'movement' in watcher to change your jump settings",
	})

	Interactable.newInteractible(interactibleManager, "npc", Vec.V2(54, -1), {
		"This is the end of the level",
		"You can explore it in no-clip mode!",
		"Enable it using the debugging object in watcher",
		"Have fun testing and modifying the demo! :)",
	})

	Interactable.newInteractible(interactibleManager, "npc", Vec.V2(49, -3), {
		"Check the resources with Ctrl+2",
		"You can open level.tmx with Tiled or another level editor",
		"You can edit the level while playing the game!",
		"These signs and other interactible objects are not part of the level!",
	})

	world.gravity = Vec.V2(0, 60)
end
-- If the resource are already loaded (hot reload), we can init right away.
-- Otherwise, we need to wait for the resource to load.
if areResourcesLoaded.loaded then
	Init()
end

function Update(deltaTime: number)
	if
		Graphics.drawSplashScreenIfNeeded({
			tiles_tiles,
			tiles_image,
			fRes,
			InteractableRes,
			WorldRes,
			icons_tiles,
			icons_image,
		})
	then
		return
	end
	if not areResourcesLoaded.loaded then
		areResourcesLoaded.loaded = true
		Init()
		-- We need to want one frame for init to finish.
		Graphics.drawSplashScreen("Initializing.")
		return
	end

	if isSmallPlayer ~= colliders.isSmallPlayer then
		isSmallPlayer = colliders.isSmallPlayer
		player.o = makePlayer()
	end

	World.update(infiniteWorld, player.o.position, deltaTime)

	camera.zoom = debugging.zoomLevel / 100

	local groundDetectWidth = 0.6 -- Larger value = coyote time kind of.
	local groundDetectYOffset = colliders.isSmallPlayer and -0.3 or -0.55
	local groundDetectHeight = 0.3
	local onGroundZone = {
		pos = player.o.position - Vec.V2(groundDetectWidth / 2, groundDetectYOffset),
		size = Vec.V2(groundDetectWidth, groundDetectHeight),
	}

	if debugging.noclip then
		player.o.position = player.o.position
			+ Vec.V2(
				((Io.isKeyDown("Left") and -10 or 0) + (Io.isKeyDown("Right") and 10 or 0)),
				((Io.isKeyDown("Up") and -10 or 0) + (Io.isKeyDown("Down") and 10 or 0))
			):scale(deltaTime)
	else
		-- Cannot move while speaking (this is impolite)
		if interactibleManager.interactingObject == nil then
			player.o.speed = player.o.speed
				+ Vec.V2(
					(
						(Io.isKeyDown("Left") and -movement.moveSpeed or 0)
						+ (Io.isKeyDown("Right") and movement.moveSpeed or 0)
					),
					0
				):scale(deltaTime)
		end
		if not Io.isKeyDown("Left") and not Io.isKeyDown("Right") then
			-- Make the player faster to stop for nicer controls.
			player.o.speed = player.o.speed * Vec.V2(0.95, 1)
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
		if Io.isKeyJustPressed("Up") and isAboveSolidGround and player.o.speed.y > -0.1 then
			player.o.speed = player.o.speed + Vec.V2(0, -movement.jumpSpeed)
		end
	end

	-- We need to more clearly distinguish between world pos and screen pos and coordinate system and stuff.
	camera:moveTowards(Vec.ZERO2 - player.o.position, 0.02)

	local windowSize = Io.getWindowSize()
	local yScale = windowSize.x / windowSize.y
	factor = Vec.V2(0.1, -0.1 * yScale)
	offset = Vec.V2(-1, 1)

	-- The player can be modeled as a square with 4 corners.
	-- local integerPlayerPos = ((player.o.position - offset) / factor):floor() -- Check the bottom left corder of the player.
	-- local t = level:getTile(COLLISION_LAYER, integerPlayerPos.x, integerPlayerPos.y + 1) -- +y = down

	Graphics.clear(Vec4.BLACK)
	Graphics.withTransformation({ translation = camera.position, scale = factor:scale(camera.zoom) }, function()
		Graphics.withTransformation({ translation = offset }, function()
			World.draw(infiniteWorld)
			Interactable.drawInteractibles(interactibleManager, deltaTime, player.o.position, player)
			local playerDrawOffset = colliders.isSmallPlayer and -0.41 or -0.7
			tiles:drawTile("player", (player.o.position - Vec.V2(0.5, playerDrawOffset)), Vec.V2(1, -1))
			drawColliders(onGroundZone)
		end)
	end)

	if not debugging.noclip then
		world:step(deltaTime)
	end
end

function drawColliders(onGroundZone: { pos: Vec.Vec2, size: Vec.Vec2 })
	if not colliders.show then
		return
	end

	-- Marker for the origin
	Graphics.drawCircle(Vec.V2(0, 0), 0.2, Vec4.GREEN)

	-- Player collider
	Graphics.drawPolygon(player.o:getPoints(), Vec4.V4(1, 0, 0, 0.5))

	-- Draw ground colliders for debug
	World.drawColliders(infiniteWorld)

	Graphics.drawRect(onGroundZone.pos, onGroundZone.size, Vec4.BLUE)
end
