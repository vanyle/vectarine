local Res = require("@vectarine/resources")

local module = {}

--- Play an audio resource.
--- If loop is true, the audio will loop indefinitely.
function module.play(audio: Res.AudioResource, loop: boolean?, fadeIn: number?): () end

--- Stop an audio resource.
--- If fadeOut is provided, the audio will fade out over that many milliseconds.
--- Does nothing if the audio is not playing.
function module.pause(audio: Res.AudioResource, fadeOut: number?): () end

function module.setVolume(audioResource: Res.AudioResource, volume: number): () end

function module.getVolume(audioResource: Res.AudioResource): number
	error("Not implemented")
end

return module
