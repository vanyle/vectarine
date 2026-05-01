--- Functional helpers that shorten the code
local module = {}

function module.map<A, B>(input: { A }, func: (A) -> B): { B }
	local result = {}
	for i, v in ipairs(input) do
		result[i] = func(v)
	end
	return result
end

function module.filter<A>(input: { A }, func: (A) -> boolean): { A }
	local result = {}
	for i, v in ipairs(input) do
		if func(v) then
			table.insert(result, v)
		end
	end
	return result
end

function module.contains<A>(table: { A }, value: A): boolean
	for i, v in ipairs(table) do
		if v == value then
			return true
		end
	end
	return false
end

return module
