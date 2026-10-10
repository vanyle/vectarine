--[[
The map page of the notebook: the explored cells of a level, with the places that matter
(exits and where they lead, campfires, teleporters, people, stairs, cisterns) and the player.

Places are recorded from the start (notebook.luau), but they are only drawn with the Cartographer's lens.
The player can browse the maps of every level they visited.
]]
const graphics = require("@vectarine/graphics")
const level = require("./level")
const levels = require("./data/levels")
const loader = require("@vectarine/loader")
const notebook = require("./notebook")
const npcs = require("./npcs")
const sprites = require("./sprites")
const theme = require("./theme")
const types = require("./types")
const ui_components = require("./ui_components")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

const module = loader.init() or {}

--- Levels read for their map, by id. Reset when the map page opens, so that it shows the walls blown up since.
module.parsed = {} :: { [string]: level.Level }

const COLORS = {
	floor = theme.hex("696353"),
	high = theme.hex("8d8a6c"),
	wall = theme.hex("3e4a52"),
	water = theme.hex("245273"),
	chasm = theme.hex("0b1016"),
	vines = theme.hex("4b6e3a"),
	ladder = theme.hex("8a5a3b"),
}

local function parsed(game: types.Game, id: string): level.Level
	local lvl = module.parsed[id]
	if not lvl then
		lvl = level.load(id, game.flags)
		module.parsed[id] = lvl
	end
	return lvl
end

--- The levels the player has seen, in the order of the journey
function module.visited(game: types.Game): { string }
	local list = {}
	for _, id in levels.order do
		if id == game.level.id or game.notebook.explored[id] ~= nil then
			table.insert(list, id)
		end
	end
	return list
end

--- A function telling whether a cell of the level `id` is explored
local function exploredIn(game: types.Game, id: string): (x: number, y: number) -> boolean
	if id == game.level.id then
		return function(x: number, y: number): boolean
			return level.inside(game.level :: level.Level, x, y) and notebook.isExplored(game, x, y)
		end
	end
	local rows = game.notebook.explored[id] or {}
	return function(x: number, y: number): boolean
		local row = rows[y + 1]
		return row ~= nil and string.sub(row, x + 1, x + 1) == "1"
	end
end

local function cellColor(lvl: level.Level, x: number, y: number): vec4.Vec4
	local wall = level.wallAt(lvl, x, y)
	local ground = level.groundAt(lvl, x, y)
	if wall and wall.ladder then
		return COLORS.ladder
	elseif wall and wall.climbable then
		return COLORS.vines
	elseif wall and not wall.walkable then
		return COLORS.wall
	elseif ground and ground.water then
		return COLORS.water
	elseif ground and ground.name == "chasm" then
		return COLORS.chasm
	elseif level.heightAt(lvl, x, y) > 0 then
		return COLORS.high
	end
	return COLORS.floor
end

--- The sprite marking a prop on the map, if it matters for finding the way
local function markerOf(game: types.Game, prop: types.Prop): string?
	if prop.kind == "campfire" then
		return "world/campfire_0"
	elseif prop.kind == "teleporter" then
		return if game.flags[`teleporter:{prop.key}`] then "world/teleporter_on_0" else "world/teleporter_off"
	elseif prop.kind == "stairs" then
		return "world/stairs_down"
	elseif prop.kind == "stairs_up" then
		return "world/stairs_up"
	elseif prop.kind == "cistern" then
		return if game.flags.waterRestored or prop.data.always then "world/cistern_full" else "world/cistern"
	elseif prop.kind == "npc" then
		return `characters/{npcs.npcs[prop.data.npc].sprite}_0`
	end
	return nil
end

local function label(content: string, pos: vec.Vec2, size: number, color: vec4.Vec4, area: { pos: vec.Vec2, size: vec.Vec2 })
	local width = theme.font:measureText(content, size).width
	local x = math.clamp(pos.x - width / 2, area.pos.x + 0.02, area.pos.x + area.size.x - width - 0.02)
	local y = math.clamp(pos.y, area.pos.y + 0.02, area.pos.y + area.size.y - size)
	graphics.drawRect(vec.V2(x - 0.008, y - 0.008), vec.V2(width + 0.016, size + 0.004), theme.colors.shadow)
	theme.font:drawText(content, vec.V2(x, y), size, color)
end

--- Draw the map of the level `id` in `area` (y up, like the rest of the interface)
function module.draw(game: types.Game, id: string, area: { pos: vec.Vec2, size: vec.Vec2 })
	local lvl = if id == game.level.id then game.level :: level.Level else parsed(game, id)
	local isExplored = exploredIn(game, id)
	-- Cells are square on the screen: OpenGL units are not, so a cell is `cell` high and narrower than that
	local unit = ui_components.square(1)
	local cell = math.min(area.size.x / (lvl.width * unit.x), area.size.y / lvl.height)
	local cellSize = unit:scale(cell)
	-- Top left corner of the map, centered in the area
	local origin = vec.V2(
		area.pos.x + (area.size.x - cellSize.x * lvl.width) / 2,
		area.pos.y + area.size.y - (area.size.y - cellSize.y * lvl.height) / 2
	)
	local function corner(x: number, y: number): vec.Vec2
		return origin + vec.V2(x * cellSize.x, -(y + 1) * cellSize.y)
	end

	-- Explored cells, and the walls around them (the explorer sees the walls of the places they walk in)
	local function seen(x: number, y: number): boolean
		if isExplored(x, y) then
			return true
		end
		for dy = -1, 1 do
			for dx = -1, 1 do
				if isExplored(x + dx, y + dy) then
					return true
				end
			end
		end
		return false
	end
	for y = 0, lvl.height - 1 do
		for x = 0, lvl.width - 1 do
			local color = cellColor(lvl, x, y)
			if isExplored(x, y) or (color == COLORS.wall and seen(x, y)) then
				graphics.drawRect(corner(x, y), cellSize, color)
			end
		end
	end

	-- Places
	local markerSize = unit:scale(math.max(cell * 1.6, 0.05))
	for _, prop in lvl.props do
		local marker = markerOf(game, prop)
		if marker and isExplored(prop.cell.x, prop.cell.y) then
			local center = corner(prop.cell.x, prop.cell.y) + cellSize:scale(0.5)
			sprites.draw(marker, center - markerSize:scale(0.5), markerSize)
		end
	end

	-- Exits, with the name of where they lead
	for name, cells in lvl.entries do
		local link = lvl.links[name]
		local found = false
		local sum = vec.V2(0, 0)
		for _, entryCell in cells do
			sum += entryCell
			if isExplored(entryCell.x, entryCell.y) then
				found = true
				graphics.drawRect(corner(entryCell.x, entryCell.y), cellSize, theme.colors.gold)
			end
		end
		if found and link then
			local middle = sum:scale(1 / #cells)
			-- Labels go inside the map, away from the edge the exit is on
			local inward = vec.V2(0, 0)
			if middle.x < 2 then
				inward = vec.V2(4, 0)
			elseif middle.x > lvl.width - 3 then
				inward = vec.V2(-4, 0)
			elseif middle.y < 2 then
				inward = vec.V2(0, 2.5)
			else
				inward = vec.V2(0, -2.5)
			end
			local spot = middle + inward
			label(levels.levels[link.level].name, corner(spot.x, spot.y) + cellSize:scale(0.5), 0.042, theme.colors.gold, area)
		end
	end

	-- The player, blinking, on the map of the level they are in
	if id == game.level.id and math.floor(game.time * 3) % 2 == 0 then
		local p = game.player.pos
		local size = unit:scale(math.max(cell * 1.4, 0.035))
		local center = origin + vec.V2(p.x * cellSize.x, -p.y * cellSize.y)
		graphics.drawRect(center - size:scale(0.5), size, theme.colors.text)
	end
end

return module
