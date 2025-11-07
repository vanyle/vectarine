local Vec = require("@vectarine/vec")

local V2 = Vec.V2

export type Hitbox = {
	position: Vec.Vec2,
	size: Vec.Vec2,
}

-- Hitbox helper functions
function createHitbox(entity: {position: Vec.Vec2, hitbox_size: Vec.Vec2, hitbox_offset: Vec.Vec2}): Hitbox
	local actualOffset = entity.hitbox_offset or V2(0, 0)
	return {
		position = entity.position + actualOffset,
		size = entity.hitbox_size,
	}
end

-- Hitbox-based collision detection
function checkHitboxCollision(hitbox1: Hitbox, hitbox2: Hitbox): boolean
	return checkCollision(hitbox1.position, hitbox1.size, hitbox2.position, hitbox2.size)
end

-- AABB (Axis-Aligned Bounding Box) collision detection
function checkCollision(pos1: Vec.Vec2, size1: Vec.Vec2, pos2: Vec.Vec2, size2: Vec.Vec2): boolean
	-- Calculate boundaries for both rectangles
	local left1 = pos1.x
	local right1 = pos1.x + size1.x
	local top1 = pos1.y
	local bottom1 = pos1.y + size1.y

	local left2 = pos2.x
	local right2 = pos2.x + size2.x
	local top2 = pos2.y
	local bottom2 = pos2.y + size2.y

	-- Check if rectangles overlap
	return not (right1 <= left2 or left1 >= right2 or bottom1 <= top2 or top1 >= bottom2)
end

-- Export the module functions
return {
	createHitbox = createHitbox,
	checkHitboxCollision = checkHitboxCollision,
}
