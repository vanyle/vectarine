local Debug = require("@vectarine/debug")
local Persist = require("@vectarine/persist")
local Resource = require("@vectarine/resources")
local Vec = require("@vectarine/vec")
local V2 = Vec.V2

local pumpkin_animation = {}
for i = 1, 6 do
	local img = Resource.loadImage("textures/PumpkinMan/PumpkinMan" .. tostring(i) .. ".png", false)
	table.insert(pumpkin_animation, img)
end

-- Entity type definitions
export type Entity = {
	position: Vec.Vec2,
	size: number,
	ratio: number,

	hitbox_size: Vec.Vec2,
	hitbox_offset: Vec.Vec2?,

	text: string?,
}

export type MovingEntity = Entity & {
	speed: number,
}

export type Monster = MovingEntity & AnimatedEntity & {
	stamina: number,
	max_stamina: number,
	state: "chasing" | "stalking" | "wandering",
	target: PathMarker?,
	patrol_route: { PathMarker }?,
}

export type AnimatedEntity = Entity & {
	animation_idx: number,
	frame_delay: number,
	anim_time: number,
	animation_images: {},
}

export type DestructibleEntity = Entity & {
	destroyed: boolean,
}

export type PathMarker = {
	position: Vec.Vec2,
	next_marker: PathMarker?,
}

export type Player = MovingEntity & AnimatedEntity & {
	fuel: number,
	max_fuel: number,
	fuel_drain_rate: number,
	fuel_death_threshold: number,
}

local path_marker: { PathMarker } = {
	{
		position = V2(20, 1),
	},
	{
		position = V2(22, 10),
	},
	{
		position = V2(21, 17),
	},
	{
		position = V2(15, 20),
	},
	{
		position = V2(14, 28),
	},
	{
		position = V2(6, 27),
	},
	{
		position = V2(3, 20),
	},
	{
		position = V2(3, 16),
	},
	{
		position = V2(1, 14),
	},
	{
		position = V2(1, 12),
	},
	{
		position = V2(3, 11),
	},
	{
		position = V2(3, 4),
	},
	{
		position = V2(8, 1),
	},
	{
		position = V2(17, 2),
	},
}

for i = 1, #path_marker - 1 do
	path_marker[i].next_marker = path_marker[i + 1]
end
path_marker[#path_marker].next_marker = path_marker[1]

function create_pumpkin(pumpkin_position: Vec.Vec2, pumpkin_list: { AnimatedEntity & MovingEntity })
	-- Use provided route or default to shared route
	local patrol_route = path_marker

	local new_pumpkin: Monster = {
		state = "wandering" :: "wandering",
		stamina = 1.0,
		max_stamina = 4.0, -- duration (in seconds) of burst speed
		target = nil,
		patrol_route = patrol_route, -- Store reference to this pumpkin's route
		position = pumpkin_position,
		size = 2.0,
		ratio = 10 / 6,

		hitbox_size = V2(1.6, 1.6),

		speed = 0.5,

		animation_idx = 0,
		animation_images = pumpkin_animation,
		anim_time = 0,
		frame_delay = 0.15,
	}
	table.insert(pumpkin_list, new_pumpkin)
end

function create_sign(sign_position: Vec.Vec2, sign_text: string, sign_list: { DestructibleEntity })
	table.insert(sign_list, {
		position = sign_position,
		size = 1.0,
		ratio = 1,
		text = sign_text,
		destroyed = false,
		time_to_die = 5.0, -- seconds
		hitbox_size = V2(1.0, 1.0),
	})
end

-- Entity lists
local pumpkin_list: { Monster } = Persist.onReload({}, "pumpkin_list")
local sign_list: { DestructibleEntity } = Persist.onReload({}, "sign_list")

function remove_entities()
	for i = #pumpkin_list, 1, -1 do
		table.remove(pumpkin_list, i)
	end
	for i = #sign_list, 1, -1 do
		table.remove(sign_list, i)
	end
end

function spawn_entities()
	-- Assign each pumpkin to its own patrol route
	create_pumpkin(V2(20, 2), pumpkin_list)
	create_pumpkin(V2(2, 10), pumpkin_list)
	create_pumpkin(V2(18, 23), pumpkin_list)
	create_sign(V2(11, 8), "watching", sign_list)
	create_sign(V2(27, 2), "directions", sign_list)
	create_sign(V2(28, 17), "hiding", sign_list)
	create_sign(V2(14, 20), "behind", sign_list)
	create_sign(V2(10, 30), "nightmare", sign_list)
	create_sign(V2(22, 31), "quiet", sign_list)
	create_sign(V2(6, 15), "water", sign_list)
	create_sign(V2(2, 22), "moving", sign_list)
	create_sign(V2(3, 7), "running", sign_list)
end

-----------------------------------------------------------------------------------------------------------------------------------
---------------------------------------------------- Level design data ------------------------------------------------------------
-----------------------------------------------------------------------------------------------------------------------------------

local grass_like = { "grass", "grass", "grass", "grass", "grass", "clover", "clover", "rose", "bush" }

local sprite_size = V2(1 / 8, 1 / 8)
local sprite_table = {
	path_end_l = V2(0, 7), -- dead end left
	path_s = V2(1, 7), -- straight 1
	path_i1 = V2(2, 7), -- intersection 1
	path_end_r = V2(3, 7), -- dead end right
	rose = V2(0, 6),
	path_end_b = V2(1, 6), -- dead end bottom
	path_s2 = V2(2, 6), -- straight 2
	grass = V2(3, 6),
	bush = V2(0, 5),
	path_t1 = V2(1, 5), -- turn 1
	path_i2 = V2(2, 5), -- intersection 2
	path_t2 = V2(3, 5), -- turn 2
	lake_bl = V2(0, 4), -- bottom left of lake
	lake_b = V2(1, 4), -- bottom of lake
	lake_br = V2(2, 4), -- bottom right of lake
	path_end_t = V2(3, 4), -- dead end top
	island_bl = V2(4, 4),
	island_b = V2(5, 4),
	island_br = V2(6, 4),
	lake_l = V2(0, 3), -- left of lake
	lake_c = V2(1, 3), -- center of lake
	lake_r = V2(2, 3), -- right of lake
	clover = V2(3, 3),
	island_l = V2(4, 3),
	island_c = V2(5, 3),
	island_r = V2(6, 3),
	lake_tl = V2(0, 2), -- top left of lake
	lake_t = V2(1, 2), -- top of lake
	lake_tr = V2(2, 2), -- top right of lake
	island_tl = V2(4, 2),
	island_t = V2(5, 2),
	island_tr = V2(6, 2),
}

local sprite_ids = {
	[1] = "grass",
	[2] = "path_end_l",
	[3] = "path_end_r",
	[4] = "path_s",
	[5] = "path_s2",
	[6] = "path_i1",
	[7] = "path_i2",
	[8] = "path_t1",
	[9] = "path_t2",
	[10] = "path_end_b",
	[11] = "path_end_t",
	[12] = "rose",
	[13] = "bush",
	[14] = "clover",
	[15] = "lake_bl",
	[16] = "lake_b",
	[17] = "lake_br",
	[18] = "lake_l",
	[19] = "lake_c",
	[20] = "lake_r",
	[21] = "lake_tl",
	[22] = "lake_t",
	[23] = "lake_tr",
	[24] = "island_bl",
	[25] = "island_b",
	[26] = "island_br",
	[27] = "island_l",
	[28] = "island_c",
	[29] = "island_r",
	[30] = "island_tl",
	[31] = "island_t",
	[32] = "island_tr",
}

local name_to_id = {}
for id, name in pairs(sprite_ids) do
	name_to_id[name] = id
end

-- 1 = grass
-- 2 = path
-- 3 = lake

-- Level layout grid
local level = {
	{ 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3 },
	{ 1, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 1, 1, 1, 3, 3, 3, 3, 3 },
	{ 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 3 },
	{ 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 3 },
	{ 1, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 1, 2, 1, 3, 3, 3, 3, 1, 1, 3 },
	{ 1, 1, 1, 1, 2, 1, 3, 3, 3, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 1, 2, 1, 3, 3, 3, 3, 3, 3, 3 },
	{ 1, 1, 1, 1, 2, 1, 3, 3, 3, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 1, 2, 1, 3, 3, 3, 3, 3, 3, 3 },
	{ 3, 3, 3, 1, 2, 1, 3, 3, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 3, 3, 3, 3, 3, 3 },
	{ 1, 1, 3, 1, 2, 1, 3, 3, 3, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 3, 3, 3, 3, 3, 3 },
	{ 1, 1, 3, 1, 2, 1, 3, 3, 3, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 3, 3, 3, 3, 3, 3 },
	{ 3, 3, 3, 1, 1, 1, 3, 3, 3, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 3, 3, 3, 3, 3, 3 },
	{ 1, 1, 1, 1, 1, 1, 3, 3, 3, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 3, 3, 3, 3, 3, 3 },
	{ 1, 1, 1, 1, 1, 3, 3, 3, 3, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 3, 3, 3 },
	{ 1, 1, 1, 1, 1, 3, 3, 3, 3, 1, 2, 1, 1, 1, 3, 3, 3, 3, 3, 3, 1, 2, 2, 2, 2, 2, 1, 1, 1, 3 },
	{ 1, 1, 1, 3, 3, 3, 3, 3, 3, 1, 2, 1, 1, 1, 3, 3, 3, 3, 3, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3 },
	{ 1, 2, 1, 1, 1, 1, 1, 1, 3, 1, 2, 1, 1, 3, 3, 3, 3, 3, 3, 3, 1, 2, 1, 3, 3, 3, 1, 1, 1, 1 },
	{ 1, 2, 2, 2, 2, 2, 2, 1, 3, 1, 2, 1, 1, 3, 3, 3, 3, 1, 1, 1, 1, 2, 1, 3, 3, 3, 1, 1, 1, 1 },
	{ 1, 1, 1, 2, 1, 1, 1, 1, 3, 1, 1, 1, 1, 3, 3, 3, 3, 1, 2, 1, 1, 2, 1, 3, 3, 3, 1, 1, 1, 1 },
	{ 3, 3, 1, 2, 1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 1, 1, 2, 1, 1, 2, 1, 3, 3, 3, 3, 1, 1, 1 },
	{ 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 3, 3, 3, 3, 3, 3 },
	{ 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 1, 2, 1, 1, 1, 1, 1, 1, 3, 3, 1, 1, 1 },
	{ 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3, 1, 1, 1, 1, 1, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 },
	{ 1, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 },
	{ 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3 },
	{ 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3 },
	{ 3, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3 },
	{ 3, 3, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3 },
	{ 3, 3, 3, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3 },
	{ 3, 3, 3, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3 },
	{ 3, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3 },
	{ 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3 },
	{ 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3 },
	{ 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3 },
	{ 3, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3 },
}

-- Process level to have the right edge rules
local level_copy = {}
for y = 1, #level do
	level_copy[y] = {}
	for x = 1, #level[y] do
		level_copy[y][x] = { level[y][x], 0 }
	end
end

for y = 1, #level do
	for x = 1, #level[y] do
		-- Determine which path sprite to use based on neighbors
		local up = (y > 1 and level[y - 1][x]) or 3
		local down = (y < #level and level[y + 1][x]) or 3
		local left = (x > 1 and level[y][x - 1]) or 3
		local right = (x < #level[y] and level[y][x + 1]) or 3
		local cell = level[y][x]
		if cell == 2 then
			-- Path rules
			if up == 2 and down == 2 and left ~= 2 and right ~= 2 then
				level_copy[y][x] = { name_to_id["path_s2"], 0 }
			elseif left == 2 and right == 2 and up ~= 2 and down ~= 2 then
				level_copy[y][x] = { name_to_id["path_s"], 0 }
			elseif up == 2 and right ~= 2 and down ~= 2 and left ~= 2 then
				level_copy[y][x] = { name_to_id["path_end_t"], 0 }
			elseif down == 2 and right ~= 2 and up ~= 2 and left ~= 2 then
				level_copy[y][x] = { name_to_id["path_end_b"], 0 }
			elseif left == 2 and up ~= 2 and down ~= 2 and right ~= 2 then
				level_copy[y][x] = { name_to_id["path_end_r"], 0 }
			elseif right == 2 and up ~= 2 and down ~= 2 and left ~= 2 then
				level_copy[y][x] = { name_to_id["path_end_l"], 0 }
			elseif up ~= 2 and down == 2 and left == 2 and right == 2 then
				level_copy[y][x] = { name_to_id["path_i1"], 0 }
			elseif up == 2 and down ~= 2 and left == 2 and right == 2 then
				level_copy[y][x] = { name_to_id["path_i2"], 0 }
			elseif up == 2 and down == 2 and left ~= 2 and right == 2 then
				level_copy[y][x] = { name_to_id["path_i2"], 3 }
			elseif up == 2 and down ~= 2 and left ~= 2 and right == 2 then
				level_copy[y][x] = { name_to_id["path_t1"], 0 }
			elseif up ~= 2 and down == 2 and left == 2 and right ~= 2 then
				level_copy[y][x] = { name_to_id["path_t2"], 0 }
			elseif up == 2 and down ~= 2 and left == 2 and right ~= 2 then
				level_copy[y][x] = { name_to_id["path_t1"], 1 }
			elseif up ~= 2 and down == 2 and left ~= 2 and right == 2 then
				level_copy[y][x] = { name_to_id["path_t2"], 1 }
			else
				level_copy[y][x] = { name_to_id["path_s2"], 0 }
			end
		elseif cell == 1 then
			-- Grass turns to island next to water in some cases:
			if up == 3 and down ~= 3 and left ~= 3 and right ~= 3 then
				level_copy[y][x] = { name_to_id["island_b"], 0 }
			elseif up ~= 3 and down == 3 and left ~= 3 and right ~= 3 then
				level_copy[y][x] = { name_to_id["island_t"], 0 }
			elseif up ~= 3 and down ~= 3 and left == 3 and right ~= 3 then
				level_copy[y][x] = { name_to_id["island_l"], 0 }
			elseif up ~= 3 and down ~= 3 and left ~= 3 and right == 3 then
				level_copy[y][x] = { name_to_id["island_r"], 0 }
			elseif up == 3 and down ~= 3 and left == 3 and right ~= 3 then
				level_copy[y][x] = { name_to_id["island_bl"], 0 }
			elseif up == 3 and down ~= 3 and left ~= 3 and right == 3 then
				level_copy[y][x] = { name_to_id["island_br"], 0 }
			elseif up ~= 3 and down == 3 and left == 3 and right ~= 3 then
				level_copy[y][x] = { name_to_id["island_tl"], 0 }
			elseif up ~= 3 and down == 3 and left ~= 3 and right == 3 then
				level_copy[y][x] = { name_to_id["island_tr"], 0 }
			else
				local top_left = (y > 1 and level[y - 1][x - 1] or 3)
				local top_right = (y > 1 and level[y - 1][x + 1] or 3)
				local bottom_left = (y < #level and level[y + 1][x - 1] or 3)
				local bottom_right = (y < #level and level[y + 1][x + 1] or 3)

				if top_left == 3 then
					level_copy[y][x] = { name_to_id["lake_bl"], 2 }
				elseif top_right == 3 then
					level_copy[y][x] = { name_to_id["lake_br"], 2 }
				elseif bottom_left == 3 then
					level_copy[y][x] = { name_to_id["lake_tl"], 2 }
				elseif bottom_right == 3 then
					level_copy[y][x] = { name_to_id["lake_tr"], 2 }
				else
					local rand = math.floor(math.random() * #grass_like) + 1
					level_copy[y][x] = { name_to_id[grass_like[rand]], 2 }
				end
			end
		elseif cell == 3 then
			-- Lake
			level_copy[y][x] = { name_to_id["lake_c"], 0 }
		else
			level_copy[y][x] = { name_to_id["grass"], 0 }
		end
	end
end

function getTileTypeAtPosition(x: number, y: number): number
	local grid_x = math.floor(x)
	local grid_y = math.floor(y)
	if grid_x < 0 or grid_x >= #level[1] or grid_y < 0 or grid_y >= #level then
		return 3 -- Out of bounds is lake
	end
	return level[grid_y + 1][grid_x + 1]
end

function getTileAt(x: number, y: number)
	local grid_x = math.floor(x)
	local grid_y = math.floor(y)
	if grid_x < 0 or grid_x >= #level[1] or grid_y < 0 or grid_y >= #level then
		return { name_to_id["lake_c"], 0 } -- Out of bounds is lake
	end
	return level_copy[grid_y + 1][grid_x + 1]
end

return {
	list_pumpkin = pumpkin_list,
	list_sign = sign_list,
	create_pumpkin = create_pumpkin,
	remove_entities = remove_entities,
	spawn_entities = spawn_entities,

	sprite_size = sprite_size,
	sprite_table = sprite_table,
	sprite_ids = sprite_ids,
	level = level_copy,
	getTileTypeAtPosition = getTileTypeAtPosition,
	getTileAt = getTileAt,
	list_path_marker = path_marker,
}
