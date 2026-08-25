--[[
Big texts drawn over the game: the name of the room, "GEAR FOUND!" and the ending.
]]
const graphics = require("@vectarine/graphics")
const loader = require("@vectarine/loader")
const sprites = require("./sprites")
const theme = require("./theme")
const ui_components = require("./ui_components")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

const module = loader.init() or {}

--- Centered text with a shadow
local function drawCentered(text: string, center: vec.Vec2, size: number, color: vec4.Vec4, shadow: vec4.Vec4)
	local width = theme.font:measureText(text, size).width
	local pos = vec.V2(center.x - width / 2, center.y - size / 2)
	theme.font:drawText(text, pos + vec.V2(0.008, -0.012), size, shadow)
	theme.font:drawText(text, pos, size, color)
end

--- The name of the room, shown for a moment after entering it. `t` is the time since entering.
function module.drawRoomName(title: string, subtitle: string, center: vec.Vec2, t: number)
	local alpha = math.min(1, t / 0.25) * math.clamp((2.2 - t) / 0.6, 0, 1)
	if alpha > 0 then
		local shadow = theme.withAlpha(vec4.BLACK, alpha * 0.8)
		drawCentered(subtitle, center + vec.V2(0, 0.14), 0.1, theme.withAlpha(theme.colors.gold, alpha), shadow)
		drawCentered(title, center, 0.18, theme.withAlpha(theme.colors.text, alpha), shadow)
	end
end

function module.drawLevelClear(t: number, duration: number)
	-- The banner grows past its size, then settles, like a spring
	local appear = math.min(1, t / 0.35)
	local scale = math.max(0.01, 0.4 + 0.6 * appear + math.sin(appear * math.pi) * 0.35 * (1 - appear))
	local alpha = math.clamp((duration - t) / 0.3, 0, 1)
	graphics.drawRect(vec.V2(-1, -0.2), vec.V2(2, 0.4), theme.withAlpha(theme.colors.shadow, 0.6 * alpha))
	drawCentered("GEAR FOUND!", vec.V2(0, 0.02), 0.22 * scale, theme.withAlpha(theme.colors.gold, alpha), theme.colors.goldDark)
	local gearSize = ui_components.square(0.2 * scale)
	local spin = math.floor(t * 12) % 4
	sprites.draw("extra/gear", vec.V2(-gearSize.x / 2, 0.2), gearSize, theme.withAlpha(vec4.WHITE, alpha), spin)
end

--- The night sky slowly turns into dawn
function module.drawEnding(turns: number, t: number)
	local dawn = math.clamp(t / 3, 0, 1)
	local top = vec4.createColor(0.08, 0.06, 0.2, 1):lerp(vec4.createColor(0.95, 0.55, 0.45, 1), dawn)
	local bottom = vec4.createColor(0.05, 0.04, 0.1, 1):lerp(vec4.createColor(1, 0.85, 0.55, 1), dawn)
	local bands = 24
	for i = 0, bands - 1 do
		graphics.drawRect(vec.V2(-1, -1 + i * 2 / bands), vec.V2(2, 2 / bands + 0.01), bottom:lerp(top, i / (bands - 1)))
	end
	local sunSize = ui_components.square(0.5)
	sprites.draw("extra/gear", vec.V2(-sunSize.x / 2, -1.1 + dawn * 0.5), sunSize, vec4.createColor(1, 0.9, 0.5, dawn))

	local alpha = math.clamp(t - 1, 0, 1)
	local text = theme.withAlpha(theme.colors.text, alpha)
	local shadow = theme.withAlpha(theme.colors.goldDark, alpha)
	drawCentered("DAWN", vec.V2(0, 0.45), 0.3, text, shadow)
	drawCentered("Plouneuc wakes up. Somewhere, a fairy smiles.", vec.V2(0, 0.2), 0.07, text, shadow)
	drawCentered(`Turns taken: {turns}`, vec.V2(0, 0.05), 0.06, shadow, theme.withAlpha(vec4.BLACK, 0))
end

return module
