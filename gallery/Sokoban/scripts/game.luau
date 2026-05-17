--[[
A Sokoban game with one big world that you can explore.

You can edit the level inside textures/level.tmx using Tiled and see changes live.

- There is a special mecanic using red and blue potions
- You can push multiple crates at once
- You have infinite undo

The assets come from https://kenney.nl/assets/tiny-dungeon with a few tweaks.
]]

local Camera = require("@vectarine/camera")
local Debug = require("@vectarine/debug")
local Event = require("@vectarine/event")
local Graphics = require("@vectarine/graphics")
local Io = require("@vectarine/io")
local Loader = require("@vectarine/loader")
local Persist = require("@vectarine/persist")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")

local WorldRes, World = Loader.loadScript("scripts/world.luau", require("./world"))
local WorldTypes = require("./world")

local SignRes, Sign = Loader.loadScript("scripts/sign.luau", require("./sign"))
local SignTypes = require("./sign")

local tiles_tiles = Loader.loadTileset("textures/tileset.tsx")
local tiles_image = Loader.loadImage("textures/tileset.png", false)
local level = Loader.loadTilemap("textures/level.tmx")
local tiles = tiles_image:withTileset(tiles_tiles)

-- Controls how the camera moves. The world is divided into parts of size 20x15 and the camera moves between parts.
local ROOM_SIZE = Vec.V2(15, 10)

local areResourcesLoaded = Persist.onReload({ loaded = false }, "areResourcesLoaded")

local camera = Persist.onReload(Camera.new(), "camera")
local ZOOM_LEVEL = 0.12 -- This factor is choosen so that you can see the current room + about 1.5 tile of the surrounding ones

local world: WorldTypes.IWorld
local signManager: SignTypes.SignManager

-- You can explore the level freely by setting noclip to true from the watcher.
local debugging = Persist.onReload({
	noclip = false,
}, "debugging")

function Init()
	world = World.createInfiniteWorld(level, tiles, tiles_tiles)
	signManager = Sign.create()
	camera.position = -ROOM_SIZE:scale(0.5)
end

if areResourcesLoaded.loaded then
	Init()
end

Event.getConsoleCommandEvent():on(function(command: string)
	if command == "entities" then
		World.reloadEntities(world, level, tiles)
	else
		Debug.print("Unknown command: " .. command)
		Debug.print("Use entities to reset entity position from the resources")
	end
end)

function Update(deltaTime: number)
	if Graphics.drawSplashScreenIfNeeded({
		tiles_tiles,
		tiles_image,
		WorldRes,
		SignRes,
	}) then
		return
	end
	if not areResourcesLoaded.loaded then
		areResourcesLoaded.loaded = true
		Init()
		-- We need to wait one frame for init to finish.
		Graphics.drawSplashScreen("Initializing.")
		return
	end

	local windowSize = Io.getWindowSize()
	local yScale = windowSize.x / windowSize.y
	local factor = Vec.V2(ZOOM_LEVEL, -ZOOM_LEVEL * yScale)
	local offset = Vec.V2(-1, 1)

	local roomCenter = (world.player.position / ROOM_SIZE):floor() * ROOM_SIZE + ROOM_SIZE:scale(0.5) - Vec.V2(1, 0)
	camera:moveTowards(-roomCenter, 0.05)

	-- We don't allow diagonal movement.
	-- It could be a cool gameplay concept, but it is slightly hard to press the 2 keys at the same time.
	local upMotion = Io.isKeyJustPressed("Up") and -1 or Io.isKeyJustPressed("Down") and 1 or 0
	local rightMotion = Io.isKeyJustPressed("Right") and 1 or Io.isKeyJustPressed("Left") and -1 or 0
	if not debugging.noclip then
		-- You cannot move while reading a sign, it is rude.
		if not world.readingSign then
			if upMotion ~= 0 then
				World.tryMovePlayer(world, 0, upMotion * world.player.moveAmount, ROOM_SIZE)
			end
			if rightMotion ~= 0 then
				World.tryMovePlayer(world, rightMotion * world.player.moveAmount, 0, ROOM_SIZE)
			end

			if Io.isKeyJustPressed("Z") then
				World.undoLastMove(world, ROOM_SIZE)
			end
		end
	else
		local newPos = world.player.position + Vec.V2(rightMotion, upMotion)
		World.moveEntity(world, world.player, newPos)
	end

	Graphics.clear(Vec4.BLACK)
	Graphics.withTransformation({ translation = camera.position, scale = factor:scale(camera.zoom) }, function()
		Graphics.withTransformation({ translation = offset }, function()
			World.draw(world, deltaTime)
		end)
	end)

	if world.readingSign then
		Sign.drawDialog(world, signManager)
	end
end
