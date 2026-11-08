local Resource = require("@vectarine/resource")

--[[
Plays sounds and music.

Load a sound file with `loader.loadAudio` to get an `AudioResource`, then call `AudioResource:play`. Sounds load in the background, so
a sound that is not loaded yet stays silent and raises no error. Volume is set per sound with `AudioResource:setVolume`.

# Example

```luau
const io = require("@vectarine/io")
const loader = require("@vectarine/loader")

const jumpSound = loader.loadAudio("audio/jump.wav")

function Update(deltaTime: number)
	if io.isKeyJustPressed("Space") then
		jumpSound:play()
	end
end
```

# Common mistakes

- Play a sound when something happens, such as a key press (`io.isKeyJustPressed`), not on every frame. Calling
  `AudioResource:play` every frame does nothing while the sound plays, but it restarts the sound as soon as it ends.
- A sound that is not loaded yet is silent and raises no error. A wrong path looks the same.
]]
local module = {}

local AudioResourceImpl = { type = "audio" }
AudioResourceImpl.__index = AudioResourceImpl

--- An audio resource is a playable piece of sound or music.
---
--- You get one from `loader.loadAudio`. Play it with `AudioResource:play`, stop it with `AudioResource:pause` and read
--- its state with `AudioResource:isPlaying`.
export type AudioResource = typeof(setmetatable({}, AudioResourceImpl)) & Resource.Resource

--- Plays the sound from its beginning.
---
--- A sound plays once at a time: calling `AudioResource:play` while the sound is already playing does nothing. If the
--- sound is paused, it continues from where it was, as `AudioResource:resume` does. When a non-looped sound reaches its
--- end, it stops, and the next call starts it from the beginning. Nothing happens if the sound is not loaded yet.
---
--- @param loop If `true`, the sound restarts when it ends, until it is paused. Defaults to `false`.
--- @param fadeIn The time, in milliseconds, over which the volume rises from silence. Defaults to `100` when the sound
--- starts, and to `0` when it continues after a pause.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- const music = loader.loadAudio("audio/music.ogg")
---
--- function Update(deltaTime: number)
--- 	music:play(true, 2000) -- background music, fading in over 2 seconds
--- end
--- ```
---
--- # See also
---
--- - `AudioResource:pause`: stops the sound.
--- - `AudioResource:setVolume`: changes the volume.
function AudioResourceImpl.play(self: AudioResource, loop: boolean?, fadeIn: number?): ()
	error("Implemented in native code")
end

--- Returns `true` if the sound is currently playing.
---
--- It is `false` before the sound is played and when it is paused. A sound that is
--- fading out after `AudioResource:pause` still counts as playing until the fade ends.
---
--- @return True if the sound is playing.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- const music = loader.loadAudio("audio/music.ogg")
---
--- function Update(deltaTime: number)
--- 	if not music:isPlaying() then
--- 		music:play()
--- 	end
--- end
--- ```
function AudioResourceImpl.isPlaying(self: AudioResource): boolean
	error("Implemented in native code")
end

--- Returns the duration of the sound in seconds.
---
--- Returns `0` if the sound is not loaded yet or failed to load.
---
--- @return The duration, in seconds.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- const music = loader.loadAudio("audio/music.ogg")
---
--- function Update(deltaTime: number)
--- 	print(music:getDuration()) -- for example 93.5
--- end
--- ```
function AudioResourceImpl.getDuration(self: AudioResource): number
	error("Implemented in native code")
end

--- Returns how far the sound has played, in seconds.
---
--- It is a number between `0` and the duration given by `AudioResource:getDuration`. Use it to show a progress bar or to
--- synchronize the game with music.
---
--- @return The time played so far, in seconds.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- const music = loader.loadAudio("audio/music.ogg")
---
--- function Update(deltaTime: number)
--- 	local ratio = music:getProgress() / music:getDuration()
--- 	print(ratio) -- 0 at the start, close to 1 at the end
--- end
--- ```
function AudioResourceImpl.getProgress(self: AudioResource): number
	error("Implemented in native code")
end

--- Pauses the sound.
---
--- The sound keeps its position: `AudioResource:resume` or `AudioResource:play` continue from there. Does nothing if the
--- sound is not playing.
---
--- @param fadeOut The time, in milliseconds, over which the volume goes down to silence before the sound pauses. The
--- sound counts as playing until then. Defaults to `0`, which pauses at once.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- const music = loader.loadAudio("audio/music.ogg")
---
--- function PauseGame()
--- 	music:pause(500)
--- end
--- ```
function AudioResourceImpl.pause(self: AudioResource, fadeOut: number?): ()
	error("Implemented in native code")
end

--- Continues a paused sound from where it was.
---
--- Use it after `AudioResource:pause`. To start a sound from the beginning, use `AudioResource:play`.
---
--- @param fadeIn The time, in milliseconds, over which the volume rises from silence. Defaults to `0`.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- const music = loader.loadAudio("audio/music.ogg")
---
--- function ResumeGame()
--- 	music:resume(500)
--- end
--- ```
function AudioResourceImpl.resume(self: AudioResource, fadeIn: number?): ()
	error("Implemented in native code")
end

--- Sets the volume of the sound.
---
--- Use a value between `0` (silent) and `1` (full volume). Nothing
--- happens if the sound is not loaded yet.
---
--- @param volume The new volume, between `0` and `1`.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- const music = loader.loadAudio("audio/music.ogg")
---
--- music:setVolume(0.3) -- quiet background music
--- ```
function AudioResourceImpl.setVolume(self: AudioResource, volume: number): ()
	error("Implemented in native code")
end

--- Returns the volume of the sound.
---
--- It is `1` until you change it with `AudioResource:setVolume`. Returns `0` if the sound is not loaded yet.
---
--- @return The volume, between `0` and `1`.
---
--- # Example
---
--- ```luau
--- const loader = require("@vectarine/loader")
---
--- const music = loader.loadAudio("audio/music.ogg")
---
--- function Update(deltaTime: number)
--- 	if music:isReady() then
--- 		music:setVolume(math.max(0, music:getVolume() - deltaTime * 0.1)) -- slowly fade out
--- 	end
--- end
--- ```
function AudioResourceImpl.getVolume(self: AudioResource): number
	error("Implemented in native code")
end

return module
