local coord = require("@vectarine/coord")
const loader = require("@vectarine/loader")
local ui = require("@vectarine/ui")
local vec = require("@vectarine/vec")
const module = loader.init() or {}

const keyboardInputIconsImage = loader.loadImage("textures/keyboard-input-icons.png")
const keyboardInputTileset = loader.loadTileset("textures/keyboard-input-icons.tsx")
const keyboardInputIcons = keyboardInputIconsImage:withTileset(keyboardInputTileset)

module.keyboardInputIcons = keyboardInputIcons

--- A widget showing the icon of a keyboard key. `key` is a tile type of keyboard-input-icons.tsx.
function module.widgetForKey(key: string, size: vec.Vec2 | coord.ScreenVec)
	local vecSize: vec.Vec2 = if type(size.x) == "number" then size :: vec.Vec2 else (size :: coord.ScreenVec):gl()

	return ui.widget(vecSize, function(event, extra)
		const p1 = coord.BOTTOM_LEFT:gl()
		const p2 = p1 + vec.V2(vecSize.x, 0)
		const p3 = p1 + vecSize
		const p4 = p1 + vec.V2(0, vecSize.y)
		keyboardInputIcons:drawTileQuad(key, p4, p3, p2, p1)
	end)
end

return module
