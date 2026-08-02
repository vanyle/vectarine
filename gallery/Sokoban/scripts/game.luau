--[[
A Sokoban game with one big world that you can explore.

You can edit the level inside textures/level.tmx using Tiled and see changes live.

- There is a special mecanic using red and blue potions
- You can push multiple crates at once
- You have infinite undo

The assets come from https://kenney.nl/assets/tiny-dungeon with a few tweaks.
]]

local camera = require("@vectarine/camera")
local debug = require("@vectarine/debug")
local event = require("@vectarine/event")
local graphics = require("@vectarine/graphics")
local io = require("@vectarine/io")
local loader = require("@vectarine/loader")
local persist = require("@vectarine/persist")
local vec = require("@vectarine/vec")
local vec4 = require("@vectarine/vec4")

local sign = require("./sign")
local worldManager = require("./world")

local tiles_tiles = loader.loadTileset("textures/tileset.tsx")
local tiles_image = loader.loadImage("textures/tileset.png", false)
local level = loader.loadTilemap("textures/level.tmx")
local tiles = tiles_image:withTileset(tiles_tiles)

-- Controls how the camera moves. The world is divided into parts of size 20x15 and the camera moves between parts.
local ROOM_SIZE = vec.V2(15, 10)

local areResourcesLoaded = persist.onReload({ loaded = false }, "areResourcesLoaded")

local gameCamera = persist.onReload(camera.new(), "camera")
local ZOOM_LEVEL = 0.12 -- This factor is choosen so that you can see the current room + about 1.5 tile of the surrounding ones

local world: worldManager.IWorld
local signManager: sign.SignManager

-- You can explore the level freely by setting noclip to true from the watcher.
local debugging = persist.onReload({
	noclip = false,
}, "debugging")

function Init()
	world = worldManager.createInfiniteWorld(level, tiles, tiles_tiles)
	signManager = sign.create()
	gameCamera.position = -ROOM_SIZE:scale(0.5)
end

if areResourcesLoaded.loaded then
	Init()
end

event.getConsoleCommandEvent():on(function(command: string)
	if command == "entities" then
		worldManager.reloadEntities(world, level, tiles)
	else
		debug.print("Unknown command: " .. command)
		debug.print("Use entities to reset entity position from the resources")
	end
end)

function Update(deltaTime: number)
	if graphics.drawSplashScreenIfNeeded({
		tiles_tiles,
		tiles_image,
	}) then
		return
	end
	if not areResourcesLoaded.loaded then
		areResourcesLoaded.loaded = true
		Init()
		-- We need to wait one frame for init to finish.
		graphics.drawSplashScreen("Initializing.")
		return
	end

	local windowSize = io.getWindowSize()
	local yScale = windowSize.x / windowSize.y
	local factor = vec.V2(ZOOM_LEVEL, -ZOOM_LEVEL * yScale)
	local offset = vec.V2(-1, 1)

	local roomCenter = (world.player.position / ROOM_SIZE):floor() * ROOM_SIZE + ROOM_SIZE:scale(0.5) - vec.V2(1, 0)
	gameCamera:moveTowards(-roomCenter, 0.05)

	-- We don't allow diagonal movement.
	-- It could be a cool gameplay concept, but it is slightly hard to press the 2 keys at the same time.
	local upMotion = io.isKeyJustPressed("Up") and -1 or io.isKeyJustPressed("Down") and 1 or 0
	local rightMotion = io.isKeyJustPressed("Right") and 1 or io.isKeyJustPressed("Left") and -1 or 0
	if not debugging.noclip then
		-- You cannot move while reading a sign, it is rude.
		if not world.readingSign then
			if upMotion ~= 0 then
				worldManager.tryMovePlayer(world, 0, upMotion * world.player.moveAmount, ROOM_SIZE)
			end
			if rightMotion ~= 0 then
				worldManager.tryMovePlayer(world, rightMotion * world.player.moveAmount, 0, ROOM_SIZE)
			end

			if io.isKeyJustPressed("Z") then
				worldManager.undoLastMove(world, ROOM_SIZE)
			end
		end
	else
		local newPos = world.player.position + vec.V2(rightMotion, upMotion)
		worldManager.moveEntity(world, world.player, newPos)
	end

	graphics.clear(vec4.BLACK)
	graphics.withTransformation({ translation = gameCamera.position, scale = factor:scale(gameCamera.zoom) }, function()
		graphics.withTransformation({ translation = offset }, function()
			worldManager.draw(world, deltaTime)
		end)
	end)

	if world.readingSign then
		sign.drawDialog(world, signManager)
	end
end
