local module = {}

--- Get a value that persist when the script is hot-reloaded. The value is stored as a global variable named `globalName`
--- and can be inspected in the watcher.
function module.onReload<T>(defaultValue: T, globalName: string): T
	error("Implemented in native code")
end

--- Same as `onReload`, but instead of taking a default value, it takes a provider function.
--- This is useful if the default value is expensive to create so it is not created on hot-reload.
function module.onReloadWithProvider<T>(provider: () -> T, globalName: string): T
	error("Implemented in native code")
end

--- Save `value` so that it is persisted across restarts.
--- You can obtain saved values using `load`.
--- Saving can take some time, so calling `save` followed by `load` might not give the expected result.
--- `name` can only contain letters, numbers, underscore and dot.
function module.save(name: string, value: any)
	error("Implemented in native code")
end

--- Load a saved value. If the value does not exist, return `nil`.
function module.load(name: string): any?
	error("Implemented in native code")
end

return module
