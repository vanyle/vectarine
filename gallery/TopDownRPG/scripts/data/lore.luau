--[[
Texts found in the world: ancient tablets of the sea people, signs and notes left by explorers.

Levels refer to them by id (`data = { lore = "tide" }` on a tablet or a sign). Read texts are listed in the notebook.

The sea people wrote in Greek letters, and the explorer can't read their language: ancient texts show as gibberish
in Greek capitals (`scramble`), until the explorer finds the artefact that translates them (`TRANSLATOR`, on a
deeper floor). The real text is kept here, for when they can read it.
]]
const loader = require("@vectarine/loader")
const types = require("../types")

export type Lore = {
	title: string,
	--- Who wrote it, shown as the name in the dialog box
	author: string,
	lines: { string },
	--- Ancient writing of the sea people: unreadable without the translator. Reading it for the first time gives experience.
	ancient: boolean?,
}

const module = loader.init() or {}

--- The artefact that lets the explorer understand the language of the sea people (found on a deeper floor)
module.TRANSLATOR = "tide_lexicon"

function module.canRead(game: types.Game, text: Lore): boolean
	return not text.ancient or game.player.artefacts[module.TRANSLATOR] == true
end

--- The Greek capitals, with those that don't look like Latin letters twice, so that the writing looks foreign
const LETTERS = {
	"Α", "Β", "Γ", "Δ", "Ε", "Ζ", "Η", "Θ", "Ι", "Κ", "Λ", "Μ", "Ν", "Ξ", "Ο", "Π", "Ρ", "Σ", "Τ", "Υ", "Φ", "Χ", "Ψ", "Ω",
	"Γ", "Δ", "Θ", "Λ", "Ξ", "Π", "Σ", "Φ", "Ψ", "Ω",
}

--- The line as the explorer sees it: each letter becomes a Greek capital, words and punctuation keep their place.
--- The same line always gives the same gibberish.
function module.scramble(line: string): string
	local result = {}
	local hash = 7
	for i = 1, #line do
		local char = string.sub(line, i, i)
		hash = (hash * 31 + string.byte(char) + i * 17) % 1000003
		if string.match(char, "%a") then
			table.insert(result, LETTERS[hash % #LETTERS + 1])
		else
			table.insert(result, char)
		end
	end
	return table.concat(result)
end

module.texts = {
	-- Notes of explorers
	first_note = {
		title = "A note in a bottle",
		author = "Note",
		lines = {
			"Day 31. The pools are clean, the figs are sweet, the rats are rude.",
			"Wren says the ants are friendly. Wren says a lot of things.",
			"If you read this, I went east, towards the light of the mushrooms. Don't follow the worms down. - M.",
		},
	},
	camp_sign = {
		title = "Wren's sign",
		author = "Sign",
		lines = { "WREN'S CAMP. Fire, water, figs. No wasps allowed." },
	},
	hollow_sign = {
		title = "A warning",
		author = "Sign",
		lines = { "DARK HOLLOW. No light, no luck. Turn back unless you carry a fire. - W." },
	},
	den_sign = {
		title = "Scratches on the wall",
		author = "Scratches",
		lines = { "Deep claw marks cross the rock. Something big lives beyond this point, and it does not like guests." },
	},
	-- Tablets of the sea people
	tide = {
		title = "The people of the tide",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"WE WERE THE PEOPLE OF THE TIDE. WE SAILED THE GREEN SEA ABOVE.",
			"WHEN THE SEA LEFT US, WE FOLLOWED THE WATER DOWN, WHERE IT STILL SINGS.",
		},
	},
	tunnelers = {
		title = "The great tunnelers",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"THE GREAT TUNNELERS ARE OUR FRIENDS. WE FEED THEM FIGS, THEY DIG OUR HALLS.",
			"NEVER WAKE A TUNNELER IN ITS DEN. EVEN FRIENDS HAVE CLAWS.",
		},
	},
	machines = {
		title = "The tireless servants",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"OUR SERVANTS OF BRONZE DO NOT SLEEP. THEY GUARD THE HALLS WHILE THE CHILDREN OF THE TIDE GO DEEPER.",
			"THE CORES OF LIGHTNING WAKE THE GATES THAT FOLD THE ROADS.",
		},
	},
	lower_stairs = {
		title = "The lower stairs",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"BEYOND THE DEN OF THE TUNNELER, THE STAIRS GO DOWN TO THE DRY HALLS.",
			"THE COUNCIL HAS SEALED THE WAY WITH VINES. ONLY THOSE WHO CLIMB MAY PASS.",
		},
	},

	-- The desert: notes and signs
	steps_note = {
		title = "A dry notebook",
		author = "Note",
		lines = {
			"Day 9 below the stairs. The water ran out on day 6. The cistern here is as dry as my throat.",
			"The cactus fruits kept us going: their water is sweet, if you can stand the spines.",
			"The ants know where water is. Follow their marks to the east. I can't walk any more. - T.",
		},
	},
	anthill_sign = {
		title = "A sign of the colony",
		author = "Sign",
		lines = { "THE ANTHILL. Water for friends. Figs very welcome. *click*" },
	},
	dunes_sign = {
		title = "A warning",
		author = "Sign",
		lines = { "STORMS. When the sand rises, keep to the rocks and wait. The storm always passes. - Mandi, guard of the colony" },
	},
	gorge_sign = {
		title = "Scratched in the rock",
		author = "Scratches",
		lines = { "The wind of the gorge throws back whoever walks against it. Only the eye of the Warden sees through the howl." },
	},
	-- The desert: tablets of the sea people
	diggers = {
		title = "The diggers",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"THE ANTS DIG. THE WARDEN WATCHES. THE CITY DRINKS.",
			"SO IT WAS, SO IT IS, SO IT WILL BE, AS LONG AS THE SUN WARDEN STANDS.",
		},
	},
	pump_gate = {
		title = "The gate of the Great Pump",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"THE GATE OF THE GREAT PUMP OPENS FOR THE ARCHER WHO WAKES THE SUN'S EYE.",
			"WATER FOR THE CITY. THIRST FOR THOSE WHO DIG.",
		},
	},
	pump_hall = {
		title = "The three valves",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"THREE VALVES FEED THE GREAT PUMP. WHEN THE SAND CAME, WE WALLED THE NORTH VALVE BEHIND STONE.",
			"THE MASONS LEFT THE WALL CRACKED, SO THAT IT COULD BE BROKEN AGAIN ONE DAY.",
		},
	},
	council = {
		title = "The council chamber",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"THIS IS THE CHAMBER OF THE COUNCIL, A PLACE OF PEACE.",
			"NO BLADE MAY CROSS ITS DOOR. COME WITH EMPTY HANDS, OR DO NOT COME.",
		},
	},
	warden = {
		title = "The Sun Warden",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"THE SUN WARDEN NEVER SLEEPS. ITS EYE SEES THROUGH SAND AND STORM.",
			"WHOEVER HOLDS THE EYE WALKS IN THE STORM, UNTOUCHED.",
		},
	},
	marshes = {
		title = "The way to the marshes",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"BELOW THE HOWLING GORGE, THE STAIRS GO DOWN TO THE MARSHES, WHERE THE CITY SENT ITS SICK.",
			"DO NOT DRINK THE GREEN WATER.",
		},
	},

	-- The wetlands: notes and signs
	nettle_sign = {
		title = "A sign by the hut",
		author = "Sign",
		lines = { "NETTLE'S HUT. Remedies, brews, advice. Wipe your feet." },
	},
	mud_sign = {
		title = "A warning",
		author = "Sign",
		lines = { "DON'T STAND STILL IN THE MUD. IT EATS THE ONES WHO WAIT. - Wick" },
	},
	smuggler_note = {
		title = "A soggy note",
		author = "Note",
		lines = { "Hid the goods behind the old wall again, where the nets dry. Nobody looks twice at a cracked wall. - Fen" },
	},
	-- The wetlands: tablets of the sea people
	four_waters = {
		title = "The four waters",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"FOUR WATERS MAKE THE CITY: THE HIGH TIDE THAT RULES, THE BRONZE THAT BUILDS, THE SALT THAT SAILS, AND THE CLAY THAT DIGS.",
			"EACH WATER KNOWS ITS PLACE, AND THE CITY STANDS.",
		},
	},
	the_ebb = {
		title = "The Ebb",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"WHO FALLS SICK FALLS FROM THEIR WATER. THE SICK ARE THE EBB, AND THE EBB GOES DOWN TO THE MARSHES.",
			"DO NOT SPEAK THE NAMES OF THE EBB. THE TIDE HAS FORGOTTEN THEM.",
		},
	},
	house_of_healing = {
		title = "The House of Healing",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"THE HOUSE OF HEALING SERVES THE HIGH TIDE. THE HEALERS WEAR THE BEAK, AND FEAR NO AIR.",
			"THE HEAD PHYSICIAN KEEPS THE MASKS IN HIS OFFICE. HIS SEAL OPENS IT.",
		},
	},
	surgery = {
		title = "The surgery",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"THE HOUSE OF HEALING OPENS ITS DOORS TO THE SICK ALONE.",
			"THE HEALTHY HAVE NOTHING TO DO IN THE SURGERY.",
		},
	},
	physician = {
		title = "The physician's journal",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"THE CHIEF MENDER KEEPS MY SEAL WHEN I REST. IT HAS BEGUN TO MEND THINGS THAT SHOULD NOT BE MENDED.",
			"TOMORROW I WILL HAVE IT SHUT DOWN. IT LOOKED AT ME STRANGELY TODAY.",
		},
	},
	quarantine = {
		title = "Scratched on the quarantine wall",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"HERE THE EBB WAIT. THE HEALERS DO NOT ENTER. THEY PASS THE BREAD UNDER THE DOOR.",
			"WE ARE NOT A WATER ANY MORE. WE ARE A PUDDLE. BUT A PUDDLE REMEMBERS THE RAIN.",
		},
	},
	shrine = {
		title = "The basin of the Ebb",
		author = "Ancient tablet",
		ancient = true,
		lines = { "LAY A WHITE LILY BEFORE THE BASIN OF THE EBB, AND THE TIDE WILL REMEMBER YOU." },
	},
	bell = {
		title = "The bell of the ferry",
		author = "Ancient tablet",
		ancient = true,
		lines = { "STRIKE THE BELL THREE TIMES, SLOW AS THE BREATH OF THE TIDE, AND THE FERRY WILL COME FOR YOU." },
	},
	frog_king = {
		title = "The king of the mire",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"THE KING OF THE MIRE GREW FAT ON WHAT WE POURED. A STONE GROWS IN ITS BELLY THAT DRINKS ALL POISON.",
			"WHEN ITS SHADOW GROWS, STEP ASIDE.",
		},
	},
	the_fen = {
		title = "The fen",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"WE POURED OUR POISONS IN THE FEN, FAR FROM THE CITY. THE WATER TURNED GREEN, AND THE FROGS GREW.",
			"NONE OF THE HIGH TIDE EVER SAW IT.",
		},
	},
	cold_stores = {
		title = "The cold below",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"BELOW THE FEN, THE COLD. THE CITY KEPT ITS FROZEN STORES THERE, AND ITS FROZEN SECRETS.",
			"THE HOUSE OF HEALING SENDS ITS DRAUGHT OF SLEEP DOWN THE STAIRS, CRATE AFTER CRATE. FOR WHOM, THE HEALERS DO NOT ASK.",
		},
	},

	-- The frontier: notes and signs
	brazier_note = {
		title = "A frozen notebook",
		author = "Note",
		lines = {
			"Day 2 in the cold. The fire in the brazier saved my life. They still burn, the old things, if you light them.",
			"Light them, whoever you are. And keep out of the snow when you can: the snow drinks your warmth. - M.",
		},
	},
	rimehold_sign = {
		title = "A sign of the village",
		author = "Sign",
		lines = { "RIMEHOLD. Warm hearths, hot broth, loud children. Wipe the snow off your boots. - the Frostborn" },
	},
	yard_sign = {
		title = "The board of the yard",
		author = "Sign",
		lines = { "THE YARD. Snowball champions only. Record: 13 points. (Sorrel: 40. Sorrel cheated.)" },
	},
	glare_sign = {
		title = "A warning",
		author = "Sign",
		lines = { "THE GLARE. Grey ice is thin ice: never stop on it. Seals slide faster than you run: step aside. - Brann" },
	},
	savings_note = {
		title = "A note in a crack",
		author = "Note",
		lines = { "Froze my savings in a block, like grandma did. A thief can push a block around all day: only a fire gets the chest out. - Fenn" },
	},
	spring_note = {
		title = "A note by the steam",
		author = "Note",
		lines = { "Followed the steam to a crack in the ice. Behind it: the warmest bath of my life. - M." },
	},
	whiteout_note = {
		title = "A note, frozen stiff",
		author = "Note",
		lines = {
			"I thought I could run it. Two broths, my thickest coat. Halfway, the cold was in my bones. Three quarters, it was in my thoughts.",
			"The gate at the end doesn't even open. A flame is carved in it. Go back, whoever you are, and find the fire.",
		},
	},
	caves_note = {
		title = "Scratched in the ice",
		author = "Scratches",
		lines = { "IF THE BLOCKS GET STUCK, WALK OUT AND COME BACK IN. THE CAVE PUTS THEM BACK. - Tamsin (not lost)" },
	},
	kids_note = {
		title = "Scribbled on the wall",
		author = "Scribbles",
		lines = { "SECRET FORT. KIDS ONLY. Grown-ups who follow our footprints are SPIES. (Spies get the old prizes, we don't want them.)" },
	},
	-- The frontier: tablets of the Watch
	frontier = {
		title = "The frontier",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"HERE STANDS THE WATCH OF THE BRONZE, BETWEEN THE CITY AND THE DRY WORLD THE SEA LEFT BEHIND.",
			"NONE GO DOWN WITHOUT THE WORD OF THE HIGH TIDE.",
		},
	},
	the_sleep = {
		title = "The order of the sleep",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"OUR SERVANTS OF BRONZE DO NOT SLEEP, BUT THEY ARE FEW. BY ORDER OF THE HIGH TIDE, THE WATCH SHALL SLEEP IN THE ICE, WITH THE FROZEN STORES, AND KEEP ITS WATCH IN THE DREAM, WHERE NO SOLDIER TIRES.",
			"THE HEALERS OF THE HOUSE BREW THE DRAUGHT. THE CLAY DIGS THE BEDS OF ICE. THE CITY SEALS ITSELF. THE WATCH REMAINS.",
		},
	},
	the_veils = {
		title = "The veils",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"THE VEILS ARE WOVEN OF OUR DREAMS. THE WAKING SHALL NOT PASS. THE DREAMING WATCH SHALL PASS.",
			"GAZE INTO THE MIRROR, SOLDIER, AND GO TO YOUR POST.",
		},
	},
	the_plates = {
		title = "The inner gate",
		author = "Ancient tablet",
		ancient = true,
		lines = { "THE INNER GATE OPENS WHEN THE ICE STANDS ON EVERY RUNE. PUSH, AND THE ICE SLIDES UNTIL IT MEETS STONE." },
	},
	captain = {
		title = "The captain of the Watch",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"THE CAPTAIN DOES NOT SLEEP, LIKE THE SUN WARDEN OF THE DRY HALLS. HE KEEPS THE FIRE OF THE WATCH, THE LAST WARMTH OF THE FRONTIER.",
			"TO THE WAKING HE IS ICE. ONLY THE DREAMING MAY RELIEVE HIM OF HIS WATCH.",
		},
	},
	cold_lock = {
		title = "The Cold Lock",
		author = "Ancient tablet",
		ancient = true,
		lines = {
			"THE COLD LOCK. BEYOND IT, THE CITY OF THE TIDE.",
			"WHO CARRIES THE FIRE OF THE WATCH WALKS THROUGH THE WHITE DEATH. THE TIDE WAITS BELOW.",
		},
	},
	-- The frontier: dream runes, only seen by dreamers (the names of Corporal Vey's squad, and other dreams)
	watch_name_orren = {
		title = "A name in the dream",
		author = "Dream rune",
		lines = { "ORREN, OF THE FOURTH SQUAD. STOOD GUARD AT THE STAIR. COLD FEET, WARM HEART." },
	},
	watch_name_maelis = {
		title = "A name in the dream",
		author = "Dream rune",
		lines = { "MAELIS, OF THE FOURTH SQUAD. KEPT THE LAKE. SHE COULD SKATE BACKWARDS." },
	},
	watch_name_tobin = {
		title = "A name in the dream",
		author = "Dream rune",
		lines = { "TOBIN, OF THE FOURTH SQUAD. KEPT THE CAVES, AND THE BLOCKS. NEVER LOST A PUZZLE." },
	},
	watch_name_astra = {
		title = "A name in the dream",
		author = "Dream rune",
		lines = { "ASTRA, OF THE FOURTH SQUAD. KEPT THE WALLS OF THE WATCH. SHE SANG ON GUARD." },
	},
	dream_tower = {
		title = "A dream of the lake",
		author = "Dream rune",
		lines = { "UNDER THE DARK WATER OF THE LAKE, A TOWER OF THE WATCH. WHO CARRIES THE FIRE MAY SAIL THERE." },
	},
	dream_armory = {
		title = "A dream of the armory",
		author = "Dream rune",
		lines = { "THE ARMORY OF THE WATCH IS ACROSS THE CHASM. THE BRIDGE IS ONLY THERE FOR THOSE WHO DREAM OF IT." },
	},
	dream_cache = {
		title = "A dream left behind",
		author = "Dream rune",
		lines = { "M. WAS HERE. SHE DREAMED THE WAY THROUGH THE MIST, AND LEFT SOMETHING FOR THE NEXT DREAMER." },
	},
} :: { [string]: Lore }

return module
