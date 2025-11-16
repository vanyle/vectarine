local Resource = require("@vectarine/resource")

local module = {}

local AudioResourceImpl = { type = "audio" }
AudioResourceImpl.__index = AudioResourceImpl
export type AudioResource = typeof(setmetatable({}, AudioResourceImpl)) & Resource.Resource

--- Play an audio resource.
--- If loop is true, the audio will loop indefinitely.
function AudioResourceImpl.play(self: AudioResource, loop: boolean?, fadeIn: number?): ()
	error("Not implemented")
end

--- Stop an audio resource.
--- If fadeOut is provided, the audio will fade out over that many milliseconds.
--- Does nothing if the audio is not playing.
function AudioResourceImpl.pause(self: AudioResource, fadeOut: number?): ()
	error("Not implemented")
end

function AudioResourceImpl.setVolume(self: AudioResource, volume: number): ()
	error("Not implemented")
end

function AudioResourceImpl.getVolume(self: AudioResource): number
	error("Not implemented")
end

return module
