--[[
Monsters. Their behavior is implemented in monster_ai.luau and referenced by name here.

Everything is turn-based: monsters act right after each action of the player.
Touching a monster is deadly, whether you walk into it or it walks into you.
Monsters die on spikes, in explosions, when shot by an arrow or when a gate closes on them.

- chaser: steps towards the player
- patrol: flies in a straight line and turns around when blocked
- mirror: moves in the opposite direction of the player
- `flying` monsters cross chasms and water, don't die on spikes and don't press plates
]]
const loader = require("@vectarine/loader")

export type Behavior = "chaser" | "patrol" | "mirror"

export type Monster = {
	name: string,
	sprite: string,
	behavior: Behavior,
	flying: boolean?,
}

const module = loader.init() or {}

module.monsters = {
	rat = { name = "Rat", sprite = "dungeon/rat", behavior = "chaser" },
	bat = { name = "Bat", sprite = "dungeon/bat", behavior = "patrol", flying = true },
	spider = { name = "Spider", sprite = "dungeon/spider", behavior = "mirror" },
} :: { [string]: Monster }

return module
