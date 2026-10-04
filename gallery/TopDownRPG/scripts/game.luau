const debug = require("@vectarine/debug")
const graphics = require("@vectarine/graphics")
const loader = require("@vectarine/loader")

--[[
Explorium: a top-down RPG template. An explorer falls into the Explorium, a huge cave system full of the ruins
of the sea people, and tries to find a way out. See GAME_DESIGN_DOC.md for the design.

Where to look:
- data/: the content of the game (levels, terrain, props, items, creatures, characters, lore). Mostly text!
- game_logic.luau: the rules, updated every frame (moving, combat, pickups, exits, needs)
- player.luau, combat.luau, monster_ai.luau, inventory.luau, interactions.luau: the parts of the rules
- level.luau: how the text maps become levels (tilemaps, heights, collisions)
- game_state.luau, progress.luau: new games, entering levels, resting, saving
- world_view.luau, lighting.luau, effects.luau, postfx.luau: how everything is drawn
- game_screen.luau, hud.luau, menus.luau, dialog.luau, prompts.luau: the interface
- title_screen.luau, settings_screen.luau, credits_screen.luau: the menus
- tools/art/: the script that draws the sprites (python3 tools/art/make_art.py)

In the editor console, try "warp colonnade", "give bronze_spear" or "heal".
]]

const mainScriptResource, mainModule = loader.loadScript("scripts/main.luau")

function Update(deltaTime: number)
	if graphics.drawSplashScreenIfNeeded({ mainScriptResource }) then
		return
	end
	if mainModule.Update then
		mainModule.Update(deltaTime)
	end
end
