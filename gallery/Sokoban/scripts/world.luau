local Debug = require("@vectarine/debug")
local Event = require("@vectarine/event")
local Fastlist = require("@vectarine/fastlist")
local Graphics = require("@vectarine/graphics")
local Image = require("@vectarine/image")
local Io = require("@vectarine/io")
local Physics = require("@vectarine/physics")
local Tile = require("@vectarine/tile")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")

local module = {}

-- local FOREGROUND_LAYER = 2
local ENTITY_LAYER = 2
local TARGET_LAYER = 1
local GROUND_LAYER = 0
local LEVEL_SIZE = 16

local BSIGN = 68

local tileNames = {
	[60] = "goal",
	[61] = "target",
	[63] = "crate",
	[67] = "sign",
	[74] = "anvil",
	[82] = "barrel",
	[84] = "wizard",
	[86] = "blacksmith",
	[87] = "viking",
	[89] = "chest",
	[97] = "knight",
	[120] = "bat",
	[122] = "spider",
	[123] = "rat",
	[76] = "gate",
	[77] = "gate",
	[78] = "gate",
	[99] = "princess",
	[127] = "r_potion",
	[128] = "b_potion",
}

local pushable = { "crate", "barrel", "rat", "spider", "bat", "blacksmith", "viking", "wizard" }
local interactable = { "r_potion", "b_potion" }

local GROUND_TILES = { 9, 30, 36, 37, 38, 39, 42, 48, 49, 50, 51, 52, 53 }

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

export type Entity = {
	position: Vec.Vec2,
	kind: string,
	tile: number,

	-- For all entites
	shownPosition: Vec.Vec2,
	jumpAnimation: number,
	firstTransactionIndex: number,
	lastTransactionIndex: number,
	-- For moving animation
	transactions: {
		[number]: {
			origin: Vec.Vec2,
			target: Vec.Vec2,
			progress: number,
		},
	},

	-- For player and crates
	moveAmount: number,

	-- For gates
	isOpen: boolean,
	shownOpen: boolean,
	firstOpenTransactionIndex: number,
	lastOpenTransactionIndex: number,
	openTransaction: {
		[number]: {
			finalState: boolean,
			progress: number,
		},
	},
}

export type UndoRedo = {
	currentMoveTransaction: number,
	lastMoveTransaction: number,
	moveTransactions: {
		[number]: {
			playerPositionBefore: Vec.Vec2,
			playerPositionAfterwards: Vec.Vec2,

			movedObjects: { { entity: Entity, from: Vec.Vec2, to: Vec.Vec2 } },
			changedMoveAmountObjects: { { entity: Entity, moveAmountBefore: number, moveAmountAfter: number } },
		},
	},
}

--- Represents a 16x16 area whose top-left corner is at (x,y)
--- Numbers stored in the Chunk should be multiples of 16.
export type Chunk = Vec.Vec2

export type IWorld = {
	level: Tile.TilemapResource,
	tiles: Image.ImageWithTileset,
	tileset: Tile.TilesetResource,
	entities: { [string]: { Entity } },
	player: Entity,
	readingSign: boolean,
	undoRedo: UndoRedo,
}

local function chunkKey(c: Chunk): string
	return c.x .. "," .. c.y
end

local function chunkFromPos(pos: Vec.Vec2): Chunk
	return Vec.V2(math.floor(pos.x / LEVEL_SIZE) * LEVEL_SIZE, math.floor(pos.y / LEVEL_SIZE) * LEVEL_SIZE)
end

-- Given a position, find the corresponding room (20x15 area) and checks if all targets are covered.
function module.isRoomCleared(world: IWorld, pos: Vec.Vec2, roomSize: Vec.Vec2): boolean
	local roomPos = (pos / roomSize):floor() * roomSize

	local isExposedTarget = false

	world.level:getTilemapPart(
		TARGET_LAYER,
		roomPos.x,
		roomPos.y,
		roomPos.x + roomSize.x,
		roomPos.y + roomSize.y,
		function(tileId: number, x: number, y: number)
			if tileId ~= 0 then
				for _, entity in module.entityAt(world, x, y) do
					if entity.kind ~= "gate" then
						return -- this target is covered by a non-gate entity
					end
				end
				isExposedTarget = true
			end
		end
	)

	return not isExposedTarget
end

-- Set the gates in the given room the open/closed state.
function module.toggleGates(world: IWorld, pos: Vec.Vec2, roomSize: Vec.Vec2, isOpen: boolean)
	local roomPos = (pos / roomSize):floor() * roomSize
	for x = roomPos.x, roomPos.x + roomSize.x - 1, 1 do
		for y = roomPos.y, roomPos.y + roomSize.y - 1, 1 do
			for _, entity in module.entityAt(world, x, y) do
				if entity.kind == "gate" and entity.isOpen ~= isOpen then
					entity.isOpen = isOpen
					entity.openTransaction[entity.lastOpenTransactionIndex + 1] = {
						finalState = isOpen,
						progress = 0,
					}
					entity.lastOpenTransactionIndex += 1
				end
			end
		end
	end
end

function module.extractEntitiesFromLevel(level: Tile.TilemapResource, entityLayer: number): { [string]: { Entity } }
	local entities = {}

	-- Expand the extraction here as needed.
	level:getTilemapPart(ENTITY_LAYER, 0, 0, 100, 100, function(tileId: number, x: number, y: number)
		local tileType = tileNames[tileId]
		if not tileType then
			return
		end
		local position = Vec.V2(x, y)
		local key = chunkKey(position)
		if not entities[key] then
			entities[key] = {}
		end
		table.insert(entities[key], {
			position = Vec.V2(x, y),
			kind = tileType,
			tile = tileId,
			shownPosition = Vec.V2(x, y),
			jumpAnimation = math.random(0, 20),
			firstTransactionIndex = 1,
			lastTransactionIndex = 0,
			transactions = {},
			--
			moveAmount = 1,
			-- Gates
			isOpen = false,
			shownOpen = false,
			firstOpenTransactionIndex = 1,
			lastOpenTransactionIndex = 0,
			openTransaction = {},
		})
	end)

	return entities
end

function module.undoLastMove(w: IWorld, roomSize: Vec.Vec2)
	local ur = w.undoRedo
	if ur.currentMoveTransaction == 0 then
		return
	end

	local transaction = ur.moveTransactions[ur.currentMoveTransaction]

	-- Move player back first to free up the position where pushed objects will be restored
	-- We know the previous position of the player is free, otherwise, the move should have not been possible.
	w.player.position = transaction.playerPositionAfterwards
	module.moveEntity(w, w.player, transaction.playerPositionBefore, false)

	-- Reverse moved objects in reverse order
	for i = #transaction.movedObjects, 1, -1 do
		local moved = transaction.movedObjects[i]
		moved.entity.position = moved.to -- Ensure the entity is at the expected position before-hand.
		module.moveEntity(w, moved.entity, moved.from, false)
	end

	-- Reverse move amount changes
	for _, change in transaction.changedMoveAmountObjects do
		change.entity.moveAmount = change.moveAmountBefore
	end

	ur.currentMoveTransaction -= 1

	local newPos = w.player.position
	local isRoomCleared = module.isRoomCleared(w, newPos, roomSize)
	module.toggleGates(w, newPos, roomSize, isRoomCleared)
end

function module.reloadEntities(w: IWorld, level: Tile.TilemapResource, tiles: Image.ImageWithTileset)
	w.entities = module.extractEntitiesFromLevel(level, ENTITY_LAYER)
	w.entities[chunkKey(w.player.position)] = { w.player }
end

function module.createInfiniteWorld(
	level: Tile.TilemapResource,
	tiles: Image.ImageWithTileset,
	tileset: Tile.TilesetResource
): IWorld
	local entities = module.extractEntitiesFromLevel(level, ENTITY_LAYER)
	local player = nil
	for _, entityList in entities do
		-- By default, the player is the first knight.
		for _, entity in entityList do
			if entity.kind == "knight" then
				player = entity
				break
			end
		end
	end

	return {
		level = level,
		tiles = tiles,
		tileset = tileset,
		entities = entities,
		player = player,
		readingSign = false,
		undoRedo = {
			currentMoveTransaction = 0,
			lastMoveTransaction = 0,
			moveTransactions = {},
		},
		playerMoveAmount = 1,
	}
end

local function tableContains(t: { number }, value: number): boolean
	for _, v in t do
		if v == value then
			return true
		end
	end
	return false
end

--- Returns true if the tile at (x, y) is walkable (i.e. is a ground tile on the background layer).
function module.isWalkable(w: IWorld, x: number, y: number): boolean
	local tileId = w.level:getTile(GROUND_LAYER, x, y)
	return tableContains(GROUND_TILES, tileId)
end

function module.entityAt(w: IWorld, x: number, y: number): { Entity }
	return w.entities[chunkKey(Vec.V2(x, y))] or {}
end

--- Move the entity to a new position without checking anything.
function module.moveEntity(w: IWorld, entity: Entity, newPos: Vec.Vec2, animate: boolean?)
	local oldPos = entity.position
	local oldList = w.entities[chunkKey(oldPos)]
	if oldList then
		for i, e in oldList do
			if e == entity then
				table.remove(oldList, i)
				break
			end
		end
		if #oldList == 0 then
			w.entities[chunkKey(oldPos)] = nil
		end
	end

	entity.position = newPos
	local newKey = chunkKey(newPos)
	if not w.entities[newKey] then
		w.entities[newKey] = {}
	end
	table.insert(w.entities[newKey], entity)

	if animate then
		entity.transactions[entity.lastTransactionIndex + 1] = {
			origin = oldPos,
			target = newPos,
			progress = 0,
		}
		entity.lastTransactionIndex = entity.lastTransactionIndex + 1
	end
end

local function hasPendingTransactions(entity: Entity): boolean
	return entity.transactions[entity.firstTransactionIndex] ~= nil
end

--- Recursively collect all entities that need to be pushed starting from pos in the given direction.
--- Each entity moves by its own moveAmount. If moveAmount >= 2, the entity phases through walls and crates.
--- Entities at the destination get pushed recursively.
--- Returns false if the push is blocked.
local function collectPushChain(
	w: IWorld,
	pos: Vec.Vec2,
	dir: Vec.Vec2,
	chain: { { entity: Entity, from: Vec.Vec2, to: Vec.Vec2 } }
): boolean
	local pushablesHere = {}
	for _, entity in module.entityAt(w, pos.x, pos.y) do
		if entity.kind == "gate" then
			if not entity.isOpen then
				return false
			end
		elseif table.find(pushable, entity.kind) ~= nil then
			table.insert(pushablesHere, entity)
		elseif table.find(interactable, entity.kind) ~= nil then
			-- potions coexist, don't block
		else
			return false
		end
	end

	for _, entity in pushablesHere do
		local dest = pos + dir:scale(entity.moveAmount)

		-- Destination must always be walkable (intermediate tiles are ignored)
		if not module.isWalkable(w, dest.x, dest.y) then
			return false
		end

		-- Recursively push whatever is at the destination
		if not collectPushChain(w, dest, dir, chain) then
			return false
		end

		table.insert(chain, { entity = entity, from = pos, to = dest })
	end

	return true
end

--- Try to move the player by (dx, dy).
--- Handles undo, redo, pushing and other consequences of the move.
--- Returns true if the move succeeded.
function module.tryMovePlayer(w: IWorld, dx: number, dy: number, roomSize: Vec.Vec2): boolean
	local dir = Vec.V2(dx, dy)
	local newPos = w.player.position + dir
	if not module.isWalkable(w, newPos.x, newPos.y) then
		return false
	end

	local movedObjects = {}
	local moveAmountTransactions = {}

	-- Collect push chain (recursive; returns false if blocked)
	local chain = {}
	if not collectPushChain(w, newPos, dir:sign(), chain) then
		return false
	end

	-- Apply interactable effects to player at destination
	local playerMoveAmountBefore = w.player.moveAmount
	for _, entity in module.entityAt(w, newPos.x, newPos.y) do
		if entity.kind == "r_potion" then
			table.insert(moveAmountTransactions, {
				entity = w.player,
				moveAmountBefore = playerMoveAmountBefore,
				moveAmountAfter = 2,
			})
			w.player.moveAmount = 2
		elseif entity.kind == "b_potion" then
			table.insert(moveAmountTransactions, {
				entity = w.player,
				moveAmountBefore = playerMoveAmountBefore,
				moveAmountAfter = 1,
			})
			w.player.moveAmount = 1
		end
	end

	-- Execute chain push (chain is ordered deepest-first, so execute in order)
	for _, entry in chain do
		table.insert(movedObjects, { entity = entry.entity, from = entry.from, to = entry.to })
		module.moveEntity(w, entry.entity, entry.to, true)
	end

	-- Apply potion effects to pushed entities at their new positions
	for _, entry in chain do
		for _, entity in module.entityAt(w, entry.to.x, entry.to.y) do
			if entity ~= entry.entity and table.find(interactable, entity.kind) ~= nil then
				local newAmount = entity.kind == "r_potion" and 2 or entity.kind == "b_potion" and 1 or nil
				if newAmount ~= nil and entry.entity.moveAmount ~= newAmount then
					table.insert(moveAmountTransactions, {
						entity = entry.entity,
						moveAmountBefore = entry.entity.moveAmount,
						moveAmountAfter = newAmount,
					})
					entry.entity.moveAmount = newAmount
				end
			end
		end
	end

	local playerFrom = w.player.position
	module.moveEntity(w, w.player, newPos, true)

	-- Resolve gate state.
	local isRoomCleared = module.isRoomCleared(w, newPos, roomSize)
	module.toggleGates(w, newPos, roomSize, isRoomCleared)

	-- Record transaction, discarding any redo history beyond current point
	local ur = w.undoRedo
	ur.currentMoveTransaction += 1
	ur.lastMoveTransaction = ur.currentMoveTransaction
	ur.moveTransactions[ur.currentMoveTransaction] = {
		playerPositionBefore = playerFrom,
		playerPositionAfterwards = newPos,
		movedObjects = movedObjects,
		changedMoveAmountObjects = moveAmountTransactions,
	}

	return true
end

function module.drawBSign(w: IWorld, deltaTime: number, signEntity: Entity)
	if not w.readingSign then
		if Io.isKeyJustPressed("B") then
			w.readingSign = true
		end
		local animation = math.sin(w.player.jumpAnimation * 0.6 + 3.1415) * 0.1
		w.tiles:drawTile(BSIGN, signEntity.position + Vec.V2(0, -2 + animation), Vec.V2(1, 1))
	end
end

--- Draw the visible part of the level. Call inside the world-space transformation.
function module.draw(w: IWorld, deltaTime: number)
	local p1 = Graphics.inverseTransform(Vec.V2(-1, -1))
	local p2 = Graphics.inverseTransform(Vec.V2(1, 1))
	p1 = p1:scale(1 / LEVEL_SIZE)
	p2 = p2:scale(1 / LEVEL_SIZE)
	local low = Vec.V2(math.min(p1.x, p2.x), math.min(p1.y, p2.y)):floor()
	local high = Vec.V2(math.max(p1.x, p2.x), math.max(p1.y, p2.y)):ceil()

	local layers = {
		{ index = GROUND_LAYER, tint = Vec4.WHITE },
		{ index = TARGET_LAYER, tint = Vec4.WHITE },
	}

	for x = low.x - 1, high.x + 1, 1 do
		for y = low.y - 1, high.y + 1, 1 do
			for _, layer in layers do
				local lvl: Tile.TilemapResource = w.level :: Tile.TilemapResource

				lvl:getTilemapPart(
					layer.index,
					x * LEVEL_SIZE,
					y * LEVEL_SIZE,
					(x + 1) * LEVEL_SIZE,
					(y + 1) * LEVEL_SIZE,
					function(tileId: number, x: number, y: number)
						w.tiles:drawTile(tileId, Vec.V2(x, y), Vec.V2(1, -1), layer.tint)
					end
				)
			end
		end
	end

	for _, entityList in w.entities do
		for _, entity in entityList do
			-- Resolve animation transactions
			local currentTransaction = entity.transactions[entity.firstTransactionIndex]
			if currentTransaction ~= nil then
				currentTransaction.progress += deltaTime * 8
				entity.shownPosition =
					currentTransaction.origin:lerp(currentTransaction.target, currentTransaction.progress)
				if currentTransaction.progress >= 1 then
					entity.shownPosition = currentTransaction.target
					entity.transactions[entity.firstTransactionIndex] = nil -- GC
					entity.firstTransactionIndex += 1
				end
			else
				-- Sync
				entity.shownPosition = entity.position
			end

			-- Resolve gate animation
			if entity.kind == "gate" then
				local currentOpenTransaction = entity.openTransaction[entity.firstOpenTransactionIndex]
				if currentOpenTransaction ~= nil then
					currentOpenTransaction.progress += deltaTime * 1
					if currentOpenTransaction.progress >= 1 then
						currentOpenTransaction.progress = 1
						entity.openTransaction[entity.firstOpenTransactionIndex] = nil -- GC
						entity.firstOpenTransactionIndex += 1
					end
				else
					entity.shownOpen = entity.isOpen
				end
			end

			if entity.kind == "sign" then
				local p = entity.position
				local belowSign = w.entities[chunkKey(p + Vec.V2(0, 1))]
				if belowSign then
					local isPlayerBelow = false
					for _, e in belowSign do
						if e == w.player then
							isPlayerBelow = true
							break
						end
					end
					if isPlayerBelow then
						module.drawBSign(w, deltaTime, entity)
					end
				end
			end

			if entity.tile then
				local targetTileId = w.level:getTile(TARGET_LAYER, entity.position.x, entity.position.y)
				local tint = targetTileId ~= 0 and Vec4.GRAY or Vec4.WHITE

				local jumpOffset
				if entity.kind == "knight" or entity.kind == "princess" then
					if not hasPendingTransactions(entity) then
						entity.jumpAnimation += deltaTime * 10
						entity.jumpAnimation = entity.jumpAnimation % (math.pi * 2)
					else
						entity.jumpAnimation = math.max(0, entity.jumpAnimation - deltaTime)
					end
					jumpOffset = math.sin(entity.jumpAnimation) * 0.1 - 0.1
				else
					jumpOffset = 0
				end

				if entity.kind == "gate" then
					-- If there are no transaction, we only draw closed gates
					if entity.openTransaction[entity.firstOpenTransactionIndex] == nil then
						if not entity.isOpen then
							w.tiles:drawTile(
								entity.tile,
								entity.shownPosition + Vec.V2(0, jumpOffset),
								Vec.V2(1, -1),
								tint
							)
						end
					else
						local currentOpenTransaction = entity.openTransaction[entity.firstOpenTransactionIndex]
						local finalState = currentOpenTransaction.finalState

						local p = finalState and currentOpenTransaction.progress or 1 - currentOpenTransaction.progress

						Graphics.drawWithMask(function()
							w.tiles:drawTile(entity.tile, entity.shownPosition + Vec.V2(0, p), Vec.V2(1, -1), tint)
						end, function()
							w.tiles:drawTile(
								entity.tile,
								entity.shownPosition + Vec.V2(0, jumpOffset),
								Vec.V2(1, -1),
								tint
							)
						end)
					end
				else
					w.tiles:drawTile(entity.tile, entity.shownPosition + Vec.V2(0, jumpOffset), Vec.V2(1, -1), tint)
				end
			end
		end
	end
end

return module
