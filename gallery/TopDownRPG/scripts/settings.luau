--[[
Player settings. They are saved to disk (or to the browser storage on the web) using `persist`,
so they are kept when the game is restarted.
]]
const io = require("@vectarine/io")
const loader = require("@vectarine/loader")
const persist = require("@vectarine/persist")

export type Settings = {
	--- Show the keyboard hints in the bottom-left corner
	showPrompts: boolean,
	--- Characters per second of the dialog typewriter effect
	textSpeed: number,
	fullscreen: boolean,
	screenShake: boolean,
}

const module = loader.init() or {}

const SAVE_NAME = "top_down_rpg.settings"

const defaults: Settings = {
	showPrompts = true,
	textSpeed = 45,
	fullscreen = false,
	screenShake = true,
}

local function loadSettings(): Settings
	local saved = persist.load(SAVE_NAME)
	local values = table.clone(defaults)
	if type(saved) == "table" then
		for key, value in saved :: { [string]: any } do
			if (values :: any)[key] ~= nil and type(value) == type((values :: any)[key]) then
				(values :: any)[key] = value
			end
		end
	end
	return values
end

module.values = module.values or loadSettings() :: Settings

function module.save()
	persist.save(SAVE_NAME, module.values)
end

function module.toggleFullscreen()
	module.values.fullscreen = not module.values.fullscreen
	io.setFullscreen(module.values.fullscreen)
	module.save()
end

return module
