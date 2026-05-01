local Fastlist = require("@vectarine/fastlist")
local Graphics = require("@vectarine/graphics")
local Image = require("@vectarine/image")
local Physics = require("@vectarine/physics")
local Tile = require("@vectarine/tile")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")

-- A module that handles infinite 2D worlds by streaming chunks.
-- It does physics and graphics.
local module = {}

local CHUNK_RADIUS = 1 -- How many chunks around the player to keep active
local CHUNK_EXPIRE_TIME = 10 -- Seconds before an out-of-range chunk is removed from the pool.

type GroundChunk = {
	object: Physics.Object2,
	lastNearTime: number,
	cx: number,
	cy: number,
}

export type Layer = {
	index: number,
	tint: Vec4.Vec4,
}

export type IWorld = {
	world: Physics.World2,
	level: Tile.Tilemap,
	collisionLayer: number,
	solidBlocks: { number },
	chunkSize: number,
	groundChunks: { [string]: GroundChunk },
	tiles: Image.ImageWithTileset,
	tileset: Tile.TilesetResource,
	layers: { Layer },
}

local function chunkKey(cx: number, cy: number): string
	return cx .. "," .. cy
end

function module.createInfiniteWorld(
	world: Physics.World2,
	level: Tile.Tilemap,
	collisionLayer: number,
	solidBlocks: { number },
	chunkSize: number,
	tiles: Image.ImageWithTileset,
	tileset: Tile.TilesetResource,
	layers: { Layer }
): IWorld
	return {
		world = world,
		level = level,
		collisionLayer = collisionLayer,
		solidBlocks = solidBlocks,
		chunkSize = chunkSize,
		groundChunks = {},
		tiles = tiles,
		tileset = tileset,
		layers = layers,
	}
end

local function createGroundChunk(w: IWorld, cx: number, cy: number)
	local key = chunkKey(cx, cy)
	if w.groundChunks[key] then
		return
	end

	local low = Vec.V2(cx * w.chunkSize, cy * w.chunkSize)
	local high = Vec.V2((cx + 1) * w.chunkSize, (cy + 1) * w.chunkSize)

	local collider = Physics.newVoxelCollider(Vec.V2(1, 1), w.level, w.collisionLayer, low, high, w.solidBlocks)
	local obj = w.world:createObject(Vec.V2(0, 0), 1.0, collider, { "ground" }, "static")
	obj:setFriction(0.0)

	w.groundChunks[key] = { object = obj, lastNearTime = 0, cx = cx, cy = cy }
end

--- Update streaming ground chunks around the player position. Call every frame.
function module.update(w: IWorld, playerPos: Vec.Vec2, deltaTime: number)
	local playerChunkX = math.floor(playerPos.x / w.chunkSize)
	local playerChunkY = math.floor(playerPos.y / w.chunkSize)

	-- Create chunks around the player
	for cx = playerChunkX - CHUNK_RADIUS, playerChunkX + CHUNK_RADIUS do
		for cy = playerChunkY - CHUNK_RADIUS, playerChunkY + CHUNK_RADIUS do
			local key = chunkKey(cx, cy)
			if w.groundChunks[key] then
				w.groundChunks[key].lastNearTime = 0
			else
				createGroundChunk(w, cx, cy)
			end
		end
	end

	-- Remove chunks that have been far away for too long
	for key, chunk in w.groundChunks do
		local chunkCenter = Vec.V2((chunk.cx + 0.5) * w.chunkSize, (chunk.cy + 0.5) * w.chunkSize)
		local distanceToPlayer = (chunkCenter - playerPos):length()
		if distanceToPlayer > CHUNK_RADIUS * w.chunkSize * 2 then
			chunk.lastNearTime = chunk.lastNearTime + deltaTime
		end
		if chunk.lastNearTime > CHUNK_EXPIRE_TIME then
			w.world:removeObject(chunk.object :: Physics.Object2)
			w.groundChunks[key] = nil
		end
	end
end

--- Draw colliders for debug. Call inside the world-space transformation.
function module.drawColliders(w: IWorld)
	for _, chunk in w.groundChunks do
		local groundPoints = chunk.object:getPoints()
		if #groundPoints > 0 then
			local basePoints = Fastlist.fromTable(groundPoints)
			local points = basePoints:unweave(4)
			table.insert(points, Fastlist.fromValue(Vec.V2(1, 0), #points[1]))
			table.insert(points, Fastlist.fromValue(Vec.V2(0, 0.5), #points[1]))
			local groundColliderDrawable = Fastlist.fromTable({}):weave(points)
			groundColliderDrawable:drawQuads()
		end
	end
end

--- Draw the visible part of the level. Call inside the world-space transformation.
function module.draw(w: IWorld)
	local p1 = Graphics.inverseTransform(Vec.V2(-1, -1))
	local p2 = Graphics.inverseTransform(Vec.V2(1, 1))
	local low1 = p1:scale(1 / w.chunkSize):floor()
	local low2 = p2:scale(1 / w.chunkSize):ceil()

	for x = low1.x - 1, low2.x + 1, 1 do
		for y = low1.y - 1, low2.y + 1, 1 do
			for _, layer in w.layers do
				local lvl: Tile.TilemapResource = w.level :: Tile.TilemapResource
				lvl:getTilemapPart(
					layer.index,
					x * w.chunkSize,
					y * w.chunkSize,
					(x + 1) * w.chunkSize,
					(y + 1) * w.chunkSize,
					function(tileId: number, x: number, y: number)
						if tileId == 0 then
							return
						end
						local tileType = w.tileset:getTile(tileId)
						if not tileType then
							return
						end
						local tint = layer.tint
						if tileType.type == "coin" then
							tint = Vec4.YELLOW
						end
						w.tiles:drawTile(tileId, Vec.V2(x, y + 1), Vec.V2(1, -1), tint)
					end
				)
			end
		end
	end
end

return module
