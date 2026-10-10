--[[
The explorer: stats, leveling, health, sanity, thirst and damage.

Stats (see the design document):
- health: maximum health, and health regeneration
- sanity: maximum sanity (used by some items), and sanity regeneration
- strength: more damage with melee weapons, and less damage taken
- agility: moves faster
- intelligence: needed by some items, and more creatures drop items ("luck")

Killing creatures (and giving items to some characters) gives experience. Each level gives a point to spend on a stat.
]]
const effects = require("./effects")
const inventory = require("./inventory")
const items = require("./data/items")
const loader = require("@vectarine/loader")
const prompts = require("./prompts")
const theme = require("./theme")
const types = require("./types")
const vec = require("@vectarine/vec")

const module = loader.init() or {}

--- Seconds of invincibility after being hit
module.INVINCIBILITY = 0.8
--- Water lost per second (100 = full)
module.THIRST_RATE = 0.3
--- Sips in the water bottle, before the waterskin
module.BOTTLE_SIPS = 3
module.SIP_WATER = 40
--- Collision half-size of the player, in tiles
module.RX = 0.28
module.RY = 0.18

function module.new(pos: vec.Vec2): types.Player
	local player: types.Player = {
		pos = pos,
		height = 0,
		facing = "down",
		hp = 0,
		sanity = 0,
		water = 100,
		warmth = 100,
		sips = module.BOTTLE_SIPS,
		stats = { health = 1, sanity = 1, strength = 1, agility = 1, intelligence = 1 },
		level = 1,
		xp = 0,
		statPoints = 0,
		inventory = {},
		equipment = {},
		artefacts = {},
		attackTimer = 0,
		swingTime = 10,
		invincible = 0,
		poison = 0,
		knockback = vec.V2(0, 0),
		velocity = vec.V2(0, 0),
		walkTime = 0,
		moving = false,
	}
	player.hp = module.maxHp(player)
	player.sanity = module.maxSanity(player)
	return player
end

-- MARK: Derived stats

function module.maxHp(player: types.Player): number
	return 20 + (player.stats.health - 1) * 8
end

function module.maxSanity(player: types.Player): number
	return 12 + (player.stats.sanity - 1) * 6
end

--- How many sips the water bottle holds
function module.bottleSize(player: types.Player): number
	return player.bottle or module.BOTTLE_SIPS
end

--- Walking speed, in tiles per second
function module.speed(player: types.Player): number
	return 3.8 + (player.stats.agility - 1) * 0.3
end

--- Health regained per second, out of combat
function module.regen(player: types.Player): number
	local regen = 0.05 * player.stats.health + inventory.charmBonus(player, "regen")
	-- The mender's coil mends wounds three times faster
	return if player.artefacts.menders_coil then regen * 3 else regen
end

function module.sanityRegen(player: types.Player): number
	return 0.15 + 0.08 * player.stats.sanity
end

--- Extra damage of melee attacks
function module.strengthBonus(player: types.Player): number
	return (player.stats.strength - 1) * 1.5
end

--- Chance multiplier for drops
function module.luck(player: types.Player): number
	return 1 + (player.stats.intelligence - 1) * 0.1 + inventory.charmBonus(player, "luck")
end

--- Experience needed to go from `level` to the next one
function module.xpToNext(level: number): number
	return math.floor(15 * level ^ 1.4)
end

-- MARK: Changes

function module.gainXp(game: types.Game, amount: number)
	local player = game.player
	player.xp += amount
	while player.xp >= module.xpToNext(player.level) do
		player.xp -= module.xpToNext(player.level)
		player.level += 1
		player.statPoints += 1
		game.events[#game.events + 1] = { kind = "levelUp", level = player.level }
	end
end

function module.spendPoint(player: types.Player, stat: types.StatName): boolean
	if player.statPoints <= 0 then
		return false
	end
	player.statPoints -= 1
	local hpBefore, sanityBefore = module.maxHp(player), module.maxSanity(player);
	(player.stats :: any)[stat] += 1
	-- Raising a maximum also fills the difference
	player.hp += module.maxHp(player) - hpBefore
	player.sanity += module.maxSanity(player) - sanityBefore
	return true
end

function module.heal(player: types.Player, amount: number)
	player.hp = math.min(module.maxHp(player), player.hp + amount)
end

--- Part of a hit that always gets through, whatever the defense: armor softens hits, it never cancels them
const MIN_DAMAGE = 0.35

--- Damage taken from a hit, after armor and strength
function module.damageTaken(player: types.Player, damage: number, damageType: types.DamageType): number
	local armorId = player.equipment.armor
	local armor = if armorId then items.items[armorId] else nil
	local multiplier = if armor and armor.resist then armor.resist[damageType] or 1 else 1
	local defense = (if armor then armor.defense or 0 else 0) + (player.stats.strength - 1) * 0.5
	local hit = damage * multiplier
	return math.max(1, math.floor(math.max(hit - defense, hit * MIN_DAMAGE) + 0.5))
end

--- Hit the player. `from` is where the hit comes from (to push the player away). Returns true if it hurt.
function module.hurt(
	game: types.Game,
	damage: number,
	damageType: types.DamageType,
	from: vec.Vec2?,
	poison: number?
): boolean
	local player = game.player
	if player.invincible > 0 or player.hp <= 0 then
		return false
	end
	local taken = module.damageTaken(player, damage, damageType)
	player.hp -= taken
	-- Cold hits chill (see frost.luau), unless the ember of the Watch keeps the player warm
	if damageType == "cold" and not player.artefacts.watch_ember then
		player.warmth = math.max(0, player.warmth - taken * 3)
	end
	player.invincible = module.INVINCIBILITY
	-- The bezoar soaks up every poison
	if poison and not player.artefacts.bezoar then
		player.poison = math.max(player.poison, poison)
	end
	if from then
		local away = player.pos - from
		if away:length() > 0.01 then
			player.knockback = away:normalized():scale(6)
		end
	end
	effects.floatText(player.pos + vec.V2(0, -1), `-{taken}`, theme.colors.health)
	effects.burst(player.pos + vec.V2(0, -0.4), "blood", 6)
	effects.shake(0.25)
	effects.flash(theme.colors.health, 0.25)
	if player.hp <= 0 then
		player.hp = 0
		game.events[#game.events + 1] = { kind = "fainted" }
	end
	return true
end

--- Poison the player for `seconds`, unless they carry the bezoar (toxic water, ooze...)
function module.poison(player: types.Player, seconds: number)
	if not player.artefacts.bezoar then
		player.poison = math.max(player.poison, seconds)
	end
end

--- Water below which the player is warned that they are very thirsty
const THIRSTY = 10

--- Warn the player when their water drops below a threshold (each time it does)
local function warnAboutThirst(game: types.Game, before: number, after: number)
	local bag = `Drink a sip from the bottle in the bag of your notebook [{prompts.keyName(prompts.NOTEBOOK_KEY)}], or fill it at a pool.`
	local text = if before > 0 and after <= 0
		then "You are dehydrated. You need to drink water, or you will die."
		elseif before > THIRSTY and after <= THIRSTY then "You are very thirsty. You should drink water."
		else nil
	if text then
		game.events[#game.events + 1] = { kind = "dialog", speaker = nil, steps = { text, bag } }
	end
end

--- Thirst, poison, regeneration. `lit` tells if the player stands in the light (sanity only recovers there).
function module.updateNeeds(game: types.Game, dt: number, lit: boolean)
	local player = game.player
	if player.artefacts.everflowing_amphora then
		player.water = 100
	else
		-- Some floors (the desert) are hotter: thirst comes faster
		local before = player.water
		player.water = math.max(0, player.water - module.THIRST_RATE * game.level.heat * dt)
		warnAboutThirst(game, before, player.water)
	end
	if player.poison > 0 then
		player.poison = math.max(0, player.poison - dt)
		player.hp -= dt * 1
	elseif player.water <= 0 then
		player.hp -= dt * 0.5
	elseif player.invincible <= 0 then
		module.heal(player, module.regen(player) * dt)
	end
	if lit then
		player.sanity = math.min(module.maxSanity(player), player.sanity + module.sanityRegen(player) * dt)
	else
		-- Darkness draining sanity is not fun
		-- player.sanity = math.max(0, player.sanity - 0.25 * dt)
		-- if player.sanity <= 0 then
		-- 	player.hp -= dt * 0.5
		-- end
	end
	if player.hp <= 0 and player.invincible <= 0 then
		player.hp = 0
		game.events[#game.events + 1] = { kind = "fainted" }
	end
end

--- Drink a sip of the water bottle. Returns a message for the player.
function module.drink(player: types.Player): string
	if player.sips <= 0 then
		return "The bottle is empty. Fill it at a pool."
	end
	player.sips -= 1
	player.water = math.min(100, player.water + module.SIP_WATER)
	return `You drink. {player.sips} sips left.`
end

--- Use a consumable from the bag. Returns a message for the player.
function module.useItem(player: types.Player, itemId: string): string?
	local item = items.items[itemId]
	local use = item.use
	if not use then
		return nil
	end
	if not inventory.take(player, itemId, 1) then
		return nil
	end
	if use.heal then
		module.heal(player, use.heal)
	end
	if use.water then
		player.water = math.min(100, player.water + use.water)
	end
	if use.sanity then
		player.sanity = math.min(module.maxSanity(player), player.sanity + use.sanity)
	end
	if use.cure then
		player.poison = 0
	end
	if use.warmth then
		player.warmth = math.min(100, player.warmth + use.warmth)
	end
	if use.dream then
		-- Down to the deep dream (see dream.luau)
		player.sanity = math.min(player.sanity, module.maxSanity(player) * 0.15)
		return "Your mind drifts away... You dream awake."
	end
	if use.bottle then
		player.bottle = module.bottleSize(player) + use.bottle
		player.sips = math.min(module.bottleSize(player), player.sips + use.bottle)
		return `Your bottle holds {module.bottleSize(player)} sips now.`
	end
	return `You use the {string.lower(item.name)}.`
end

return module
