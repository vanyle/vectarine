--[[
Combat: the attacks of the player, damage to creatures, drops and projectiles.

Damage depends on the damage type of the weapon: creatures can resist it or be weak to it (data/monsters.luau).
Weak spots show in gold, resisted hits in grey, so that players learn which weapon to use.

Bombs are dropped lit and blow up after a moment: they hurt everything around (the player too), break crates and
open cracked walls. An arrow that hits a sun eye opens the gate it guards (see switches.luau).

On the frontier: phantoms can only be hit by a dreaming player (see dream.luau), dream weapons only work while
dreaming, cold weapons slow creatures down, and during a round of the snowball gallery (gallery.luau)
attacking throws snowballs at the targets.
]]
const dream = require("./dream")
const effects = require("./effects")
const gallery = require("./gallery")
const inventory = require("./inventory")
const items = require("./data/items")
const level = require("./level")
const loader = require("@vectarine/loader")
const monsters = require("./data/monsters")
const player_rules = require("./player")
const props = require("./data/props")
const switches = require("./switches")
const theme = require("./theme")
const types = require("./types")
const vec = require("@vectarine/vec")

const module = loader.init() or {}

module.DIRECTIONS = {
	up = vec.V2(0, -1),
	down = vec.V2(0, 1),
	left = vec.V2(-1, 0),
	right = vec.V2(1, 0),
} :: { [types.Direction]: vec.Vec2 }

const ANGLES: { [types.Direction]: number } = { right = 0, down = math.pi / 2, left = math.pi, up = -math.pi / 2 }

--- Projectiles: sprite, speed (tiles per second) and how long they fly (seconds)
module.SHOTS = {
	arrow = { sprite = "characters/shot_arrow", speed = 11, life = 0.9 },
	spark = { sprite = "characters/shot_spark", speed = 13, life = 0.7 },
	acid = { sprite = "characters/shot_acid", speed = 5, life = 1.6 },
	rock = { sprite = "characters/shot_rock", speed = 7, life = 1.5 },
	fire = { sprite = "characters/shot_fire", speed = 6, life = 1.4 },
	sand = { sprite = "characters/shot_sand", speed = 6.5, life = 1.1 },
	poison = { sprite = "characters/shot_poison", speed = 5, life = 1.5 },
	icicle = { sprite = "characters/shot_icicle", speed = 12, life = 0.8 },
	dirge = { sprite = "characters/shot_dirge", speed = 8, life = 0.9 },
	snowball = { sprite = "characters/shot_snowball", speed = 12, life = 0.85 },
	shard = { sprite = "characters/shot_shard", speed = 7.5, life = 1.6 },
	frost = { sprite = "characters/shot_frost", speed = 6, life = 1.5 },
} :: { [string]: { sprite: string, speed: number, life: number } }

--- Seconds before a bomb blows up, and the radius of its blast (in tiles)
module.BOMB_FUSE = 1.6
const BLAST_RADIUS = 1.7
--- Damage of a blast to the player (the creatures take the damage of the bomb)
const BLAST_SELF_DAMAGE = 8

--- The point where a body of the player is, to aim from (the middle of the sprite, above the feet)
local function chest(pos: vec.Vec2): vec.Vec2
	return pos + vec.V2(0, -0.35)
end

-- MARK: Damage

local function dropLoot(game: types.Game, monster: types.Monster, species: monsters.Species)
	local luck = player_rules.luck(game.player)
	for _, drop in species.drops or {} do
		if math.random() < drop.chance * luck then
			local scatter = vec.V2(math.random() - 0.5, math.random() - 0.5):scale(0.6)
			table.insert(game.pickups, {
				pos = monster.pos + scatter,
				item = drop.item,
				count = drop.count or 1,
				key = nil,
				age = 0,
				dead = false,
			})
		end
	end
end

local function kill(game: types.Game, monster: types.Monster, species: monsters.Species)
	monster.dead = true
	local notebook = game.notebook.creatures
	notebook[monster.kind] = (notebook[monster.kind] or 0) + 1
	if monster.uniqueKey then
		game.flags[monster.uniqueKey] = true
	end
	local sheet = if species.size == 2 then "bosses" else "characters"
	effects.dying(monster.pos + vec.V2(0, -(species.size or 1) / 2), `{sheet}/{species.sprite}_0`, monster.facing == "left", species.size)
	effects.burst(monster.pos, "poof", 10)
	player_rules.gainXp(game, species.xp)
	effects.floatText(monster.pos + vec.V2(0, -1.2), `+{species.xp} xp`, theme.colors.xp)
	dropLoot(game, monster, species)
	game.events[#game.events + 1] = { kind = "killed", monster = monster.kind, boss = species.boss }
end

--- A creature dies without a fight (a creature that slid into freezing water): experience and drops all the same
function module.killMonster(game: types.Game, monster: types.Monster)
	if not monster.dead then
		kill(game, monster, monsters.species[monster.kind])
	end
end

--- Can the monster be hit right now? (a worm underground can't)
function module.isVulnerable(monster: types.Monster): boolean
	local state = monster.state
	-- Hazards (ooze) can't be hurt, and nothing reaches a frog in the air
	if monsters.species[monster.kind].hazard then
		return false
	end
	return state ~= "hidden" and state ~= "underground" and state ~= "jump" and state ~= "rise" and state ~= "air"
		and state ~= "lock" and state ~= "fall" and state ~= "down" and not monster.dead
end

--- Hit a monster. Returns true if it was hit.
function module.hurtMonster(
	game: types.Game,
	monster: types.Monster,
	damage: number,
	damageType: types.DamageType,
	from: vec.Vec2
): boolean
	if not module.isVulnerable(monster) or not dream.isPresent(game, monster) then
		return false
	end
	local species = monsters.species[monster.kind]
	-- Targets of the snowball gallery count points, they don't get hurt
	if species.prize then
		if gallery.isActive() then
			gallery.hit(game, monster)
		end
		return true
	end
	local multiplier = if species.resist then species.resist[damageType] or 1 else 1
	-- Armored creatures (the Sun Warden) barely take damage, except when they are flooded
	if species.armor then
		multiplier *= if monster.state == "flooded" then 1 else species.armor
	end
	-- The Last Watchman is a husk of ice for those who are awake: only the dreaming can hurt him
	if species.husk and not dream.isDreaming(game) then
		multiplier *= species.husk
	end
	local dealt = math.max(1, math.floor(damage * multiplier + 0.5))
	monster.hp -= dealt
	monster.hitTime = 0
	-- Cold slows creatures down (bosses too, a little less)
	if damageType == "cold" then
		monster.chill = if species.boss then 1 else 2.5
	end
	if not species.boss then
		local away = monster.pos - from
		if away:length() > 0.01 then
			monster.knockback = away:normalized():scale(3.5)
		end
	end
	local color = if multiplier > 1 then theme.colors.gold elseif multiplier < 1 then theme.colors.textDim else theme.colors.text
	effects.floatText(monster.pos + vec.V2(0, -(species.size or 1)), tostring(dealt), color)
	effects.animate(monster.pos + vec.V2(0, -0.4), { "characters/hit_0", "characters/hit_1" }, 0.15)
	effects.shake(0.08)
	-- Seeing a creature adds it to the bestiary
	game.notebook.creatures[monster.kind] = game.notebook.creatures[monster.kind] or 0
	if monster.hp <= 0 then
		kill(game, monster, species)
	end
	return true
end

-- MARK: Player attacks

local function breakProp(game: types.Game, prop: types.Prop)
	local kind = props.kinds[prop.kind]
	game.flags[`broken:{prop.key}`] = true
	level.clearProp(game.level :: any, prop.cell)
	local center = prop.cell + vec.V2(0.5, 0.6)
	effects.burst(center, "debris", 10)
	for _, drop in kind.drops or {} do
		if math.random() < drop.chance * player_rules.luck(game.player) then
			table.insert(game.pickups, { pos = center, item = drop.item, count = drop.count or 1, age = 0, dead = false })
		end
	end
end

--- The prop at a cell, unless it was broken
function module.propAt(game: types.Game, x: number, y: number): types.Prop?
	local prop = level.propAt(game.level :: any, x, y)
	if prop and game.flags[`broken:{prop.key}`] then
		return nil
	end
	return prop
end

--- Attack with the equipped weapon. Returns a message for the player if the attack is not possible.
function module.attack(game: types.Game): string?
	local player = game.player
	if player.attackTimer > 0 then
		return nil
	end
	local weaponId = inventory.weapon(player)
	local weapon = items.items[weaponId]
	local direction = module.DIRECTIONS[player.facing]

	-- The snowball gallery: whatever the weapon, the player throws snowballs
	if gallery.isActive() then
		local shot = module.SHOTS.snowball
		player.attackTimer = 0.5
		player.swingTime = 0
		table.insert(game.projectiles, {
			pos = chest(player.pos) + direction:scale(0.4),
			velocity = direction:scale(shot.speed),
			damage = 1,
			damageType = "cold" :: types.DamageType,
			fromPlayer = true,
			sprite = shot.sprite,
			life = shot.life,
			dead = false,
		})
		return nil
	end

	-- Dream weapons only exist in a dreaming hand
	if weapon.dreamOnly and not dream.isDreaming(game) then
		player.attackTimer = 0.4
		return "Your mind is too clear: your fingers close on nothing. (Dream weapons need a dreaming mind.)"
	end

	if weapon.bomb then
		if not inventory.take(player, weaponId, 1) then
			return "No bombs left"
		end
		player.attackTimer = weapon.cooldown or 0.8
		table.insert(game.bombs, { pos = player.pos + direction:scale(0.6), fuse = module.BOMB_FUSE, damage = weapon.damage or 30 })
		return nil
	end

	if weapon.shot then
		if weapon.ammo and not inventory.take(player, weapon.ammo, 1) then
			return `No {items.items[weapon.ammo].name:lower()} left`
		end
		if weapon.sanityCost then
			if player.sanity < weapon.sanityCost then
				return "Your mind is too tired"
			end
			player.sanity -= weapon.sanityCost
		end
		player.attackTimer = weapon.cooldown or 0.5
		player.swingTime = 0
		local shot = module.SHOTS[weapon.shot]
		table.insert(game.projectiles, {
			pos = chest(player.pos) + direction:scale(0.4),
			velocity = direction:scale(shot.speed),
			damage = (weapon.damage or 1) + (if weapon.shot == "spark" then (player.stats.intelligence - 1) else 0),
			damageType = (weapon.damageType or "pierce") :: types.DamageType,
			fromPlayer = true,
			sprite = shot.sprite,
			life = shot.life,
			dead = false,
		})
		return nil
	end

	player.attackTimer = weapon.cooldown or 0.4
	player.swingTime = 0
	local reach = weapon.reach or 0.8
	local center = chest(player.pos) + direction:scale(reach * 0.6)
	local radius = reach * 0.65
	effects.animate(
		center,
		{ "characters/slash_0", "characters/slash_1", "characters/slash_2" },
		0.18,
		ANGLES[player.facing],
		reach
	)
	local damage = (weapon.damage or 1) + player_rules.strengthBonus(player)
	for _, monster in game.monsters do
		local species = monsters.species[monster.kind]
		local body = monster.pos + vec.V2(0, -0.3 * (species.size or 1))
		if not monster.dead and (body - center):length() < radius + species.radius then
			module.hurtMonster(game, monster, damage, (weapon.damageType or "blunt") :: types.DamageType, player.pos)
		end
	end
	-- Break pots and crates in front of the player
	local cell = level.cellOf(player.pos + direction:scale(0.8))
	local prop = module.propAt(game, cell.x, cell.y)
	if prop and props.kinds[prop.kind].breakable then
		breakProp(game, prop)
	end
	return nil
end

-- MARK: Bombs

local function explode(game: types.Game, bomb: types.Bomb)
	local lvl = game.level :: any
	local pos = bomb.pos
	effects.animate(pos + vec.V2(0, -0.3), { "characters/explosion_0", "characters/explosion_1", "characters/explosion_2" }, 0.45, 0, 2.6)
	effects.burst(pos, "debris", 18)
	effects.shake(0.6)
	for _, monster in game.monsters do
		local species = monsters.species[monster.kind]
		if not monster.dead and (monster.pos - pos):length() < BLAST_RADIUS + species.radius then
			module.hurtMonster(game, monster, bomb.damage, "blunt", pos)
		end
	end
	if (game.player.pos - pos):length() < BLAST_RADIUS then
		player_rules.hurt(game, BLAST_SELF_DAMAGE, "blunt", pos)
	end
	-- Cracked walls open, crates and pots break
	local cell = level.cellOf(pos)
	local opened = false
	for y = cell.y - 2, cell.y + 2 do
		for x = cell.x - 2, cell.x + 2 do
			local center = vec.V2(x + 0.5, y + 0.5)
			if (center - pos):length() < BLAST_RADIUS + 0.4 then
				if level.breakWall(lvl, x, y) then
					game.flags[level.crackedKey(lvl.id, x, y)] = true
					effects.burst(center, "debris", 14)
					opened = true
				end
				local prop = module.propAt(game, x, y)
				if prop and props.kinds[prop.kind].breakable then
					breakProp(game, prop)
				end
			end
		end
	end
	if opened then
		game.events[#game.events + 1] = { kind = "message", text = "The cracked wall crumbles! There was a way behind it." }
	end
end

function module.updateBombs(game: types.Game, dt: number)
	for i = #game.bombs, 1, -1 do
		local bomb = game.bombs[i]
		bomb.fuse -= dt
		if bomb.fuse <= 0 then
			table.remove(game.bombs, i)
			explode(game, bomb)
		end
	end
end

-- MARK: Creatures hurting the player

--- How hard bosses hit, depending on the health of the player: hard while they are healthy, softer when they are low.
--- A boss fight is scary (it brings the player down to about a third of their health), but rarely deadly.
function module.bossFactor(player: types.Player): number
	local fraction = math.clamp(player.hp / player_rules.maxHp(player), 0, 1)
	return 0.45 + 0.9 * fraction
end

--- A creature hurts the player (bosses hit softer when the player is low, see bossFactor)
function module.creatureHurts(
	game: types.Game,
	monster: types.Monster,
	damage: number,
	damageType: types.DamageType,
	from: vec.Vec2?,
	poison: number?
): boolean
	local scaled = if monsters.species[monster.kind].boss then damage * module.bossFactor(game.player) else damage
	return player_rules.hurt(game, scaled, damageType, from, poison)
end

-- MARK: Projectiles

--- A creature shoots at a point
function module.monsterShoot(game: types.Game, monster: types.Monster, target: vec.Vec2, angleOffset: number?)
	local species = monsters.species[monster.kind]
	local shot = module.SHOTS[species.shot or "acid"]
	local from = monster.pos + vec.V2(0, -0.4)
	local direction = (target - from)
	if direction:length() < 0.01 then
		return
	end
	local angle = direction:angle() + (angleOffset or 0)
	table.insert(game.projectiles, {
		pos = from,
		velocity = vec.fromAngle(angle, shot.speed),
		damage = species.damage,
		damageType = species.damageType,
		fromPlayer = false,
		sprite = shot.sprite,
		life = shot.life,
		poison = species.poison,
		boss = species.boss,
		dead = false,
	})
end

function module.updateProjectiles(game: types.Game, dt: number)
	local lvl = game.level :: any
	for _, projectile in game.projectiles do
		projectile.pos = projectile.pos + projectile.velocity:scale(dt)
		projectile.life -= dt
		local cell = level.cellOf(projectile.pos + vec.V2(0, 0.35))
		local wall = level.wallAt(lvl, cell.x, cell.y)
		local prop = module.propAt(game, cell.x, cell.y)
		local blocked = not level.inside(lvl, cell.x, cell.y)
			or (wall ~= nil and not wall.walkable)
			or (prop ~= nil and props.kinds[prop.kind].solid and not props.kinds[prop.kind].flat)
		if projectile.life <= 0 or blocked then
			projectile.dead = true
			effects.burst(projectile.pos, "debris", 4)
			-- An arrow wakes up a sun eye, which opens what it guards
			local kind = if prop then props.kinds[prop.kind] else nil
			if blocked and prop and kind and kind.target and projectile.fromPlayer and projectile.damageType == "pierce" then
				if switches.open(game, prop.data.opens) then
					game.events[#game.events + 1] = { kind = "message", text = "The bronze eye opens wide. Somewhere, stone grinds against stone." }
				end
			end
			continue
		end
		if projectile.fromPlayer then
			for _, monster in game.monsters do
				local species = monsters.species[monster.kind]
				local body = monster.pos + vec.V2(0, -0.3 * (species.size or 1))
				if not monster.dead and (body - projectile.pos):length() < species.radius + 0.2 then
					if module.hurtMonster(game, monster, projectile.damage, projectile.damageType, projectile.pos) then
						projectile.dead = true
						break
					end
				end
			end
		else
			local body = chest(game.player.pos)
			if (body - projectile.pos):length() < 0.4 then
				projectile.dead = true
				local damage = if projectile.boss then projectile.damage * module.bossFactor(game.player) else projectile.damage
				player_rules.hurt(game, damage, projectile.damageType, projectile.pos, projectile.poison)
			end
		end
	end
	for i = #game.projectiles, 1, -1 do
		if game.projectiles[i].dead then
			table.remove(game.projectiles, i)
		end
	end
end

return module
