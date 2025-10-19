local module = {}

function module.initFromGlobal<T>(default: T, global_name: string): T
	if _G[global_name] ~= nil then
		return _G[global_name]
	else
		_G[global_name] = default
		return default
	end
end

return module
