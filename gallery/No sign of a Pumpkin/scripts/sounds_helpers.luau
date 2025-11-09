local Audio = require("@vectarine/audio")
local Debug = require("@vectarine/debug")
local Event = require("@vectarine/event")
local Resource = require("@vectarine/resources")

local _, eventHelpers = Resource.loadScript("scripts/event_helpers.luau", require("event_helpers.luau"))

local ambiantMusic = Resource.loadAudio("audio/game_music.wav")
local popSound = Resource.loadAudio("audio/short_fire_whoosh.wav")
local fireOnsetSound = Resource.loadAudio("audio/fire_onset.wav")
local candleBlowSound = Resource.loadAudio("audio/candle_blow.wav")
local deathMusic = Resource.loadAudio("audio/death_music.wav")
local creepySound = Resource.loadAudio("audio/creepy_sound.wav")
local battleMusic = Resource.loadAudio("audio/battle_music.wav")

-- Set up background music to play when resource loads (must be immediate, not delayed)
Event.getResourceLoadedEvent():on(function(r)
	if r == ambiantMusic then
		Debug.print("Playing background music from resource loaded event...")
		Audio.play(ambiantMusic, true)
		Audio.setVolume(ambiantMusic, 0.2)
	end
end)

function play_pop_sound()
	if Resource.isResourceReady(popSound) then
		Audio.play(popSound, false)
	else
		Debug.print("Pop sound not ready!")
	end
end

function play_fire_onset_sound()
	if Resource.isResourceReady(fireOnsetSound) then
		Audio.play(fireOnsetSound, false)
	else
		Debug.print("Fire onset sound not ready!")
	end
end

function play_candle_blow_sound()
	if Resource.isResourceReady(candleBlowSound) then
		Audio.play(candleBlowSound, false)
	else
		Debug.print("Candle blow sound not ready!")
	end
end

function play_death_music()
	if Resource.isResourceReady(deathMusic) then
		Audio.setVolume(ambiantMusic, 0.0)
		Audio.play(deathMusic, false)
		Audio.setVolume(deathMusic, 0.7)
		eventHelpers.schedule_action(6.0, function()
			Audio.setVolume(ambiantMusic, 0.5)
		end, "Restore ambiant music volume after death music")
	else
		Debug.print("Death music not ready!")
	end
end

function play_creepy_sound()
	if Resource.isResourceReady(creepySound) then
		Audio.play(creepySound, false)
	else
		Debug.print("Creepy sound not ready!")
	end
end

function play_battle_music()
	if Resource.isResourceReady(battleMusic) then
		Audio.setVolume(ambiantMusic, 0.0)
		Audio.play(battleMusic, true)
		Audio.setVolume(battleMusic, 0.5)
	else
		Debug.print("Battle music not ready!")
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
