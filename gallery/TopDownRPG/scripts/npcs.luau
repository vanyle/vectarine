--[[
Friendly characters: what they say, what they trade and the quests they give.
(This file is with the rules rather than in data/ because the dialogs give items and experience.)

`talk(game)` returns the lines of the dialog (see dialog.luau). It reads and changes `game.flags` to remember where the
story is: a line can be a function that gives an item or starts a quest, and returns a line of narration.

`shop` lists what a trader sells: `price` is paid with `currency` (an item id).
]]
const gallery = require("./gallery")
const inventory = require("./inventory")
const items = require("./data/items")
const level = require("./level")
const levels = require("./data/levels")
const loader = require("@vectarine/loader")
const player_rules = require("./player")
const types = require("./types")

export type Offer = { item: string, count: number, price: number, currency: string }

export type Npc = {
	name: string,
	--- Prefix of the frames in the "characters" sheet (wren_0, wren_1)
	sprite: string,
	talk: (game: types.Game) -> { types.DialogStep },
	shop: { Offer }?,
	--- Only there for a dreaming player (a phantom, see dream.luau)
	dream: boolean?,
}

const module = loader.init() or {}

--- A dialog step that gives an item and tells the player
local function give(itemId: string, count: number?): types.DialogStep
	return function(game: types.Game): string?
		local left = inventory.give(game.player, itemId, count or 1)
		local name = items.items[itemId].name
		if items.items[itemId].kind == "artefact" then
			game.events[#game.events + 1] = { kind = "artefact", item = itemId }
		end
		if left > 0 then
			table.insert(game.pickups, { pos = game.player.pos, item = itemId, count = left, age = 0, dead = false })
			return `Your bag is full: the {string.lower(name)} is on the ground.`
		end
		return if (count or 1) > 1 then `You got {count} × {name}!` else `You got: {name}!`
	end
end

local function xp(amount: number): types.DialogStep
	return function(game: types.Game): string?
		player_rules.gainXp(game, amount)
		return `+{amount} experience`
	end
end

local function setFlag(name: string, value: any): types.DialogStep
	return function(game: types.Game): string?
		game.flags[name] = value
		return nil
	end
end

--- The dream runes where the squad of Corporal Vey carved their names (data/lore.luau)
module.WATCH_NAMES = { "watch_name_orren", "watch_name_maelis", "watch_name_tobin", "watch_name_astra" }

--- Where each of them stood guard (Corporal Vey tells the player)
module.WATCH_POSTS = { "at the frozen stair", "on the frozen lake", "in the ice caves", "on the walls of the Watch" }

--- The places of the frontier where a brazier is still dark (for Brann the lamplighter)
function module.darkPlaces(game: types.Game): { string }
	local places = {}
	for _, id in levels.order do
		local data = levels.levels[id]
		if data.depth == 4 then
			for _, prop in level.load(id, {}).props do
				if prop.kind == "brazier" and not game.flags[`lit:{prop.key}`] then
					table.insert(places, data.name)
					break
				end
			end
		end
	end
	return places
end

--- The braziers of the frontier: how many are lit, and how many there are (for Brann the lamplighter)
function module.braziers(game: types.Game): (number, number)
	local lit, total = 0, 0
	for id, data in levels.levels do
		if data.depth == 4 then
			for _, prop in level.load(id, {}).props do
				if prop.kind == "brazier" then
					total += 1
					if game.flags[`lit:{prop.key}`] then
						lit += 1
					end
				end
			end
		end
	end
	return lit, total
end

module.npcs = {
	wren = {
		name = "Wren",
		sprite = "wren",
		talk = function(game)
			local flags = game.flags
			if not flags.wrenMet then
				return {
					"Well, well. Another one fell through the sinkhole? Don't look up, it only makes it worse.",
					"I'm Wren. I came down here forty years ago, looking for the city of the sea people.",
					"I found a fig tree, a bad knee and a lot of rats. Welcome to the Explorium.",
					"Keep your bottle full: the pools are clean. Rest by my fire when you're hurt, it's the only safe place around.",
					"The way out? Up is too far. The only way is down, through the den of the Great Mole. She doesn't like visitors.",
					"While you're wandering... I lost my pack in the Fern Hollow, south of here. My maps are in it. Bring it back?",
					setFlag("wrenMet", true),
					setFlag("questPack", "started"),
				}
			end
			if flags.questPack == "started" and inventory.count(game.player, "wren_pack") > 0 then
				return {
					"My pack! And the maps are dry. You're a good one.",
					function(g: types.Game): string?
						inventory.take(g.player, "wren_pack", 1)
						g.flags.questPack = "done"
						return nil
					end,
					"Here, the best thing in it: this lens. Sea people work, found it in the ruins east of here.",
					"Look through it while you walk, and the places you've seen draw themselves. That's how I made my maps.",
					give("cartographer_lens"),
					"Open your notebook and see for yourself. And take my old vest too. It stopped a lot of teeth. It'll stop more.",
					give("leather_vest"),
					xp(25),
				}
			end
			-- Saves from before the lens was Wren's reward
			if flags.questPack == "done" and not game.player.artefacts.cartographer_lens then
				return {
					"Wait, I almost forgot! The best thing in my pack: this lens. It draws the places you've seen.",
					give("cartographer_lens"),
				}
			end
			if flags.questPack == "started" then
				return {
					"My pack is in the Fern Hollow, south of the camp. Cross the stream at the ford, then go to the far south-east corner, past the tall grass.",
					"That's where a snake chased me and I dropped it. Mind the snakes: they hide in the tall grass, and they coil before they strike.",
				}
			end
			if game.flags["killed:great_mole"] then
				return {
					"You beat the Great Mole? Ha! Forty years I've been hiding from her.",
					"Go on, then. Go down. And if you find the city... tell me if it's as beautiful as I dreamt.",
				}
			end
			return {
				"Ilse, the scientist, camps in the old colonnade to the east. She pays well for fragments of the ruins.",
				"And the ant, Click? Trades anything for figs. I don't ask what an ant does with that many figs.",
			}
		end,
	},

	click = {
		name = "Click",
		sprite = "click",
		talk = function(game)
			if not game.flags.clickMet then
				game.flags.clickMet = true
				return {
					"*click click* ... You. Two legs. Smell of figs? *click*",
					"Click trades. Figs for things. Things for figs. *happy clicking*",
				}
			end
			return {
				"*click* Figs? Figs! Click has things. *click click*",
				"Cousin Clack trades in the dry halls, down below. Clack sells boom-booms! Click doesn't like boom-booms.",
			}
		end,
		shop = {
			{ item = "bandage", count = 1, price = 2, currency = "fig" },
			{ item = "herb_tea", count = 1, price = 3, currency = "fig" },
			{ item = "arrows", count = 5, price = 2, currency = "fig" },
			{ item = "firefly_jar", count = 1, price = 6, currency = "fig" },
			{ item = "vigor_ring", count = 1, price = 10, currency = "fig" },
			{ item = "bone_bow", count = 1, price = 12, currency = "fig" },
		},
	},

	ilse = {
		name = "Dr. Ilse",
		sprite = "ilse",
		talk = function(game)
			local flags = game.flags
			local steps: { types.DialogStep } = {}
			if flags.questPack == "started" and inventory.count(game.player, "wren_pack") > 0 then
				table.insert(steps, "Wait, that pack... it's Wren's! He has been grumbling about it for weeks. Take it back to his camp, before he limps all the way here with his bad knee.")
			end
			if not flags.ilseMet then
				flags.ilseMet = true
				table.insert(steps, "Oh! A living person. Sorry, I talk to the columns too much. Dr. Ilse, university of the surface.")
				table.insert(steps, "My expedition came down three years ago. I'm what's left of it. But look around you!")
				table.insert(steps, "The sea people built this. Engineers, astronomers... and when their sea dried up, they followed the water down.")
				table.insert(steps, "Bring me the fragments you find in the ruins. Every piece of writing tells me more. I'll teach you what I learn.")
				table.insert(steps, "Bring me three, and I'll give you something to deal with those bronze beetles.")
			end
			local fragments = inventory.count(game.player, "ancient_fragment")
			if fragments > 0 then
				table.insert(steps, function(g: types.Game): string?
					inventory.take(g.player, "ancient_fragment", fragments)
					g.flags.fragmentsGiven = (g.flags.fragmentsGiven or 0) + fragments
					player_rules.gainXp(g, fragments * 10)
					return `You give {fragments} fragment{if fragments > 1 then "s" else ""} to Ilse. +{fragments * 10} experience`
				end)
				table.insert(steps, "Wonderful. This symbol... it means 'tide'. Or 'mother'. They used the same word.")
			end
			local given: number = flags.fragmentsGiven or 0
			if given + fragments >= 3 and not flags.ilseReward then
				table.insert(steps, "Three fragments! As promised: a spark caster. I found it in the eastern halls, still humming.")
				table.insert(steps, "It turns thoughts into lightning. Those bronze beetles hate it: metal shells and lightning don't mix.")
				table.insert(steps, "It takes a steady mind, though. Train your sanity before you try it, or it will burn you out.")
				table.insert(steps, give("spark_caster"))
				table.insert(steps, setFlag("ilseReward", true))
			end
			if not flags.ilseReward and given + fragments < 3 then
				table.insert(steps, `{given + fragments} of 3 fragments so far. The chests of these halls hold some: one behind the vines, others in the eastern halls.`)
				table.insert(steps, "Break the pots of the ruins too, the sea people kept their writings in them. And the beetles swallow fragments with the columns they eat.")
			elseif #steps == 0 then
				table.insert(steps, "Fragments, tablets, anything with writing on it. Each one tells me more about the sea people.")
			end
			return steps
		end,
	},

	unit7 = {
		name = "Unit-7",
		sprite = "unit7",
		talk = function(game)
			local flags = game.flags
			local killed: number = game.notebook.creatures.beetle_bronze or 0
			if not flags.unit7 then
				return {
					"UNIT-7 ONLINE. LAST INSTRUCTION: PROTECT THE HALLS OF THE TIDE.",
					"STATUS: HALLS UNDER ATTACK. BRONZE BEETLES ARE EATING THE COLUMNS. UNIT-7 ARMS: MISSING.",
					"REQUEST: DESTROY FOUR BRONZE BEETLES. REWARD: ONE ENERGY CORE. TELEPORTERS ACCEPT ENERGY CORES.",
					setFlag("unit7", "started"),
					setFlag("unit7KillsAtStart", killed),
				}
			end
			if flags.unit7 == "started" then
				local start: number = flags.unit7KillsAtStart or 0
				local done = killed - start
				if done >= 4 then
					return {
						"BEETLES DESTROYED: FOUR. THE HALLS ARE SAFE. UNIT-7 IS... GRATEFUL.",
						give("energy_core"),
						xp(30),
						setFlag("unit7", "done"),
					}
				end
				return {
					`BEETLES DESTROYED: {math.max(0, done)} OF FOUR. THEY ARE CRAWLING IN THE EASTERN HALLS.`,
					"ANALYSIS: BLADES SLIDE ON BRONZE SHELLS. A CLUB CRACKS THEM. LIGHTNING CRACKS THEM BETTER.",
				}
			end
			return {
				"THE CHILDREN OF THE TIDE WENT DOWN THE STAIRS. UNIT-7 STAYED. UNIT-7 WAITS.",
				"THE GREAT TUNNELER GUARDS THE STAIRS. SHE WAS A FRIEND, ONCE. SHE FORGOT.",
			}
		end,
	},

	-- MARK: The desert

	queen = {
		name = "Queen Tikra",
		sprite = "queen",
		talk = function(game)
			local flags = game.flags
			if not flags.queenMet then
				return {
					"*click*... A two-legs, alive, in the dry halls. I am Tikra, queen of this colony.",
					"Long ago the sea people made us dig these halls. Their Sun Warden watched us, day and night. It still does.",
					"When the sea people left, the Great Pump kept our cistern full. Then the sand ate its valves, and the water stopped.",
					"We drink the last drops of the cistern. Help yourself, friend: water shared is never lost.",
					"The Great Pump is past the dry canals. Its gate is sealed, its valves are stuck. Make the water flow again, and the colony will give you its greatest treasure.",
					"The sea people wrote down how their machines work. Ossa, our scribe, can teach you to read their words.",
					"And whatever you do, do not face the Sun Warden while the colony is thirsty. It drinks the water of those who fight it.",
					setFlag("queenMet", true),
					setFlag("questPump", "started"),
				}
			end
			if flags.waterRestored and flags.questPump ~= "done" then
				return {
					"Water! I hear it singing in the walls! The cistern is full for the first time in my life. *click click*",
					"Take this: the everflowing amphora. The sea people gave it to their overseers. We stole it, and kept it hidden. Its water never ends.",
					give("everflowing_amphora"),
					xp(50),
					setFlag("questPump", "done"),
					"One thing still keeps us slaves: the Sun Warden. It waits in its hall, beyond the storm dunes. If you are strong enough... free us.",
					setFlag("questWarden", "started"),
				}
			end
			if game.flags["killed:sun_warden"] and flags.questWarden ~= "done" then
				return {
					"The Warden is silent? ... For the first time in a thousand years, no ant has to dig for anyone.",
					"You are a friend of the colony, forever. The way down is through the Howling Gorge: with the Warden's eye, the storm will part for you.",
					xp(60),
					setFlag("questWarden", "done"),
				}
			end
			if flags.questWarden == "done" then
				return { "Go well, friend. The colony will sing about you. Badly, but loudly. *click*" }
			end
			if flags.questWarden == "started" then
				return {
					"The hall of the Sun Warden is beyond the storm dunes. It will try to drink you dry...",
					"...but the amphora never empties. Let it drink, friend. Let it drink until it chokes.",
				}
			end
			return {
				"The pump is past the dry canals. Its gate is covered with the words of the sea people: learn to read them, Ossa can help.",
				"Do not go near the Sun Warden before the water flows again. Our bravest died thirsty at its feet.",
			}
		end,
	},

	ossa = {
		name = "Ossa the scribe",
		sprite = "ossa",
		talk = function(game)
			local flags = game.flags
			local steps: { types.DialogStep } = {}
			if not flags.ossaMet then
				flags.ossaMet = true
				table.insert(steps, "Ah, a reader? I am Ossa, scribe of the colony. I keep what is left of our archive.")
				table.insert(steps, "The sea people burned it when we rebelled. Only our lexicon survived: the key to their language. And three of its pages are lost.")
				table.insert(steps, "Clack's scouts saw one in the dry canals, behind a cracked wall. Another is in the bazaar, among the bronze guards. And little Pip found one, but Pip trades, Pip never gives.")
				table.insert(steps, "Bring me the three pages and I will bind the lexicon for you. The tablets of the sea people will speak to you, even those of the oasis above.")
			end
			local pages = inventory.count(game.player, "lexicon_page")
			local given: number = flags.lexiconPages or 0
			-- A failsafe: once the three pages have been found (the two chests opened, Pip fed), the lexicon is whole,
			-- even if a page got lost on the way (left in a chest, dropped...). Pages still around are bound into it.
			local allFound = flags.pipFed and flags["opened:dry_canals:2,29"] and flags["opened:bazaar:44,28"]
			if allFound and not game.player.artefacts.tide_lexicon then
				table.insert(steps, function(g: types.Game): string?
					inventory.take(g.player, "lexicon_page", pages)
					local quantum = g.chests.quantum or {}
					local stored = 0
					for _, stack in quantum do
						if stack.id == "lexicon_page" then
							stored += stack.count
						end
					end
					inventory.removeFrom(quantum, "lexicon_page", stored)
					g.flags.lexiconPages = 3
					return "You give Ossa what you found of the lexicon."
				end)
				table.insert(steps, "The three pages... I recognize them all, even the one you left behind. The lexicon is whole again! Take it, two-legs. Read everything, and remember us when you do.")
				table.insert(steps, give("tide_lexicon"))
				table.insert(steps, xp(40))
				return steps
			end
			if pages > 0 and not game.player.artefacts.tide_lexicon then
				table.insert(steps, function(g: types.Game): string?
					inventory.take(g.player, "lexicon_page", pages)
					g.flags.lexiconPages = given + pages
					return `You give {pages} page{if pages > 1 then "s" else ""} to Ossa.`
				end)
				if given + pages >= 3 then
					table.insert(steps, "The lexicon is whole again! Take it, two-legs. Read everything, and remember us when you do.")
					table.insert(steps, give("tide_lexicon"))
					table.insert(steps, xp(40))
				else
					table.insert(steps, `{given + pages} of 3 pages. Keep looking, please.`)
				end
				return steps
			end
			if game.player.artefacts.tide_lexicon then
				table.insert(steps, "Read everything, two-legs. The sea people wrote down how to open every door they built.")
				return steps
			end
			table.insert(steps, `{given} of 3 pages. The gate of the Great Pump, the door of the council... the sea people carved words next to every door.`)
			if not flags["opened:dry_canals:2,29"] then
				table.insert(steps, "One page is in the dry canals, behind a cracked wall in the far south-west. Clack's scouts saw it.")
			end
			if not flags["opened:bazaar:44,28"] then
				table.insert(steps, "One is in a chest of the bazaar, in the court of the bronze sentinels. The chest stays sealed while they stand.")
			end
			if not flags.pipFed then
				table.insert(steps, "And little Pip, in the colony, keeps the last one. Pip wants a prickly pear: the cacti of the canals grow them, Clack sells them.")
			end
			return steps
		end,
	},

	clack = {
		name = "Clack",
		sprite = "clack",
		talk = function(game)
			if not game.flags.clackMet then
				game.flags.clackMet = true
				return {
					"*click* A customer! Cousin Click told me about two-legs. Figs? Figs!",
					"Clack sells bombs. Boom-boom! Walls with cracks are hollow: one bomb, and the wall is a door. Step back first.",
					"My scouts saw cracks in the south-west of the dry canals. Boom-boom, and who knows what's behind?",
				}
			end
			return { "*click* Bombs, pears, arrows. Clack has everything, for figs. Cracks in walls? Boom-boom!" }
		end,
		shop = {
			{ item = "bomb", count = 2, price = 3, currency = "fig" },
			{ item = "prickly_pear", count = 1, price = 2, currency = "fig" },
			{ item = "arrows", count = 5, price = 2, currency = "fig" },
			{ item = "herb_tea", count = 1, price = 3, currency = "fig" },
			{ item = "bandage", count = 1, price = 2, currency = "fig" },
			{ item = "desert_wraps", count = 1, price = 10, currency = "fig" },
			{ item = "bone_bow", count = 1, price = 12, currency = "fig" },
		},
	},

	pip = {
		name = "Pip",
		sprite = "pip",
		talk = function(game)
			local flags = game.flags
			if not flags.pipFed and inventory.count(game.player, "prickly_pear") > 0 then
				return {
					"A pear! For me? *crunch crunch crunch* Thank you thank you!",
					function(g: types.Game): string?
						inventory.take(g.player, "prickly_pear", 1)
						g.flags.pipFed = true
						return "You give a prickly pear to Pip."
					end,
					"Here, the funny paper! It has the old squiggles on it.",
					give("lexicon_page"),
				}
			end
			if not flags.pipFed then
				return {
					"*click!* Hi! You're tall! Do you have a pear? I'm sooo thirsty. I found a funny paper, you can have it for a pear!",
				}
			end
			return {
				"Mom says the big bronze man drinks people. He drinks them until they're dry like a leaf! *shiver*",
				"Mom says nobody can beat him while we're thirsty. When the water comes back, maybe.",
			}
		end,
	},

	mandi = {
		name = "Mandi",
		sprite = "mandi",
		talk = function(game)
			if game.flags["killed:sun_warden"] then
				return { "The Warden is down? *click* ... I guess I'll need a new job. Thank you, two-legs." }
			end
			if not game.flags.mandiMet then
				game.flags.mandiMet = true
				return {
					"Halt. ...A two-legs. Fine, the queen says friends may pass. I am Mandi, I guard the colony.",
					"Scorpions: blades slide on their shell. Hit them with something heavy, and keep away from the tail when it rises.",
					"East of the bazaar are the storm dunes. When the sand rises, you see nothing. Keep to the rocks: storms pass.",
					"Beyond the dunes is the Howling Gorge. Its wind throws you back. They say only the Warden's eye sees through it.",
					"And the Warden... it drinks the water of those who fight it, and its bronze heals. Don't go before the water flows again.",
				}
			end
			return {
				"Sentinels fire when their lens glows. Hit them between two shots. Better: bring something heavy, or lightning.",
				"When the sand rises on the dunes, the wind is stronger than legs. Wait behind a rock until it passes.",
			}
		end,
	},

	mara = {
		name = "Mara",
		sprite = "mara",
		talk = function(game): { types.DialogStep }
			local flags = game.flags
			if flags.maraHelped then
				return {
					"Three dead palms leaning together, by the north wall of the dunes. Water behind the rock. Bring a bomb!",
					"I'll rest here a while. Then I'll go and see these ants.",
				}
			end
			local player = game.player
			local hasWater = player.sips > 0 or inventory.count(player, "prickly_pear") > 0
			if not hasWater then
				return { "Water... please. Anything. I've been stuck in this bazaar for days... if you find any water, please..." }
			end
			local steps: { types.DialogStep } = {
				"Water... please. Anything. I've been stuck in this bazaar for days...",
				function(g: types.Game): string?
					if g.player.sips > 0 then
						g.player.sips -= 1
						return "You give Mara a sip from your bottle."
					end
					inventory.take(g.player, "prickly_pear", 1)
					return "You give Mara a prickly pear."
				end,
				"Oh... thank you. I'm Mara. I wrote the note you may have found up in the sinkhole. I did go east... and down, and down, as far as the cold below the marshes.",
				"Then I came back up. I wanted to see the sun once more. The desert ran me dry before I could.",
				"Take my spare waterskin. Two more sips in your bottle: you'll need them more than I do now.",
				give("waterskin"),
				xp(20),
				setFlag("maraHelped", true),
				"And listen: in the storm dunes, by the north wall, three dead palms lean together. I heard water behind the rock there. The wall is cracked... I had nothing to break it with.",
			}
			return steps
		end,
	},

	-- MARK: The wetlands

	nettle = {
		name = "Nettle",
		sprite = "nettle",
		talk = function(game): { types.DialogStep }
			local flags = game.flags
			local player = game.player
			local steps: { types.DialogStep } = {}
			if not flags.nettleMet then
				flags.nettleMet = true
				table.insert(steps, "Hm? A two-legs, dripping on my floor. Wipe your feet. I am Nettle.")
				table.insert(steps, "My grandmothers cared for the Ebb, the sick the city threw down here. Herbs, broths, and a kind word. More than the healers ever gave them.")
				table.insert(steps, "The marsh is sick too. Its heart is the Frog King, fat on the poisons of the House of Healing.")
				table.insert(steps, "To go down, you must cross the fen. Its water eats you from the inside... unless you hold the stone that grows in the King's belly.")
				table.insert(steps, "The King lives on an island of the great mire: you'll need a boat. The ferry of the old ones sleeps at the docks, but the air there is rotten. The healers had masks for that. Look in their House.")
				table.insert(steps, "And if you find bog lilies, bring me four: I'll brew you my tonic.")
			end
			if inventory.count(player, "bog_lily") >= 4 and not flags.lilyQuest then
				table.insert(steps, function(g: types.Game): string?
					inventory.take(g.player, "bog_lily", 4)
					g.flags.lilyQuest = true
					return "You give four bog lilies to Nettle."
				end)
				table.insert(steps, "Fresh ones! Give me a moment... *bubble bubble* There. Drink it when the marsh bites hard.")
				table.insert(steps, give("nettle_tonic", 3))
				table.insert(steps, xp(30))
				return steps
			end
			if #steps > 0 then
				return steps
			end
			if not flags.lilyQuest then
				table.insert(steps, `Lilies, two-legs: {inventory.count(player, "bog_lily")} of 4. They grow in the bogs: at the edge of the mire, in the gardens of the hospital, on an island of the great mire. The lurkers of the bogs swallow them too.`)
			end
			if not player.artefacts.beaked_mask then
				table.insert(steps, "The masks of the healers are in their House. The menders still guard it, and they mend each other: strike the menders first.")
			elseif not player.artefacts.ferry_skiff then
				table.insert(steps, "With the mask, the docks. The ferry answers to the bell, they say, but not to just anyone. The old ones wrote everything on their stones.")
			elseif not flags["killed:frog_king"] then
				table.insert(steps, "The King's island is in the middle of the great mire. Watch its shadow when it leaps. When its throat swells, step aside.")
			else
				table.insert(steps, "The bezoar! Then cross the fen, two-legs. And come back one day to tell me what's below.")
			end
			return steps
		end,
		shop = {
			{ item = "swamp_salve", count = 1, price = 3, currency = "fig" },
			{ item = "nettle_tonic", count = 1, price = 6, currency = "fig" },
			{ item = "herb_tea", count = 1, price = 3, currency = "fig" },
			{ item = "arrows", count = 5, price = 2, currency = "fig" },
			{ item = "bomb", count = 2, price = 3, currency = "fig" },
			{ item = "oilskin_coat", count = 1, price = 14, currency = "fig" },
		},
	},

	unit9 = {
		name = "Unit-9",
		sprite = "unit9",
		talk = function(game): { types.DialogStep }
			local steps: { types.DialogStep } = {}
			if not game.flags.unit9Met then
				game.flags.unit9Met = true
				table.insert(steps, "UNIT-9, CARE AUTOMATON. WELCOME TO THE HOUSE OF HEALING. PLEASE STATE YOUR WATER.")
				table.insert(steps, "...NO WATER? THEN YOU ARE EBB. THE EBB ARE TREATED IN THE QUARANTINE. UNIT-9 IS SORRY.")
				table.insert(steps, "WARNING: THE CHIEF MENDER IS MALFUNCTIONING. IT KEEPS THE SEAL OF THE PHYSICIAN, AND IT MENDS EVERYTHING. EVEN WHAT SHOULD STAY BROKEN.")
			elseif game.player.artefacts.beaked_mask and not game.notebook.lore.quarantine then
				table.insert(steps, "THE QUARANTINE IS BEHIND THE GREEN AIR. NO ONE HAS ENTERED IT SINCE THE TIDE LEFT. UNIT-9 WOULD LIKE TO KNOW WHAT HAPPENED TO THE EBB.")
			end
			table.insert(steps, function(g: types.Game): string?
				g.player.hp = player_rules.maxHp(g.player)
				g.player.poison = 0
				return "Unit-9 treats your wounds. TREATMENT COMPLETE. NEXT PATIENT."
			end)
			return steps
		end,
	},

	wick = {
		name = "Wick",
		sprite = "wick",
		talk = function(game): { types.DialogStep }
			if not game.flags.wickMet then
				game.flags.wickMet = true
				return {
					"*click* Shh! You'll scare the fish. I'm Wick. I fish here, since the frogs ate my last boat.",
					"The ferry of the old ones still sleeps at the docks, past the green air. My grandmother said it came when the bell was rung right. Nobody remembers how.",
					"My uncle Fen was a smuggler. He hid his loot in the walls of the docks, he said. We never found where.",
				}
			end
			if game.player.artefacts.ferry_skiff and not game.flags.wickWaters then
				game.flags.wickWaters = true
				return {
					"You have the ferry! *click click* Then listen: the old waters above hide things.",
					"The lakes of the hollows, the drowned colonnade of the oasis... and the desert ants say their canals are full again. Sail them all.",
				}
			end
			if not game.player.artefacts.ferry_skiff then
				return {
					"The bell of the ferry is at the end of the docks, past the green air. The healers had masks for that air.",
					"My grandmother hummed a slow tune when she spoke of it. Slow, like the breath of the tide. Its stone says the rest, if you can read it.",
				}
			end
			return { "Fish don't bite when the King croaks. And the King croaks a lot." }
		end,
	},

	-- The frontier: the Frostborn of Rimehold, and what the Watch left behind

	--- Elder Hask, in the lodge of Rimehold: the way down, and the lost pup
	hask = {
		name = "Elder Hask",
		sprite = "hask",
		talk = function(game): { types.DialogStep }
			local flags = game.flags
			local player = game.player
			local steps: { types.DialogStep } = {}
			if not flags.haskMet then
				flags.haskMet = true
				table.insert(steps, "Sit, sit, warm your hands. A traveller from above... We are the Frostborn. Our grandmothers were soldiers of the Watch, the soldiers of the Bronze.")
				table.insert(steps, "The Watch kept this frontier for the city below. When the city sealed itself, the High Tide wanted a Watch that never tires, like its servants of bronze. So the Watch was told to sleep in the ice, and dream its watch forever.")
				table.insert(steps, "Some of us woke up. A soldier who wakes falls from his water: the city called us Ebb, like its sick in the marshes, and forgot us. We stayed in the cold, and we never quite stopped dreaming. We see things you don't. Not yet.")
				table.insert(steps, "The way down is past the Old Watch, beyond the Glare. Its gate is at the top of the icefall, and only crampons climb an icefall. The children keep the champion's pair, in the yard. Beat them at their game.")
				table.insert(steps, "And... one more thing. My granddaughter Tamsin went to the ice caves, south of the village, and she did not come back. If you see her, send her home.")
			end
			if flags.tamsinFound and not flags.tamsinThanked then
				table.insert(steps, function(g: types.Game): string?
					g.flags.tamsinThanked = true
					return nil
				end)
				table.insert(steps, "Tamsin is home, frozen to the bone and very proud of herself. You have my thanks, traveller. Take my old cloak: it kept me warm for sixty winters.")
				table.insert(steps, give("fur_cloak"))
				table.insert(steps, xp(60))
				return steps
			end
			if #steps > 0 then
				return steps
			end
			if not flags.tamsinFound then
				table.insert(steps, "Tamsin plays in the ice caves, south of the village, where the old blocks of ice are. Blocks slide until they hit something, she says. She must be stuck behind them.")
			end
			if not player.artefacts.rime_crampons then
				table.insert(steps, "The crampons first. Sleet, in the yard, will tell you the rules. They cheat, you know. Children.")
				return steps
			elseif not flags["killed:last_watchman"] then
				table.insert(steps, "The Watch is still up there, at the Old Watch. The Last Watchman never slept: they say he has become a husk of ice.")
				table.insert(steps, "Ysolde says that only those who dream can touch him. Ysolde says many things. This time, I believe her.")
				return steps
			end
			table.insert(steps, "You have his ember... Then he is relieved, at last. Go down, traveller. The whiteout won't bite you now.")
			return steps
		end,
	},

	--- Sleet, leader of the kids of the yard: the snowball gallery (gallery.luau)
	sleet = {
		name = "Sleet",
		sprite = "sleet",
		talk = function(game): { types.DialogStep }
			local flags = game.flags
			local steps: { types.DialogStep } = {}
			if not flags.sleetMet then
				flags.sleetMet = true
				table.insert(steps, "Hey, two-legs! This is our yard. Wanna play? Snowballs at the targets: snowmen, pucks on the rink, and the hare if you're quick.")
				table.insert(steps, `{gallery.ROUND_TIME} seconds. Our record is {gallery.GOAL} points. Beat it, and you get the champion's crampons. Lose, and you're a snowman.`)
			elseif flags.galleryWon and not flags.galleryChampion then
				table.insert(steps, `Another go? Sorrel's record is still {gallery.CHAMPION}. She said the pale snowmen count triple. What pale snowmen? Ask her, she's... well. Gone.`)
			elseif flags.galleryChampion then
				table.insert(steps, "Champion! The yard is yours, whenever you want.")
			else
				table.insert(steps, `Ready? {gallery.ROUND_TIME} seconds. Beat {gallery.GOAL}!`)
			end
			table.insert(steps, function(g: types.Game): string?
				gallery.start(g)
				return "Snowballs! Attack to throw. Go!"
			end)
			return steps
		end,
	},

	--- Ysolde the dreamer: what the dream is, and what hides in it
	ysolde = {
		name = "Ysolde",
		sprite = "ysolde",
		talk = function(game): { types.DialogStep }
			local flags = game.flags
			if not flags.ysoldeMet then
				flags.ysoldeMet = true
				return {
					"Oh. You are awake. All the way awake. How loud the world must be for you.",
					"The veils, the walls of mist? Frozen dreams. A tired mind walks through them. A bright one bumps its nose.",
					"The healers of the House brewed the sleep of the Watch from hollowcaps. We still eat them, a little, to see. Gaze into the mirrors of the Watch, or eat a hollowcap, and you will dream. The light of a fire wakes you up again, slowly.",
					"In the dream, the Watch still patrols. Pale shades. They are not kind to the waking, and the waking's blades barely touch them.",
				}
			end
			if not flags.ysoldeVeils then
				flags.ysoldeVeils = true
				return {
					"I dreamed of you. You were walking in the gardens above, where the water is warm. There was a veil there too.",
					"The sea people raised veils wherever they hid something: in the gardens of the oasis, in the dry halls, in the marshes... Look for the mist.",
				}
			end
			if not game.player.artefacts.sleepless_eye then
				return { "In the barracks of the Old Watch, a corporal still stands guard. He only speaks to dreamers. He has lost the names of his men." }
			end
			return { "The sleepless eye... Now you dream with your eyes open, like us. Welcome home." }
		end,
		shop = {
			{ item = "hollowcap", count = 2, price = 2, currency = "fig" },
			{ item = "herb_tea", count = 1, price = 3, currency = "fig" },
			{ item = "dirge_bell", count = 1, price = 30, currency = "fig" },
		},
	},

	--- Grizzle, the trader of Rimehold
	grizzle = {
		name = "Grizzle",
		sprite = "grizzle",
		talk = function(game): { types.DialogStep }
			if not game.flags.grizzleMet then
				game.flags.grizzleMet = true
				return {
					"Figs! You have figs? Nothing grows down here but mushrooms and frostbite. I trade anything for figs.",
					"Broth to warm your belly, cloaks for your back, slings for the wolves. The bombs come down from the ants of the dry halls, Clack's cousins. Snow is free.",
				}
			end
			return { "Snow out there, wolves out there, broth in here. Buying?" }
		end,
		shop = {
			{ item = "warm_broth", count = 1, price = 4, currency = "fig" },
			{ item = "fur_cloak", count = 1, price = 18, currency = "fig" },
			{ item = "icicle_sling", count = 1, price = 16, currency = "fig" },
			{ item = "icicle", count = 10, price = 2, currency = "fig" },
			{ item = "bomb", count = 2, price = 3, currency = "fig" },
			{ item = "arrows", count = 5, price = 2, currency = "fig" },
		},
	},

	--- Brann the lamplighter: light every brazier of the frontier
	brann = {
		name = "Brann",
		sprite = "brann",
		talk = function(game): { types.DialogStep }
			local flags = game.flags
			local lit, total = module.braziers(game)
			if not flags.brannMet then
				flags.brannMet = true
				return {
					"Ho there! Brann, lamplighter of the frontier. Well. I was. My legs don't carry me to the far braziers any more.",
					"The Watch put braziers everywhere: a warm spot in the cold, a light in the dark. Light them for me, would you? Just walk up and light them.",
					`{lit} of {total} lit, by my count. Light them all and I'll give you my hearthstone.`,
				}
			end
			if lit >= total and not flags.brannDone then
				return {
					"I can see them from here, all burning! Like when I was a boy. Here, take my hearthstone: it never goes cold.",
					function(g: types.Game): string?
						g.flags.brannDone = true
						return nil
					end,
					give("hearthstone"),
					xp(50),
				}
			end
			if flags.brannDone then
				return { "All lit. The frontier is warmer for it. Thank you, friend." }
			end
			local dark = module.darkPlaces(game)
			return {
				`{lit} of {total} braziers burning. Keep going!`,
				`From my window I can still see darkness over: {table.concat(dark, ", ")}. There's always one more, somewhere behind the ice.`,
			}
		end,
	},

	--- Tamsin, Elder Hask's granddaughter, stuck in the ice caves (and back home once found)
	tamsin = {
		name = "Tamsin",
		sprite = "tamsin",
		talk = function(game): { types.DialogStep }
			if game.level.id == "rimehold" then
				return { "I'm never going in the caves again. Well. Maybe tomorrow." }
			end
			return {
				"Is someone there? I'm not lost! I'm... exploring. The ice blocks moved, and the way back is gone.",
				"You opened it? You can push the blocks? Wow. Grandpa is going to be so mad. Thank you!",
				function(g: types.Game): string?
					g.flags.tamsinFound = true
					return "Tamsin runs home to Rimehold."
				end,
			}
		end,
	},

	--- Corporal Vey, a phantom of the Watch: only dreamers see him. He lost the names of his men.
	vey = {
		name = "Corporal Vey",
		sprite = "vey",
		dream = true,
		talk = function(game): { types.DialogStep }
			local flags = game.flags
			local found = 0
			for _, id in module.WATCH_NAMES do
				if game.notebook.lore[id] then
					found += 1
				end
			end
			if flags.veyDone then
				return { "Orren, Maelis, Tobin, Astra. I say their names every watch. Thank you, dreamer." }
			end
			local steps: { types.DialogStep } = {}
			if not flags.veyMet then
				flags.veyMet = true
				table.insert(steps, "Halt. Who... You can see me? A dreamer. Good. Corporal Vey, fourth squad of the Watch. Still on duty.")
				table.insert(steps, "We slept in the ice, as ordered. Since then I have forgotten the names of my men. Four of them. A corporal should know his men.")
				table.insert(steps, "We carved our names where we stood guard, in the dream. Find them for me. Read them to me.")
			end
			if found >= #module.WATCH_NAMES then
				table.insert(steps, "Orren. Maelis. Tobin. Astra. ...Yes. Yes, that was them. Thank you.")
				table.insert(steps, "Take my eye. I kept it open for a thousand years: with it, you will always see the dream.")
				table.insert(steps, function(g: types.Game): string?
					g.flags.veyDone = true
					return nil
				end)
				table.insert(steps, give("sleepless_eye"))
				table.insert(steps, xp(80))
			elseif flags.veyMet and #steps == 0 then
				local missing = {}
				for i, id in module.WATCH_NAMES do
					if not game.notebook.lore[id] then
						table.insert(missing, module.WATCH_POSTS[i])
					end
				end
				table.insert(steps, `{found} of {#module.WATCH_NAMES} names. Still missing: the ones who stood guard {table.concat(missing, ", ")}. They carved their names in the dream: dream there, and look.`)
			end
			return steps
		end,
	},
} :: { [string]: Npc }

return module
