--[[
Dialog system: shows lines of text one after the other in a box at the bottom of the screen.

Usage:
	dialog.open(game, { name = "Morgane", sprite = "dungeon/fairy" }, {
		"Hello!",
		function(game) return game:giveItem("sword") end, -- shown as "You got the Sword!"
		"Good luck!",
	})

While a dialog is open, the game is paused and [Space] shows the next line.
See types.DialogStep for what a step can be.
]]
const graphics = require("@vectarine/graphics")
const loader = require("@vectarine/loader")
const settings = require("./settings")
const sprites = require("./sprites")
const theme = require("./theme")
const types = require("./types")
const ui = require("@vectarine/ui")
const ui_components = require("./ui_components")
const vec = require("@vectarine/vec")

const module = loader.init() or {}

type DialogState = {
	game: types.Game,
	speaker: types.Speaker?,
	steps: { types.DialogStep },
	nextStep: number,
	--- Who says the current line. nil for narration lines (returned by function steps).
	lineSpeaker: types.Speaker?,
	text: string,
	--- Number of characters of `text` currently visible (typewriter effect)
	shownChars: number,
	time: number,
}

module.current = module.current :: DialogState?

--- Go to the next line of text, running function steps along the way. Returns false when there is nothing left.
local function goToNextLine(state: DialogState): boolean
	while state.nextStep <= #state.steps do
		local step = state.steps[state.nextStep]
		state.nextStep += 1
		local text: string? = nil
		local speaker: types.Speaker? = nil
		if type(step) == "string" then
			text = step
			speaker = state.speaker
		else
			text = step(state.game)
		end
		if text then
			state.text = text
			state.lineSpeaker = speaker
			state.shownChars = 0
			return true
		end
	end
	return false
end

function module.open(game: types.Game, speaker: types.Speaker?, steps: { types.DialogStep })
	local state: DialogState = {
		game = game,
		speaker = speaker,
		steps = steps,
		nextStep = 1,
		text = "",
		shownChars = 0,
		time = 0,
	}
	if goToNextLine(state) then
		module.current = state
	end
end

function module.isOpen(): boolean
	return module.current ~= nil
end

--- Is the current line still being typed?
function module.isTyping(): boolean
	local state = module.current
	return state ~= nil and state.shownChars < #state.text
end

--- Show the full line if it is being typed, otherwise go to the next line (or close the dialog).
function module.advance()
	local state = module.current
	if not state then
		return
	end
	if module.isTyping() then
		state.shownChars = #state.text
	elseif not goToNextLine(state) then
		module.current = nil
	end
end

function module.close()
	module.current = nil
end

function module.update(dt: number)
	local state = module.current
	if state then
		state.time += dt
		state.shownChars = math.min(#state.text, state.shownChars + dt * settings.values.textSpeed)
	end
end

-- MARK: Drawing

module.BOX_SIZE = vec.V2(1.6, 0.44)
const PADDING = 0.035

module.widget = ui.widget(module.BOX_SIZE, function(_events, extra)
	local state = module.current
	if not state then
		return
	end
	local speaker = state.lineSpeaker
	local portraitSize = ui_components.square(0.2)
	local textWidth = module.BOX_SIZE.x - PADDING * 2 - (if speaker and speaker.sprite then portraitSize.x + PADDING else 0)

	local textColumn = ui.column({ gap = 0.02 }, {
		ui.text(if speaker then speaker.name else "", {
			align = "left",
			font = theme.font,
			fontSize = 0.075,
			color = theme.colors.gold,
		}),
		ui.text(string.sub(state.text, 1, math.floor(state.shownChars)), {
			align = "left",
			font = theme.font,
			fontSize = 0.07,
			color = theme.colors.text,
			maxWidth = textWidth,
			maxHeight = 0.26,
		}),
	})

	local content: { ui.Widget<any> } = {}
	local sprite: string? = if speaker then speaker.sprite else nil
	if sprite then
		table.insert(
			content,
			ui.widget(portraitSize, function()
				sprites.draw(sprite, vec.V2(-1, -1), portraitSize)
			end)
		)
	end
	table.insert(content, textColumn)

	ui.stack({ alignX = "start", alignY = "end" }, {
		ui_components.panel(module.BOX_SIZE),
		ui.row({ padding = PADDING, gap = PADDING, align = "end" }, content),
	}):draw(extra)

	-- Blinking arrow telling the player that they can continue.
	if not module.isTyping() and math.floor(state.time * 2.5) % 2 == 0 then
		local tip = vec.V2(-1 + module.BOX_SIZE.x - PADDING * 1.5, -1 + PADDING * 1.2)
		graphics.drawPolygon({
			tip,
			tip + vec.V2(0.02, 0.035),
			tip + vec.V2(-0.02, 0.035),
		}, theme.colors.gold)
	end
end)

return module
