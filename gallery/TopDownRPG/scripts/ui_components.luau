--[[
Reusable UI widgets: panels, item slots and menus.
]]
const graphics = require("@vectarine/graphics")
const io = require("@vectarine/io")
const loader = require("@vectarine/loader")
const sprites = require("./sprites")
const theme = require("./theme")
const ui = require("@vectarine/ui")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

const module = loader.init() or {}

--- Time in seconds since the game started, used for small animations (blinking, bobbing).
--- It is advanced by main.luau so that animations stay deterministic in automated tests.
module.time = module.time or 0

function module.tick(dt: number)
	module.time += dt
end

--- Size (in OpenGL units) of a square that is `height` tall. OpenGL units are stretched with the window,
--- so a square needs a different width depending on the aspect ratio.
function module.square(height: number): vec.Vec2
	local windowSize = io.getWindowSize()
	return vec.V2(height * windowSize.y / windowSize.x, height)
end

--- Draw a retro panel: a light border around a dark background.
function module.drawPanel(pos: vec.Vec2, size: vec.Vec2, borderColor: vec4.Vec4?, fillColor: vec4.Vec4?)
	local border = module.square(0.012)
	graphics.drawRect(pos, size, borderColor or theme.colors.panelBorder)
	graphics.drawRect(pos + border, size - border:scale(2), fillColor or theme.colors.panel)
end

function module.panel<T>(size: vec.Vec2, borderColor: vec4.Vec4?): ui.Widget<T>
	return ui.widget(size, function()
		module.drawPanel(vec.V2(-1, -1), size, borderColor)
	end)
end

--- A square slot showing an item sprite, used by the HUD and the inventory.
function module.drawItemSlot(pos: vec.Vec2, size: vec.Vec2, sprite: string?, highlighted: boolean?)
	local borderColor = if highlighted then theme.colors.gold else theme.colors.textDim
	module.drawPanel(pos, size, borderColor)
	if sprite then
		local margin = size:scale(0.15)
		sprites.draw(sprite, pos + margin, size - margin:scale(2))
	end
end

-- MARK: Menus

export type MenuItem<T> = {
	label: string | () -> string,
	--- Optional icon from the icons sheet, for example "icons/play"
	icon: string?,
	action: (extra: T) -> (),
}

--- A vertical list of buttons that can be used with the mouse or the keyboard (Up/Down + Space/Enter).
function module.menu<T>(items: { MenuItem<T> }, buttonSize: vec.Vec2): ui.Widget<T>
	const gap = 0.04
	local state = {
		selected = 1,
		--- The mouse only selects buttons when it moves. Otherwise, buttons sliding under a still mouse
		--- during screen transitions would steal the keyboard selection.
		mouseMoved = false,
		lastMouse = io.getMouse(),
	}

	local buttons: { ui.Widget<T> } = {}
	for index, item in items do
		table.insert(
			buttons,
			ui.widget(buttonSize, function(events, extra: T)
				if events.isMouseInside and state.mouseMoved then
					state.selected = index
				end
				if events.isMouseJustReleased then
					state.selected = index
					item.action(extra)
				end
				local selected = state.selected == index
				local pressOffset = if events.isMouseDown then vec.V2(0, -0.01) else vec.V2(0, 0)
				local origin = vec.V2(-1, -1) + pressOffset

				-- Drop shadow, then the button itself
				graphics.drawRect(origin + vec.V2(0, -0.015), buttonSize, theme.colors.shadow)
				module.drawPanel(
					origin,
					buttonSize,
					if selected then theme.colors.gold else theme.colors.textDim,
					if selected then vec4.createColor(0.16, 0.11, 0.07, 0.95) else theme.colors.panel
				)

				local iconSize = module.square(buttonSize.y * 0.5)
				if item.icon then
					sprites.draw(
						item.icon,
						origin + vec.V2(iconSize.x * 0.6, (buttonSize.y - iconSize.y) / 2),
						iconSize,
						if selected then theme.colors.gold else theme.colors.textDim
					)
				end

				local label = if type(item.label) == "function" then item.label() else item.label
				local fontSize = buttonSize.y * 0.62
				local width = theme.font:measureText(label, fontSize).width
				theme.font:drawText(
					label,
					origin + vec.V2((buttonSize.x - width) / 2, buttonSize.y * 0.24),
					fontSize,
					if selected then theme.colors.gold else theme.colors.text
				)

				-- A small knife points at the selected button
				if selected then
					local pointerSize = module.square(buttonSize.y * 0.8)
					local bob = math.sin(module.time * 6) * 0.008
					sprites.draw(
						"items/rusty_knife",
						origin + vec.V2(-pointerSize.x - 0.03 + bob, (buttonSize.y - pointerSize.y) / 2),
						pointerSize
					)
				end
			end)
		)
	end

	local column = ui.column({ align = "center", gap = gap }, buttons)
	local size = vec.V2(buttonSize.x, #items * buttonSize.y + (#items - 1) * gap)

	return ui.widget(size, function(_events, extra: T)
		local mouse = io.getMouse()
		state.mouseMoved = (mouse - state.lastMouse):length() > 0.001
		state.lastMouse = mouse
		if io.isKeyJustPressed("Down") then
			state.selected = state.selected % #items + 1
		elseif io.isKeyJustPressed("Up") then
			state.selected = (state.selected - 2) % #items + 1
		elseif io.isKeyJustPressed("Space") or io.isKeyJustPressed("Return") then
			items[state.selected].action(extra)
		end
		column:draw(extra)
	end)
end

return module
