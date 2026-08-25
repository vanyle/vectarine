const debug = require("@vectarine/debug")
const graphics = require("@vectarine/graphics")
const loader = require("@vectarine/loader")

--[[
A template for a top-down puzzle RPG, in the style of the first Zelda game.

The clock of Plouneuc stopped at midnight, and time with it: nothing moves unless the player does.
Every step is a turn. Find the 10 gears hidden in the old mine to start the clock again.
Each level is a single-screen puzzle with monsters, spikes, bombs, arrows, potions, plates and gates.

Where to look:
- data/: the content of the game (rooms, levels, terrain, monsters, characters, story). Mostly text!
- rules.luau: what happens during a turn (moving, monsters, bombs, arrows, gates, deaths)
- monster_ai.luau: where monsters want to go
- game_state.luau: the state of the game, levels, undo and restart
- world.luau: how the rooms are turned into a tilemap
- world_view.luau, effects.luau, postfx.luau: how everything is drawn (with lots of juice)
- game_screen.luau: keyboard controls and the flow between levels
- dialog.luau, hud.luau, prompts.luau: the interface
- title_screen.luau, settings_screen.luau, credits_screen.luau: the menus

Automated tests are in testdata/TopDownRPG at the root of the Vectarine repository.
In the editor console, try "level 3", "solve" or "finish 9".
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
