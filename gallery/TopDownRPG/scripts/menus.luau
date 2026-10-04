--[[
In-game menus, opened over the world while the game is paused:
- notebook: the bag (items, equipment, stats), the bestiary, the artefacts and the map
- shop: buy items from a trader
- storage: move items between the bag and a chest (or the quantum chest, shared by all quantum chests)
- teleport: travel to an active teleporter
- pause

Menus read the keyboard themselves (arrows, Space, X, E, the notebook key, Escape): they are part of the interface, not of the rules.
The notebook key opens and closes the notebook, E turns its pages, Escape closes any menu.
`update` returns false when the menu closes.
]]
const graphics = require("@vectarine/graphics")
const inventory = require("./inventory")
const io = require("@vectarine/io")
const items = require("./data/items")
const levels = require("./data/levels")
const loader = require("@vectarine/loader")
const monsters = require("./data/monsters")
const map_view = require("./map_view")
const npcs = require("./npcs")
const player_rules = require("./player")
const prompts = require("./prompts")
const sprites = require("./sprites")
const theme = require("./theme")
const types = require("./types")
const ui = require("@vectarine/ui")
const ui_components = require("./ui_components")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

const module = loader.init() or {}

export type Menu = {
	kind: "notebook" | "shop" | "storage" | "teleport" | "pause",
	--- Notebook: the page shown
	page: number,
	--- Selected entry (meaning depends on the menu). Notebook map page: the index of the level shown.
	selected: number,
	--- Storage: 1 = bag column, 2 = chest column. Notebook bag page: 0 = stats, 1 = bag.
	column: number,
	npc: string?,
	chest: string?,
	--- A message shown at the bottom of the menu
	note: string?,
	--- Set by the pause menu when the player wants to quit
	quit: boolean?,
}

module.current = module.current :: Menu?

--- The scrollable lists of the menus, by name (see scrollList)
type ScrollList = {
	widget: ui.Widget<any>,
	size: vec.Vec2,
	contentHeight: number,
	--- The first line shown at the top of the view
	firstVisible: number,
	--- Set every frame: the line to keep visible, and what to draw
	line: number?,
	draw: (top: number) -> (),
}
local scrollLists: { [string]: ScrollList } = {}

const PAGES = { "Bag", "Creatures", "Artefacts", "Map" }
const STATS: { { id: types.StatName, name: string, help: string } } = {
	{ id = "health", name = "Health", help = "More health, faster healing" },
	{ id = "sanity", name = "Sanity", help = "More sanity, for powerful items" },
	{ id = "strength", name = "Strength", help = "Stronger hits, less damage taken" },
	{ id = "agility", name = "Agility", help = "Faster on your feet" },
	{ id = "intelligence", name = "Intelligence", help = "Use ancient items, more drops" },
}
--- Tools always carried, shown before the bag
const TOOLS = { "water_bottle", "flashlight", "notebook" }
const TOOL_TEXT = {
	water_bottle = "Your water bottle. Space: drink a sip. Fill it at a pool with [E].",
	flashlight = "Lights the way in front of you. Always on.",
	notebook = "Your notebook. Creatures, artefacts and maps are recorded here.",
}
const BAG_COLUMNS = 6

function module.open(menu: { kind: string, [string]: any })
	scrollLists = {}
	module.current = {
		kind = menu.kind :: any,
		page = menu.page or 1,
		selected = 1,
		column = 1,
		npc = menu.npc,
		chest = menu.chest,
	}
end

--- Show a page of the open notebook
function module.turnPage(game: types.Game, page: number)
	local menu = module.current :: Menu
	menu.page = page
	menu.selected = 1
	menu.column = 1
	menu.note = nil
	if PAGES[page] == "Artefacts" then
		menu.selected = game.level.depth
	end
	if PAGES[page] == "Map" then
		-- The map opens on the level the player is in
		menu.selected = table.find(map_view.visited(game), game.level.id) or 1
		map_view.parsed = {}
	end
end

function module.isOpen(): boolean
	return module.current ~= nil
end

function module.close()
	module.current = nil
end

local function pressed(key: string): boolean
	return io.isKeyJustPressed(key)
end

--- -1, 0 or 1 for the arrows pressed this frame
local function arrows(): (number, number)
	local dx = (if pressed("Right") or pressed("D") then 1 else 0) - (if pressed("Left") or pressed("A") then 1 else 0)
	local dy = (if pressed("Down") or pressed("S") then 1 else 0) - (if pressed("Up") or pressed("W") then 1 else 0)
	return dx, dy
end

-- MARK: Drawing helpers

local function text(content: string, pos: vec.Vec2, size: number, color: vec4.Vec4?)
	theme.font:drawText(content, pos, size, color or theme.colors.text)
end

local function textWidth(content: string, size: number): number
	return theme.font:measureText(content, size).width
end

--- Draw text wrapped to `width`. Returns the y below the last line.
local function paragraph(content: string, pos: vec.Vec2, width: number, size: number, color: vec4.Vec4?): number
	local y = pos.y
	local line = ""
	for word in string.gmatch(content, "%S+") do
		local candidate = if line == "" then word else line .. " " .. word
		if textWidth(candidate, size) > width and line ~= "" then
			text(line, vec.V2(pos.x, y), size, color)
			y -= size * 1.05
			line = word
		else
			line = candidate
		end
	end
	if line ~= "" then
		text(line, vec.V2(pos.x, y), size, color)
		y -= size * 1.05
	end
	return y
end

local function drawIcon(icon: string, pos: vec.Vec2, size: vec.Vec2, tint: vec4.Vec4?)
	sprites.draw(icon, pos, size, tint)
end

-- MARK: Notebook - bag page

--- Draws a scrollable list at `pos` (bottom-left corner), of `size` at most, for content that does not always fit.
--- The content is `contentHeight` high and drawn by `draw(top)` in the coordinates of the list: its bottom-left
--- corner is at (-1, -1) and the top of the content at `top`. The content is made of lines of `lineHeight`, and the
--- list scrolls to keep the line `line` (counted from 1 at the top) visible: the keyboard moves the selection, and
--- the list follows. A thin scrollbar on the right side shows when the content doesn't fit.
local function scrollList(key: string, pos: vec.Vec2, size: vec.Vec2, contentHeight: number, lineHeight: number, line: number?, draw: (top: number) -> ())
	-- The view shows whole lines: no line is cut at the top or at the bottom
	local fitted = math.max(1, math.floor(size.y / lineHeight + 0.001)) * lineHeight
	pos = pos + vec.V2(0, size.y - fitted)
	size = vec.V2(size.x, fitted)
	contentHeight = math.max(size.y, contentHeight)
	local list = scrollLists[key]
	-- The widget knows the size of its content: build it again when that changes
	local changed = list == nil or list.contentHeight ~= contentHeight or list.size.x ~= size.x or list.size.y ~= size.y
	if changed then
		local newList: ScrollList
		local content = ui.widget(vec.V2(size.x, contentHeight), function()
			newList.draw(-1 + contentHeight)
		end)
		local widget = ui.scrollableArea(size, content, function(info: ui.ScrollbarInfo): number
			local visibleLines = math.max(1, math.floor(info.viewHeight / lineHeight))
			local current = newList.line
			if current then
				if current < newList.firstVisible then
					newList.firstVisible = current
				elseif current > newList.firstVisible + visibleLines - 1 then
					newList.firstVisible = current - visibleLines + 1
				end
			end
			local hidden = info.contentHeight - info.viewHeight
			if hidden <= 0.001 then
				return 0
			end
			local ratio = math.clamp((newList.firstVisible - 1) * lineHeight / hidden, 0, 1)
			local x = -1 + info.viewWidth - 0.012
			local thumb = info.viewHeight * info.visibleRatio
			graphics.drawRect(vec.V2(x, -1), vec.V2(0.008, info.viewHeight), theme.colors.panelLight)
			graphics.drawRect(vec.V2(x, -1 + (info.viewHeight - thumb) * (1 - ratio)), vec.V2(0.008, thumb), theme.colors.gold)
			return ratio
		end)
		newList = { widget = widget, size = size, contentHeight = contentHeight, firstVisible = 1, line = line, draw = draw }
		list = newList
		scrollLists[key] = newList
	end
	local current = list :: ScrollList
	current.line = line
	current.draw = draw
	graphics.withTransformation({ translation = pos - vec.V2(-1, -1) }, function()
		current.widget:draw({})
	end)
end

local function bagEntries(game: types.Game): { { tool: string?, stack: types.ItemStack? } }
	local entries = {}
	for _, tool in TOOLS do
		table.insert(entries, { tool = tool })
	end
	for _, stack in game.player.inventory do
		table.insert(entries, { stack = stack })
	end
	return entries
end

local function updateBag(game: types.Game, menu: Menu)
	local player = game.player
	local dx, dy = arrows()
	if menu.column == 0 then
		if dx > 0 then
			menu.column = 1
			menu.selected = 1
		end
		menu.selected = math.clamp(menu.selected + dy, 1, #STATS)
		if pressed("Space") or pressed("Return") then
			local stat = STATS[menu.selected]
			if player_rules.spendPoint(player, stat.id) then
				menu.note = `{stat.name} is now {(player.stats :: any)[stat.id]}`
			else
				menu.note = "No points to spend. Level up by defeating creatures."
			end
		end
		return
	end
	local entries = bagEntries(game)
	local count = #TOOLS + inventory.capacity(player)
	local col = (menu.selected - 1) % BAG_COLUMNS
	if dx < 0 and col == 0 then
		menu.column = 0
		menu.selected = 1
		return
	end
	menu.selected = math.clamp(menu.selected + dx + dy * BAG_COLUMNS, 1, count)
	local entry = entries[menu.selected]
	if not entry then
		return
	end
	if pressed("Space") or pressed("Return") then
		if entry.tool == "water_bottle" then
			menu.note = player_rules.drink(player)
		elseif entry.stack then
			local itemId = entry.stack.id
			if inventory.slotOf(itemId) then
				menu.note = inventory.toggleEquip(player, itemId)
			elseif items.items[itemId].use then
				menu.note = player_rules.useItem(player, itemId)
			end
		end
	elseif pressed("X") and entry.stack then
		local stack = entry.stack
		inventory.take(player, stack.id, stack.count)
		table.insert(game.pickups, { pos = player.pos, item = stack.id, count = stack.count, age = -1, dead = false })
		menu.note = `You drop the {string.lower(items.items[stack.id].name)}.`
	end
end

local function itemStats(itemId: string): string
	local item = items.items[itemId]
	local parts = {}
	if item.damage then
		table.insert(parts, `{item.damage} {item.damageType} damage`)
	end
	if item.defense then
		table.insert(parts, `{item.defense} defense`)
	end
	if item.resist then
		for damageType, multiplier in item.resist do
			table.insert(parts, `-{math.floor((1 - multiplier) * 100)}% {damageType}`)
		end
	end
	if item.ammo then
		table.insert(parts, `uses {string.lower(items.items[item.ammo].name)}`)
	end
	if item.sanityCost then
		table.insert(parts, `{item.sanityCost} sanity per shot`)
	end
	if item.requires then
		for stat, value in item.requires do
			table.insert(parts, `needs {value} {stat}`)
		end
	end
	return table.concat(parts, " · ")
end

local function drawBag(game: types.Game, menu: Menu, area: { pos: vec.Vec2, size: vec.Vec2 })
	local player = game.player
	local left = area.pos.x + 0.05
	local top = area.pos.y + area.size.y - 0.05

	-- Stats
	text(`Level {player.level}`, vec.V2(left, top - 0.06), 0.07, theme.colors.gold)
	text(`{player.xp} / {player_rules.xpToNext(player.level)} xp`, vec.V2(left, top - 0.13), 0.05, theme.colors.textDim)
	local pointsColor = if player.statPoints > 0 then theme.colors.gold else theme.colors.textDim
	text(`{player.statPoints} points to spend`, vec.V2(left, top - 0.2), 0.05, pointsColor)
	for i, stat in STATS do
		local y = top - 0.3 - (i - 1) * 0.085
		local selected = menu.column == 0 and menu.selected == i
		if selected then
			graphics.drawRect(vec.V2(left - 0.02, y - 0.015), vec.V2(0.5, 0.075), theme.colors.panelLight)
		end
		text(stat.name, vec.V2(left, y), 0.055, if selected then theme.colors.gold else theme.colors.text)
		text(tostring((player.stats :: any)[stat.id]), vec.V2(left + 0.4, y), 0.055, theme.colors.text)
	end
	text(`Health {math.ceil(player.hp)}/{player_rules.maxHp(player)}`, vec.V2(left, top - 0.78), 0.045, theme.colors.textDim)
	text(`Sanity {math.ceil(player.sanity)}/{player_rules.maxSanity(player)}`, vec.V2(left, top - 0.84), 0.045, theme.colors.textDim)

	-- Bag grid: it scrolls when the bag holds more rows than fit above the description
	local entries = bagEntries(game)
	local slots = #TOOLS + inventory.capacity(player)
	local slotSize = ui_components.square(0.15)
	local gridLeft = area.pos.x + 0.62
	local gap = vec.V2(0.012, 0.015)
	local rowHeight = slotSize.y + gap.y
	local rows = math.ceil(slots / BAG_COLUMNS)
	local gridTop = top - 0.2 + slotSize.y + gap.y
	local gridBottom = area.pos.y + 0.41
	local gridSize = vec.V2(BAG_COLUMNS * (slotSize.x + gap.x) + 0.02, gridTop - gridBottom)
	local selectedRow = if menu.column == 1 then (menu.selected - 1) // BAG_COLUMNS + 1 else nil
	scrollList("bag", vec.V2(gridLeft, gridBottom), gridSize, rows * rowHeight, rowHeight, selectedRow, function(contentTop: number)
		for i = 1, slots do
			local col = (i - 1) % BAG_COLUMNS
			local row = (i - 1) // BAG_COLUMNS
			local pos = vec.V2(-1 + col * (slotSize.x + gap.x), contentTop - (row + 1) * rowHeight + gap.y / 2)
			local selected = menu.column == 1 and menu.selected == i
			local entry = entries[i]
			local equipped = entry and entry.stack and inventory.isEquipped(player, entry.stack.id)
			ui_components.drawPanel(
				pos,
				slotSize,
				if selected then theme.colors.gold elseif equipped then theme.colors.good else theme.colors.textDim,
				if i <= #TOOLS then theme.colors.panelLight else nil
			)
			if entry then
				local icon = if entry.tool then items.items[entry.tool] and items.items[entry.tool].icon or `items/{entry.tool}` else items.items[(entry.stack :: types.ItemStack).id].icon
				local margin = slotSize:scale(0.12)
				drawIcon(icon, pos + margin, slotSize - margin:scale(2))
				local count = if entry.tool == "water_bottle" then player.sips elseif entry.stack and entry.stack.count > 1 then entry.stack.count else nil
				if count then
					text(tostring(count), pos + vec.V2(slotSize.x * 0.6, 0.008), 0.045, theme.colors.text)
				end
				if equipped then
					text("E", pos + vec.V2(0.012, slotSize.y - 0.05), 0.04, theme.colors.good)
				end
			end
		end
	end)

	-- Description of the selected entry
	local descriptionTop = area.pos.y + 0.32
	local entry = if menu.column == 1 then entries[menu.selected] else nil
	if entry then
		local name, description, stats
		if entry.tool then
			name = if entry.tool == "water_bottle" then "Water bottle" elseif entry.tool == "flashlight" then "Flashlight" else "Notebook"
			description = TOOL_TEXT[entry.tool]
			stats = ""
		else
			local item = items.items[(entry.stack :: types.ItemStack).id]
			name = item.name
			description = item.description
			stats = itemStats((entry.stack :: types.ItemStack).id)
		end
		text(name, vec.V2(gridLeft, descriptionTop), 0.065, theme.colors.gold)
		local y = paragraph(description, vec.V2(gridLeft, descriptionTop - 0.07), area.size.x - 0.7, 0.048)
		if stats ~= "" then
			paragraph(stats, vec.V2(gridLeft, y - 0.01), area.size.x - 0.7, 0.042, theme.colors.textDim)
		end
	elseif menu.column == 0 then
		paragraph(STATS[menu.selected].help, vec.V2(gridLeft, descriptionTop), area.size.x - 0.7, 0.05, theme.colors.textDim)
	end
end

-- MARK: Notebook - other pages

local function updateCreatures(game: types.Game, menu: Menu)
	local _, dy = arrows()
	menu.selected = math.clamp(menu.selected + dy, 1, #monsters.order)
end

local function drawCreatures(game: types.Game, menu: Menu, area: { pos: vec.Vec2, size: vec.Vec2 })
	local left = area.pos.x + 0.05
	local top = area.pos.y + area.size.y - 0.08
	local listWidth = 0.55
	-- The list scrolls: there are more creatures than lines on the page
	local lineHeight = 0.075
	local listTop = top + 0.05
	local listBottom = area.pos.y + 0.03
	local listSize = vec.V2(listWidth + 0.03, listTop - listBottom)
	scrollList("creatures", vec.V2(left - 0.02, listBottom), listSize, #monsters.order * lineHeight, lineHeight, menu.selected, function(contentTop: number)
		for i, kind in monsters.order do
			local known = game.notebook.creatures[kind] ~= nil
			local y = contentTop - i * lineHeight
			local selected = menu.selected == i
			if selected then
				graphics.drawRect(vec.V2(-1, y + 0.003), vec.V2(listWidth, lineHeight - 0.005), theme.colors.panelLight)
			end
			local name = if known then monsters.species[kind].name else "???"
			text(name, vec.V2(-0.98, y + 0.018), 0.05, if selected then theme.colors.gold elseif known then theme.colors.text else theme.colors.textDim)
		end
	end)
	local kind = monsters.order[menu.selected]
	local species = monsters.species[kind]
	local right = left + listWidth + 0.08
	if game.notebook.creatures[kind] == nil then
		paragraph("You have not met this creature yet.", vec.V2(right, top), area.size.x - listWidth - 0.25, 0.05, theme.colors.textDim)
		return
	end
	local spriteSize = ui_components.square(0.32)
	local sheet = if species.size == 2 then "bosses" else "characters"
	drawIcon(`{sheet}/{species.sprite}_0`, vec.V2(right, top - spriteSize.y + 0.04), spriteSize)
	text(species.name, vec.V2(right + spriteSize.x + 0.04, top - 0.06), 0.07, theme.colors.gold)
	text(`Defeated: {game.notebook.creatures[kind]}`, vec.V2(right + spriteSize.x + 0.04, top - 0.14), 0.045, theme.colors.textDim)
	local y = paragraph(species.description, vec.V2(right, top - 0.38), area.size.x - listWidth - 0.25, 0.048)
	local facts = { `Health {species.hp}`, `Damage {species.damage} ({species.damageType})` }
	for damageType, multiplier in species.resist or {} do
		table.insert(facts, if multiplier > 1 then `Weak to {damageType}` else `Resists {damageType}`)
	end
	paragraph(table.concat(facts, " · "), vec.V2(right, y - 0.02), area.size.x - listWidth - 0.25, 0.042, theme.colors.textDim)
end

--- The deepest floor the player reached: the artefacts page shows the floors down to it
local function deepestFloor(game: types.Game): number
	return math.max(game.flags.deepestFloor or 1, game.level.depth)
end

--- Artefacts page: `menu.selected` is the floor shown
local function updateArtefacts(game: types.Game, menu: Menu)
	local dx = arrows()
	menu.selected = math.clamp(menu.selected + dx, 1, deepestFloor(game))
end

local function drawArtefacts(game: types.Game, menu: Menu, area: { pos: vec.Vec2, size: vec.Vec2 })
	local left = area.pos.x + 0.05
	local top = area.pos.y + area.size.y - 0.04
	local found = 0
	local iconSize = ui_components.square(0.2)
	local floorArtefacts = items.artefactsByFloor[menu.selected] or {}
	for i, id in floorArtefacts do
		local item = items.items[id]
		local owned = game.player.artefacts[id] == true
		if owned then
			found += 1
		end
		-- Each row: the icon on the left, the name and the description on its right, aligned on its top
		local rowTop = top - 0.12 - (i - 1) * 0.28
		local iconPos = vec.V2(left, rowTop - iconSize.y)
		ui_components.drawPanel(iconPos, iconSize, if owned then theme.colors.gold else theme.colors.textDim)
		drawIcon(item.icon, iconPos + iconSize:scale(0.1), iconSize:scale(0.8), if owned then nil else vec4.createColor(0, 0, 0, 0.7))
		local textLeft = left + iconSize.x + 0.04
		text(if owned then item.name else "???", vec.V2(textLeft, rowTop - 0.06), 0.06, if owned then theme.colors.gold else theme.colors.textDim)
		paragraph(
			if owned then item.description else "An artefact of the sea people, still hidden somewhere.",
			vec.V2(textLeft, rowTop - 0.12),
			area.size.x - iconSize.x - 0.2,
			0.045,
			theme.colors.textDim
		)
	end
	local floorName = levels.floorNames[menu.selected] or `Floor {menu.selected}`
	local title = `{floorName}: {found} / {#floorArtefacts} artefacts found`
	text(if deepestFloor(game) > 1 then `< {title} >` else title, vec.V2(left, top - 0.06), 0.05, theme.colors.text)
end

--- Map page: `menu.selected` is the index of the level shown, among the levels visited
local function updateMap(game: types.Game, menu: Menu)
	local dx = arrows()
	menu.selected = math.clamp(menu.selected + dx, 1, #map_view.visited(game))
end

local function drawMap(game: types.Game, menu: Menu, area: { pos: vec.Vec2, size: vec.Vec2 })
	local left = area.pos.x + 0.05
	local top = area.pos.y + area.size.y - 0.08
	if not game.player.artefacts.cartographer_lens then
		paragraph(
			"Your sketches are a mess of lines and arrows. Wren used to draw maps... maybe he knows a better way.",
			vec.V2(left, top),
			area.size.x - 0.1,
			0.055,
			theme.colors.textDim
		)
		return
	end
	local visited = map_view.visited(game)
	local id = visited[menu.selected] or game.level.id
	local name = levels.levels[id].name
	local title = if #visited > 1 then `< {name} >` else name
	text(title, vec.V2(area.pos.x + (area.size.x - textWidth(title, 0.065)) / 2, top), 0.065, theme.colors.gold)
	map_view.draw(game, id, { pos = area.pos + vec.V2(0.04, 0.04), size = area.size - vec.V2(0.08, 0.2) })
end

-- MARK: Shop, storage, teleport

local function updateShop(game: types.Game, menu: Menu)
	local shop = npcs.npcs[menu.npc :: string].shop or {}
	local _, dy = arrows()
	menu.selected = math.clamp(menu.selected + dy, 1, #shop)
	if pressed("Space") or pressed("Return") then
		local offer = shop[menu.selected]
		if inventory.count(game.player, offer.currency) < offer.price then
			menu.note = `Not enough {string.lower(items.items[offer.currency].name)}s.`
		else
			inventory.take(game.player, offer.currency, offer.price)
			local left = inventory.give(game.player, offer.item, offer.count)
			if left > 0 then
				table.insert(game.pickups, { pos = game.player.pos, item = offer.item, count = left, age = -1, dead = false })
			end
			menu.note = `You get {offer.count} × {items.items[offer.item].name}.`
		end
	end
end

local function drawShop(game: types.Game, menu: Menu, area: { pos: vec.Vec2, size: vec.Vec2 })
	local npc = npcs.npcs[menu.npc :: string]
	local shop = npc.shop or {}
	local left = area.pos.x + 0.05
	local top = area.pos.y + area.size.y - 0.08
	text(`{npc.name} trades`, vec.V2(left, top), 0.07, theme.colors.gold)
	local currency = if #shop > 0 then shop[1].currency else "fig"
	text(`You have {inventory.count(game.player, currency)} {string.lower(items.items[currency].name)}s`, vec.V2(left, top - 0.08), 0.05, theme.colors.textDim)
	local iconSize = ui_components.square(0.09)
	for i, offer in shop do
		local y = top - 0.2 - (i - 1) * 0.11
		local selected = menu.selected == i
		if selected then
			graphics.drawRect(vec.V2(left - 0.02, y - 0.02), vec.V2(area.size.x - 0.06, 0.1), theme.colors.panelLight)
		end
		local item = items.items[offer.item]
		drawIcon(item.icon, vec.V2(left, y - 0.005), iconSize)
		local label = if offer.count > 1 then `{item.name} × {offer.count}` else item.name
		text(label, vec.V2(left + iconSize.x + 0.03, y + 0.01), 0.055, if selected then theme.colors.gold else theme.colors.text)
		text(`{offer.price}`, vec.V2(left + area.size.x - 0.25, y + 0.01), 0.055, theme.colors.text)
		drawIcon(items.items[offer.currency].icon, vec.V2(left + area.size.x - 0.18, y - 0.005), iconSize)
	end
	local offer = shop[menu.selected]
	if offer then
		paragraph(items.items[offer.item].description, vec.V2(left, area.pos.y + 0.2), area.size.x - 0.1, 0.045, theme.colors.textDim)
	end
end

local function chestStacks(game: types.Game, menu: Menu): { types.ItemStack }
	local key = menu.chest :: string
	game.chests[key] = game.chests[key] or {}
	return game.chests[key]
end

const CHEST_SIZE = 24

local function updateStorage(game: types.Game, menu: Menu)
	local dx, dy = arrows()
	if dx ~= 0 then
		menu.column = if dx > 0 then 2 else 1
		menu.selected = 1
	end
	local source = if menu.column == 1 then game.player.inventory else chestStacks(game, menu)
	menu.selected = math.clamp(menu.selected + dy, 1, math.max(1, #source))
	if (pressed("Space") or pressed("Return")) and source[menu.selected] then
		local stack = source[menu.selected]
		local target = if menu.column == 1 then chestStacks(game, menu) else game.player.inventory
		local capacity = if menu.column == 1 then CHEST_SIZE else inventory.capacity(game.player)
		local left = inventory.addTo(target, capacity, stack.id, stack.count)
		local moved = stack.count - left
		if moved == 0 then
			menu.note = "No room."
		else
			if menu.column == 1 then
				inventory.take(game.player, stack.id, moved)
			else
				inventory.removeFrom(source, stack.id, moved)
			end
			menu.selected = math.clamp(menu.selected, 1, math.max(1, #source))
		end
	end
end

--- Height of a row of the storage lists
const ROW = 0.085

local function storageStacks(game: types.Game, menu: Menu, column: number): { types.ItemStack }
	return if column == 1 then game.player.inventory else chestStacks(game, menu)
end

--- A list of the storage menu (1 = the bag, 2 = the chest), in a scrollable area: big bags and chests don't fit
local function drawStorageList(game: types.Game, menu: Menu, column: number, pos: vec.Vec2, size: vec.Vec2)
	local stacks = storageStacks(game, menu, column)
	local selected = if menu.column == column then menu.selected else nil
	scrollList(`storage{column}`, pos, size, math.max(1, #stacks) * ROW, ROW, selected, function(top: number)
		local iconSize = ui_components.square(0.08)
		for i, stack in stacks do
			local y = top - i * ROW
			if selected == i then
				graphics.drawRect(vec.V2(-1, y + 0.003), vec.V2(size.x - 0.03, ROW - 0.006), theme.colors.panelLight)
			end
			drawIcon(items.items[stack.id].icon, vec.V2(-0.98, y + 0.003), iconSize)
			local label = items.items[stack.id].name .. (if stack.count > 1 then ` × {stack.count}` else "")
			text(label, vec.V2(-0.98 + iconSize.x + 0.02, y + 0.02), 0.045, if selected == i then theme.colors.gold else theme.colors.text)
		end
		if #stacks == 0 then
			text("(empty)", vec.V2(-0.98, top - ROW + 0.02), 0.045, theme.colors.textDim)
		end
	end)
end

local function drawStorage(game: types.Game, menu: Menu, area: { pos: vec.Vec2, size: vec.Vec2 })
	local top = area.pos.y + area.size.y - 0.08
	local half = area.size.x / 2
	local title = if menu.chest == "quantum" then "Quantum chest (shared)" else "Chest"
	-- The lists go from below the titles to above the note at the bottom of the menu
	local listSize = vec.V2(half - 0.08, area.size.y - 0.32)
	local listBottom = area.pos.y + 0.12
	for column, x in { area.pos.x + 0.05, area.pos.x + half + 0.03 } do
		text(if column == 1 then "Your bag" else title, vec.V2(x, top), 0.06, theme.colors.gold)
		drawStorageList(game, menu, column, vec.V2(x, listBottom), listSize)
	end
end

local function teleporters(game: types.Game): { { key: string, name: string } }
	local list: { { key: string, name: string } } = {}
	for flag, value in game.flags do
		if type(value) == "table" and string.sub(flag, 1, 11) == "teleporter:" then
			table.insert(list, { key = string.sub(flag, 12), name = value.name })
		end
	end
	table.sort(list, function(a: { key: string, name: string }, b: { key: string, name: string }): boolean
		return a.name < b.name
	end)
	return list
end

local function drawTeleport(game: types.Game, menu: Menu, area: { pos: vec.Vec2, size: vec.Vec2 })
	local left = area.pos.x + 0.05
	local top = area.pos.y + area.size.y - 0.08
	text("Where to?", vec.V2(left, top), 0.07, theme.colors.gold)
	for i, entry in teleporters(game) do
		local y = top - 0.15 - (i - 1) * 0.09
		local selected = menu.selected == i
		if selected then
			graphics.drawRect(vec.V2(left - 0.02, y - 0.02), vec.V2(area.size.x - 0.06, 0.08), theme.colors.panelLight)
		end
		text(entry.name, vec.V2(left, y), 0.055, if selected then theme.colors.gold else theme.colors.text)
	end
end

const PAUSE_ITEMS = { "Resume", "Notebook", "Save and quit to title" }

local function drawPause(menu: Menu, area: { pos: vec.Vec2, size: vec.Vec2 })
	local center = area.pos + area.size:scale(0.5)
	local title = "Paused"
	text(title, vec.V2(center.x - textWidth(title, 0.12) / 2, center.y + 0.2), 0.12, theme.colors.gold)
	for i, label in PAUSE_ITEMS do
		local selected = menu.selected == i
		local size = 0.065
		local pos = vec.V2(center.x - textWidth(label, size) / 2, center.y + 0.02 - (i - 1) * 0.1)
		text(label, pos, size, if selected then theme.colors.gold else theme.colors.text)
	end
end

-- MARK: Update and draw

--- Handle the keyboard for the open menu. Returns an action for the screen ("travel", "quit"...) or nil.
function module.update(game: types.Game): { kind: string, [string]: any }?
	local menu = module.current
	if not menu then
		return nil
	end
	if pressed("Escape") or (menu.kind ~= "notebook" and pressed("E")) then
		module.close()
		return nil
	end
	if menu.kind == "notebook" then
		if pressed(prompts.NOTEBOOK_KEY) then
			module.close()
			return nil
		end
		if pressed(prompts.PAGE_KEY) then
			module.turnPage(game, menu.page % #PAGES + 1)
			return nil
		end
		if menu.page == 1 then
			updateBag(game, menu)
		elseif menu.page == 2 then
			updateCreatures(game, menu)
		elseif menu.page == 3 then
			updateArtefacts(game, menu)
		elseif menu.page == 4 then
			updateMap(game, menu)
		end
	elseif menu.kind == "shop" then
		updateShop(game, menu)
	elseif menu.kind == "storage" then
		updateStorage(game, menu)
	elseif menu.kind == "teleport" then
		local list = teleporters(game)
		local _, dy = arrows()
		menu.selected = math.clamp(menu.selected + dy, 1, math.max(1, #list))
		if (pressed("Space") or pressed("Return")) and list[menu.selected] then
			module.close()
			return { kind = "travel", key = list[menu.selected].key }
		end
	elseif menu.kind == "pause" then
		local _, dy = arrows()
		menu.selected = math.clamp(menu.selected + dy, 1, #PAUSE_ITEMS)
		if pressed("Space") or pressed("Return") then
			if menu.selected == 1 then
				module.close()
			elseif menu.selected == 2 then
				module.open({ kind = "notebook" })
			else
				module.close()
				return { kind = "quit" }
			end
		end
	end
	return nil
end

--- The keys to show in the prompt bar while a menu is open
function module.prompts(game: types.Game): { { key: string, label: string } }
	local menu = module.current
	if not menu then
		return {}
	end
	if menu.kind == "notebook" then
		local list = { { key = string.lower(prompts.PAGE_KEY), label = "Next page" }, { key = "esc", label = "Close" } }
		if menu.page == 4 and #map_view.visited(game) > 1 then
			table.insert(list, 1, { key = "arrows", label = "Other places" })
		elseif menu.page == 3 and deepestFloor(game) > 1 then
			table.insert(list, 1, { key = "arrows", label = "Other floors" })
		end
		if menu.page == 1 then
			table.insert(list, 1, { key = "x", label = "Drop" })
			table.insert(list, 1, { key = "space", label = if menu.column == 0 then "Spend point" else "Use / Equip" })
		end
		return list
	elseif menu.kind == "storage" then
		return { { key = "arrows", label = "Choose" }, { key = "space", label = "Move" }, { key = "esc", label = "Close" } }
	elseif menu.kind == "shop" then
		return { { key = "space", label = "Buy" }, { key = "esc", label = "Close" } }
	end
	return { { key = "space", label = "Choose" }, { key = "esc", label = "Close" } }
end

function module.draw(game: types.Game, area: { pos: vec.Vec2, size: vec.Vec2 })
	local menu = module.current
	if not menu then
		return
	end
	graphics.drawRect(vec.V2(-1, -1), vec.V2(2, 2), theme.withAlpha(theme.colors.darkness, 0.55))
	ui_components.drawPanel(area.pos, area.size)
	local inner = { pos = area.pos, size = area.size }
	if menu.kind == "notebook" then
		-- Tabs of the notebook
		local tabLeft = area.pos.x + 0.05
		for i, page in PAGES do
			local color = if i == menu.page then theme.colors.gold else theme.colors.textDim
			text(page, vec.V2(tabLeft, area.pos.y + area.size.y + 0.02), 0.06, color)
			tabLeft += textWidth(page, 0.06) + 0.06
		end
		if menu.page == 1 then
			drawBag(game, menu, inner)
		elseif menu.page == 2 then
			drawCreatures(game, menu, inner)
		elseif menu.page == 3 then
			drawArtefacts(game, menu, inner)
		else
			drawMap(game, menu, inner)
		end
	elseif menu.kind == "shop" then
		drawShop(game, menu, inner)
	elseif menu.kind == "storage" then
		drawStorage(game, menu, inner)
	elseif menu.kind == "teleport" then
		drawTeleport(game, menu, inner)
	elseif menu.kind == "pause" then
		drawPause(menu, inner)
	end
	if menu.note then
		text(menu.note, vec.V2(area.pos.x + 0.05, area.pos.y + 0.04), 0.05, theme.colors.gold)
	end
end

return module
