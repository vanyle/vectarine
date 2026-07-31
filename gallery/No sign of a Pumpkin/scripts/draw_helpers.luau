const coord = require("@vectarine/coord")
const debug = require("@vectarine/debug")
const graphics = require("@vectarine/graphics")
const io = require("@vectarine/io")
const loader = require("@vectarine/loader")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

local _, levelDesign = loader.loadScript("scripts/level_design.luau", require("level_design.luau"))
local _, coordHelpers = loader.loadScript("scripts/coord_helpers.luau", require("coord_helpers.luau"))
local _, mathHelpers = loader.loadScript("scripts/math_helpers.luau", require("math_helpers.luau"))

local environment = loader.loadImage("textures/environment/gloomy-forest.png", false)
type PathMarker = {
	position: vec.Vec2,
	next_marker: PathMarker?,
}

local V2 = vec.V2

-- cross for position debugging
local CROSS_SIZE_PX = 30
local CROSS_THICKNESS_PX = 3

-- cross for position debugging
local function drawPosition(GameState, worldPos: vec.Vec2, color: vec4.Vec4?)
	if not GameState.show_entity_positions then
		return
	end

	local pos_color = color or vec4.WHITE -- Default white
	local screen_pos = coordHelpers.convertToScreenPos(GameState, worldPos, false)

	local windowSize = io.getWindowSize()

	-- Draw a 6x6 pixel cross
	-- Horizontal line (6px wide, 1px tall)
	local h_start = screen_pos + coord.pxVec(V2(-CROSS_SIZE_PX / 2, 0), windowSize) -- Center the 6px line
	local h_size = coord.pxVec(V2(CROSS_SIZE_PX, CROSS_THICKNESS_PX), windowSize)
	graphics.drawRect(h_start, h_size, pos_color)

	-- Vertical line (1px wide, 6px tall)
	local v_start = screen_pos + coord.pxVec(V2(0, -CROSS_SIZE_PX / 2), windowSize) -- Center the 6px line
	local v_size = coord.pxVec(V2(CROSS_THICKNESS_PX, CROSS_SIZE_PX), windowSize)
	graphics.drawRect(v_start, v_size, pos_color)
end

local function drawHitbox(GameState, worldPos: vec.Vec2, worldSize: vec.Vec2, color: vec4.Vec4?)
	if not GameState.show_hitboxes then
		return
	end

	local hitbox_color = color or vec4.createColor(1.0, 0.0, 0.0, 0.5) -- Default red with transparency
	local screen_pos = coordHelpers.convertToScreenPos(GameState, worldPos, false)
	local screen_size = coordHelpers.convertToScreenSize(GameState, worldSize)

	-- Draw a rectangle outline for the hitbox
	graphics.drawRect(screen_pos, screen_size, hitbox_color)
end

local function drawPathMarkers(GameState, list_path_markers: { PathMarker })
	if not GameState.cheating then
		return
	end

	for _, marker in ipairs(list_path_markers) do
		local screen_pos = coordHelpers.convertToScreenPos(GameState, marker.position, false)
		graphics.drawCircle(screen_pos, 0.01, vec4.YELLOW) -- Yellow color for path markers
	end
end

local toScreenPos = coordHelpers.convertToScreenPos

local function drawEnvironmentTile(GameState, tileName: string, position: vec.Vec2, rotation: number)
	local tileInfo: vec.Vec2? = levelDesign.sprite_table[tileName]
	if tileInfo == nil then
		debug.fprint(tileName, " is not in table")
		return
	end
	local sprite_pos: vec.Vec2 = tileInfo * levelDesign.sprite_size

	local dest_p1 = nil
	local dest_p2 = nil
	local dest_p3 = nil
	local dest_p4 = nil

	-- This is the part that takes the most time during rendering, so we optimize it a bit.
	debug.timed("drawEnvironmentTile:toScreenPos", function()
		dest_p1 = toScreenPos(GameState, position, false):gl()
		dest_p3 = toScreenPos(GameState, position + V2(1, 1), false):gl()
		--dest_p2 = toScreenPos(GameState, position + V2(1, 0), false)
		dest_p2 = V2(dest_p3.x, dest_p1.y)
		dest_p4 = V2(dest_p1.x, dest_p3.y)
		--dest_p4 = toScreenPos(GameState, position + V2(0, 1), false)
	end)
	debug.timed("drawEnvironmentTile:environment:drawPart", function()
		if rotation == 0 then
			environment:drawPart(dest_p1, dest_p2, dest_p3, dest_p4, sprite_pos, levelDesign.sprite_size)
		elseif rotation == 1 then
			environment:drawPart(dest_p4, dest_p1, dest_p2, dest_p3, sprite_pos, levelDesign.sprite_size)
		elseif rotation == 2 then
			environment:drawPart(dest_p3, dest_p4, dest_p1, dest_p2, sprite_pos, levelDesign.sprite_size)
		elseif rotation == 3 then
			environment:drawPart(dest_p2, dest_p3, dest_p4, dest_p1, sprite_pos, levelDesign.sprite_size)
		else
			environment:drawPart(dest_p1, dest_p2, dest_p3, dest_p4, sprite_pos, levelDesign.sprite_size)
		end
	end)
end

-- Clear the currently displayed asset (Large Sign)
local function clear_displayed_asset(GameState: any)
	GameState.displayed_asset = nil
	GameState.displayed_asset_name = ""
	debug.print("Cleared displayed asset")
end

-- Export the module functions
return {
	drawHitbox = drawHitbox,
	drawPosition = drawPosition,
	drawEnvironmentTile = drawEnvironmentTile,
	drawPathMarkers = drawPathMarkers,

	clear_displayed_asset = clear_displayed_asset,
}
