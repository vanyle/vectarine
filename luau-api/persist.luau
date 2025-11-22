local module = {}

function module.onReload<T>(defaultValue: T, globalName: string): T
	error("Not implemented")
end

--- Save `value` so that it is persisted across restarts.
--- You can obtain saved values using `load`.
--- Saving can take some time, so calling `save` followed by `load` might not give the expected result.
--- `name` can only contain letters, numbers, underscore and dot.
function module.save(name: string, value: any)
	error("Not implemented")
end

--- Load a saved value. If the value does not exist, return `nil`.
function module.load(name: string): any?
	error("Not implemented")
end

return module
