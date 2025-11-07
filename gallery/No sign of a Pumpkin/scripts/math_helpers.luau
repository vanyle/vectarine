function permute(a, b, c, d, rot)
	if rot == 0 then
		return { a, b, c, d }
	elseif rot == 1 then
		return { d, a, b, c }
	elseif rot == 2 then
		return { c, d, a, b }
	elseif rot == 3 then
		return { b, c, d, a }
	else
		return { a, b, c, d }
	end
end

-- Export the module functions
return {
	permute = permute,
}
