local Debug = require("@vectarine/debug")
local Event = require("@vectarine/event")
local Resources = require("@vectarine/resources")
local Vec = require("@vectarine/vec")

local module = {}

export type Entity = {
	type: string,
	position: Vec.Vec2,
	health: number,
	isHealthShown: boolean,
	attackDamage: number,
}

local _, utils = Resources.loadScript("scripts/utils.luau", require("utils.luau"))
local entities: { Entity } = utils.initFromGlobal({}, "entities")

local entitySpawnRate = 1.0
local timeSinceLastSpawn = 0.0

local function spawnEntity(entityType: string, position: Vec.Vec2)
	local entity: Entity = {
		type = entityType,
		position = position,
		health = 100,
		isHealthShown = true,
		attackDamage = 10,
	}
	if #entities >= 100 then
		Debug.print("Maximum number of entities reached. Cannot spawn more.")
		return entity
	end
	table.insert(entities, entity)
	return entity
end

function module.entityBehavior()
	-- Entities are attracted by the player, but there is a small spring force
	-- that make them repeal from one another.
	local playerPos: Vec.Vec2 = _G.player.position
	local delta: number = _G.deltaTime
	local entitySpeed = 0.5
	for i, entity in pairs(entities) do
		local moveDir = playerPos - entity.position
		if moveDir:length() > 0.01 then
			moveDir = moveDir:scale(delta * entitySpeed / moveDir:length())
			entity.position = entity.position + moveDir
		end

		-- Repulsion from other entities
		for j, otherEntity in pairs(entities) do
			if i ~= j then
				local toOther = otherEntity.position - entity.position
				local dist = toOther:length()
				if dist < 1.0 and dist > 0.0 then
					local repelForce = toOther:scale(-delta * entitySpeed / (dist * dist))
					entity.position = entity.position + repelForce
				end
			end
		end

		if entity.health <= 0 then
			entity.isHealthShown = false
			table.remove(entities, i)
		end
	end

	-- Spawn new entities over time
	timeSinceLastSpawn = timeSinceLastSpawn + delta
	if timeSinceLastSpawn >= 1 / entitySpawnRate then
		timeSinceLastSpawn = 0
		spawnEntity("slime", Vec.V2(math.random(0, 10), math.random(0, 10)))
	end
end

function module.getEntities(): { [number]: any }
	return entities
end

Event.getConsoleCommandEvent():on(function(command)
	if command == "spawn" then
		local newEntity = spawnEntity("slime", Vec.V2(math.random(0, 10), math.random(0, 10)))
		Debug.print(
			"Spawned entity of type ",
			newEntity.type,
			" at position (",
			newEntity.position.x,
			", ",
			newEntity.position.y,
			")"
		)
	end
end)

return module
