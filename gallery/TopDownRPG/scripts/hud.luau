--[[
The bar at the top of the screen: where you are, how many gears you found and what you carry.
]]
const game_state = require("./game_state")
const graphics = require("@vectarine/graphics")
const items = require("./data/items")
const levels = require("./data/levels")
const loader = require("@vectarine/loader")
const sprites = require("./sprites")
const theme = require("./theme")
const types = require("./types")
const ui_components = require("./ui_components")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")
const world = require("./world")

const module = loader.init() or {}

local function drawText(text: string, pos: vec.Vec2, size: number, color: vec4.Vec4)
	theme.font:drawText(text, pos, size, color)
end

--- Where the player is: the name of the room, or the number and name of the level
local function drawLocation(game: types.Game, pos: vec.Vec2, height: number)
	local room = world.roomData(game.currentRoom)
	local title = if game.level then `{game.level.index}. {room.name}` else room.name
	local subtitle = if game.level then `{game.level.moves} moves` else room.subtitle
	local titleSize = height * 0.55
	local titleWidth = theme.font:measureText(title, titleSize).width
	drawText(title, pos, titleSize, theme.colors.gold)
	drawText(subtitle, pos + vec.V2(titleWidth + 0.04, 0), height * 0.4, theme.colors.textDim)
end

local function drawGears(game: types.Game, center: vec.Vec2, height: number)
	local iconSize = ui_components.square(height * 0.55)
	local fontSize = height * 0.45
	local text = `{game:finishedCount()} / {#levels.levels}`
	local width = iconSize.x + 0.02 + theme.font:measureText(text, fontSize).width
	local left = center.x - width / 2
	local spin = math.floor(ui_components.time * 2) % 4
	sprites.draw("extra/gear", vec.V2(left, center.y - iconSize.y / 2), iconSize, nil, if game.level then spin else 0)
	drawText(text, vec.V2(left + iconSize.x + 0.02, center.y - fontSize * 0.38), fontSize, theme.colors.gold)
end

--- One slot per item the player can use, with the number of arrows or bombs left.
local function drawItems(game: types.Game, right: vec.Vec2, height: number)
	local slotSize = ui_components.square(height * 0.62)
	local gap = 0.02
	local owned = game_state.ownedItems(game)
	local selected = game_state.selectedItem(game)
	local slots: { { sprite: string, count: number?, highlighted: boolean } } = {}
	for _, itemId in owned do
		local item = items.items[itemId]
		local count = if item.ammo then (game.inventory :: any)[item.ammo] else nil
		table.insert(slots, { sprite = item.sprite, count = count, highlighted = itemId == selected })
	end
	if game.inventory.keys > 0 then
		table.insert(slots, { sprite = "town/key", count = game.inventory.keys, highlighted = false })
	end
	for index, slot in slots do
		local x = right.x - (#slots - index + 1) * (slotSize.x + gap)
		local pos = vec.V2(x, right.y - slotSize.y / 2)
		ui_components.drawItemSlot(pos, slotSize, slot.sprite, slot.highlighted)
		if slot.count then
			drawText(`{slot.count}`, pos + vec.V2(slotSize.x * 0.62, 0.01), slotSize.y * 0.38, theme.colors.text)
		end
	end
	-- The effect of the potion drunk by the player
	local effect = game.player.effect
	if effect then
		local sprite = if effect == "leap" then "dungeon/potion_red" else "dungeon/potion_blue"
		local x = right.x - (#slots + 1) * (slotSize.x + gap)
		sprites.draw(sprite, vec.V2(x, right.y - slotSize.y / 2), slotSize)
	end
end

--- Draw the HUD in the rectangle starting at `pos` (bottom-left) of size `size`.
function module.draw(game: types.Game, pos: vec.Vec2, size: vec.Vec2)
	graphics.drawRect(pos, size, theme.colors.background)
	graphics.drawRect(pos, vec.V2(size.x, 0.008), theme.colors.goldDark)
	local height = math.min(size.y * 0.95, 0.3)
	local centerY = pos.y + size.y / 2
	drawLocation(game, vec.V2(pos.x + 0.04, centerY - height * 0.2), height)
	drawGears(game, vec.V2(pos.x + size.x / 2, centerY), height)
	drawItems(game, vec.V2(pos.x + size.x - 0.03, centerY), height)
end

return module
