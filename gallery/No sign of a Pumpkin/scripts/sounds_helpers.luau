local debug = require("@vectarine/debug")
local event = require("@vectarine/event")
local loader = require("@vectarine/loader")

local _, eventHelpers = loader.loadScript("scripts/event_helpers.luau", require("event_helpers.luau"))

local ambiantMusic = loader.loadAudio("audio/game_music.wav")
local popSound = loader.loadAudio("audio/short_fire_whoosh.wav")
local fireOnsetSound = loader.loadAudio("audio/fire_onset.wav")
local candleBlowSound = loader.loadAudio("audio/candle_blow.wav")
local deathMusic = loader.loadAudio("audio/death_music.wav")
local creepySound = loader.loadAudio("audio/creepy_sound.wav")
local battleMusic = loader.loadAudio("audio/battle_music.wav")

local ambiantIsNotPlaying = true

-- Set up background music to play when resource loads (must be immediate, not delayed)
event.getResourceLoadedEvent():on(function(r)
	if ambiantMusic:isReady() and ambiantIsNotPlaying then
		ambiantMusic:play(true)
		ambiantMusic:setVolume(0.2)
		ambiantIsNotPlaying = false
	end
end)

function play_pop_sound()
	if popSound:isReady() then
		popSound:play(false)
	else
		debug.print("Pop sound not ready!")
	end
end

function play_fire_onset_sound()
	if fireOnsetSound:isReady() then
		fireOnsetSound:play(false)
	else
		debug.print("Fire onset sound not ready!")
	end
end

function play_candle_blow_sound()
	if candleBlowSound:isReady() then
		candleBlowSound:play(false)
	else
		debug.print("Candle blow sound not ready!")
	end
end

function play_death_music()
	if deathMusic:isReady() then
		ambiantMusic:setVolume(0.0)
		deathMusic:play(false)
		deathMusic:setVolume(0.7)
		eventHelpers.schedule_action(6.0, function()
			ambiantMusic:setVolume(0.5)
		end, "Restore ambiant music volume after death music")
	else
		debug.print("Death music not ready!")
	end
end

function play_creepy_sound()
	if creepySound:isReady() then
		creepySound:play(false)
	else
		debug.print("Creepy sound not ready!")
	end
end

function play_battle_music()
	if battleMusic:isReady() then
		ambiantMusic:setVolume(0.0)
		battleMusic:play(true)
		battleMusic:setVolume(0.5)
	else
		debug.print("Battle music not ready!")
	end
end

-- Export the module functions
return {
	play_pop_sound = play_pop_sound,
	play_fire_onset_sound = play_fire_onset_sound,
	play_candle_blow_sound = play_candle_blow_sound,
	play_death_music = play_death_music,
	play_creepy_sound = play_creepy_sound,
	play_battle_music = play_battle_music,
}
