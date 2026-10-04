--[[
The bar at the top of the screen: health, sanity, water, experience, and the equipped items.
]]
const gallery = require("./gallery")
const graphics = require("@vectarine/graphics")
const inventory = require("./inventory")
const items = require("./data/items")
const loader = require("@vectarine/loader")
const monsters = require("./data/monsters")
const player_rules = require("./player")
const prompts = require("./prompts")
const sprites = require("./sprites")
const theme = require("./theme")
const types = require("./types")
const ui_components = require("./ui_components")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

const module = loader.init() or {}

--- A bar filled with `fraction` of its width. `value` is written inside, `label` (if any) on its left.
local function drawBar(
	pos: vec.Vec2,
	size: vec.Vec2,
	fraction: number,
	color: vec4.Vec4,
	dark: vec4.Vec4,
	value: string,
	label: string?
)
	local border = ui_components.square(0.008)
	local filled = vec.V2(size.x * math.clamp(fraction, 0, 1), size.y)
	graphics.drawRect(pos - border, size + border:scale(2), theme.colors.panelBorder)
	graphics.drawRect(pos, size, dark)
	graphics.drawRect(pos, filled, color)
	-- A lighter band at the top of the filled part, lit from above
	graphics.drawRect(pos + vec.V2(0, size.y * 0.65), vec.V2(filled.x, size.y * 0.35), theme.withAlpha(theme.colors.text, 0.18))
	local textSize = size.y * 1.05
	if value ~= "" then
		local width = theme.font:measureText(value, textSize).width
		local textPos = pos + vec.V2((size.x - width) / 2, size.y * 0.08)
		theme.font:drawText(value, textPos + vec.V2(0.003, -0.004), textSize, theme.withAlpha(theme.colors.darkness, 0.8))
		theme.font:drawText(value, textPos, textSize, theme.colors.text)
	end
	if label then
		local width = theme.font:measureText(label, textSize).width
		theme.font:drawText(label, pos + vec.V2(-width - 0.015, size.y * 0.08), textSize, theme.colors.textDim)
	end
end

local function drawSlot(pos: vec.Vec2, size: vec.Vec2, itemId: string?, fallback: string?)
	ui_components.drawPanel(pos, size, theme.colors.textDim)
	local icon = if itemId then items.items[itemId].icon else fallback
	if icon then
		local margin = size:scale(0.12)
		sprites.draw(icon, pos + margin, size - margin:scale(2), if itemId then nil else theme.withAlpha(vec4.WHITE, 0.35))
	end
end

--- The health bar of a boss fighting the player, at the bottom of the world. `bottom` is the y of the bottom of the world.
--- Returns true if a bar was drawn.
function module.drawBossBar(game: types.Game, bottom: number): boolean
	for _, monster in game.monsters do
		local species = monsters.species[monster.kind]
		if species.boss and (monster.pos - game.player.pos):length() < species.sight then
			local width = 1.2
			local height = 0.035
			local pos = vec.V2(-width / 2, bottom + 0.05)
			theme.font:drawText(species.name, vec.V2(pos.x, pos.y + height + 0.012), 0.055, theme.colors.text)
			drawBar(pos, vec.V2(width, height), monster.hp / species.hp, theme.colors.health, theme.colors.healthDark, "")
			return true
		end
	end
	return false
end

--- The round of the snowball gallery: time left and score, at the top of the world. `top` is the y of its top.
function module.drawGallery(top: number)
	local state = gallery.state
	if not state.active then
		return
	end
	local text = `SNOWBALLS!  {math.ceil(state.time)} s   Score {state.score} / {gallery.GOAL}`
	local size = 0.06
	local width = theme.font:measureText(text, size).width
	local pos = vec.V2(-width / 2, top - 0.09)
	graphics.drawRect(pos - vec.V2(0.03, 0.02), vec.V2(width + 0.06, size + 0.03), theme.colors.panel)
	theme.font:drawText(text, pos, size, if state.time < 5 then theme.colors.gold else theme.colors.text)
end

--- Draw the HUD in the rectangle starting at `pos` (bottom-left) of size `size`.
function module.draw(game: types.Game, pos: vec.Vec2, size: vec.Vec2)
	local player = game.player
	graphics.drawRect(pos, size, theme.colors.background)
	graphics.drawRect(pos, vec.V2(size.x, 0.006), theme.colors.goldDark)

	local barHeight = size.y * 0.26
	local barWidth = 0.4
	local rowTop = pos.y + size.y * 0.56
	local rowBottom = pos.y + size.y * 0.16
	local left = pos.x + 0.13
	local maxHp = player_rules.maxHp(player)
	local maxSanity = player_rules.maxSanity(player)
	drawBar(
		vec.V2(left, rowTop),
		vec.V2(barWidth, barHeight),
		player.hp / maxHp,
		if player.poison > 0 then theme.colors.good else theme.colors.health,
		theme.colors.healthDark,
		`{math.ceil(player.hp)} / {maxHp}`,
		"HP"
	)
	drawBar(
		vec.V2(left, rowBottom),
		vec.V2(barWidth, barHeight),
		player.sanity / maxSanity,
		theme.colors.sanity,
		theme.colors.sanityDark,
		`{math.ceil(player.sanity)} / {maxSanity}`,
		"SAN"
	)

	-- Water: the thirst bar, and the sips left in the bottle
	local waterLeft = left + barWidth + 0.14
	drawBar(vec.V2(waterLeft, rowTop), vec.V2(barWidth * 0.55, barHeight), player.water / 100, theme.colors.water, theme.colors.waterDark, "", "H2O")
	local sipSize = ui_components.square(barHeight * 1.6)
	for i = 1, player_rules.bottleSize(player) do
		sprites.draw(
			"items/water_bottle",
			vec.V2(waterLeft + (i - 1) * sipSize.x * 0.85, rowBottom - barHeight * 0.3),
			sipSize,
			if i <= player.sips then nil else theme.withAlpha(vec4.WHITE, 0.2)
		)
	end

	-- Experience
	local xpLeft = waterLeft + barWidth * 0.55 + 0.12
	local needed = player_rules.xpToNext(player.level)
	-- Points to spend: shown in the bar, blinking
	local points = if player.statPoints > 0 and math.floor(game.time * 2) % 2 == 0
		then `+{player.statPoints} [{prompts.keyName(prompts.NOTEBOOK_KEY)}]`
		else ""
	drawBar(vec.V2(xpLeft, rowTop), vec.V2(barWidth * 0.7, barHeight), player.xp / needed, theme.colors.xp, theme.colors.xpDark, points, `LV {player.level}`)

	-- Warmth, below the level: only on the frontier (or while the player is still cold)
	if game.level.depth >= 4 or player.warmth < 100 then
		local cold = player.warmth < 30
		drawBar(
			vec.V2(xpLeft, rowBottom),
			vec.V2(barWidth * 0.7, barHeight),
			player.warmth / 100,
			if cold then theme.colors.sanity else theme.colors.gold,
			if cold then theme.colors.sanityDark else theme.colors.goldDark,
			""
		)
		-- A thermometer instead of a label (its glass is in the middle of the sprite)
		local iconSize = ui_components.square(barHeight * 1.6)
		sprites.draw("items/thermometer", vec.V2(xpLeft - 0.024 - iconSize.x * 0.75, rowBottom + (barHeight - iconSize.y) / 2), iconSize)
	end

	-- Equipped items
	local slotSize = ui_components.square(size.y * 0.62)
	local right = pos.x + size.x - 0.03
	local slotY = pos.y + (size.y - slotSize.y) / 2
	drawSlot(vec.V2(right - slotSize.x, slotY), slotSize, player.equipment.charm2, "items/lucky_shell")
	drawSlot(vec.V2(right - slotSize.x * 2.15, slotY), slotSize, player.equipment.charm, "items/lucky_shell")
	drawSlot(vec.V2(right - slotSize.x * 3.3, slotY), slotSize, player.equipment.armor, "items/leather_vest")
	drawSlot(vec.V2(right - slotSize.x * 4.45, slotY), slotSize, player.equipment.weapon or "fist", nil)
	-- Ammunition of the weapon
	local weapon = player.equipment.weapon
	local ammo = if weapon then items.items[weapon].ammo else nil
	if ammo then
		local count = inventory.count(player, ammo)
		theme.font:drawText(tostring(count), vec.V2(right - slotSize.x * 3.7, slotY + 0.005), slotSize.y * 0.4, theme.colors.text)
	end
end

return module
