local module = {}

-- Providing a version in your plugin.luau and your native code can be useful so that people can easily know what version they are using
-- and if there is a mismatch between their .luau file and the native code (which should never happen!!)
module.VERSION = 2
module.NAME = "Plugin Template"

--- Returns the square of a number
--- @param n number
--- @return number
function module.square(n: number): number
	return n * n
end

return module
