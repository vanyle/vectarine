local Coord = require("@vectarine/coord")
local Debug = require("@vectarine/debug")
local Graphics = require("@vectarine/graphics")
local Io = require("@vectarine/io")
local Loader = require("@vectarine/loader")
local Vec = require("@vectarine/vec")

local _, levelDesign = Loader.loadScript("scripts/level_design.luau", require("level_design.luau"))
local _, coordHelpers = Loader.loadScript("scripts/coord_helpers.luau", require("coord_helpers.luau"))
local _, mathHelpers = Loader.loadScript("scripts/math_helpers.luau", require("math_helpers.luau"))

local environment = Loader.loadImage("textures/environment/gloomy-forest.png", false)
type PathMarker = {
	position: Vec.Vec2,
	next_marker: PathMarker?,
}

local V2 = Vec.V2

-- cross for position debugging
local CROSS_SIZE_PX = 30
local CROSS_THICKNESS_PX = 3

-- cross for position debugging
local function drawPosition(GameState, worldPos: Vec.Vec2, color: { r: number, g: number, b: number, a: number }?)
	if not GameState.show_entity_positions then
		return
	end

	local pos_color = color or { r = 1, g = 1, b = 1, a = 1.0 } -- Default white
	local screen_pos = coordHelpers.convertToScreenPos(GameState, worldPos, false)

	local windowSize = Io.getWindowSize()

	-- Draw a 6x6 pixel cross
	-- Horizontal line (6px wide, 1px tall)
	local h_start = screen_pos + Coord.pxVec(V2(-CROSS_SIZE_PX / 2, 0), windowSize) -- Center the 6px line
	local h_size = Coord.pxVec(V2(CROSS_SIZE_PX, CROSS_THICKNESS_PX), windowSize)
	Graphics.drawRect(h_start, h_size, pos_color)

	-- Vertical line (1px wide, 6px tall)
	local v_start = screen_pos + Coord.pxVec(V2(0, -CROSS_SIZE_PX / 2), windowSize) -- Center the 6px line
	local v_size = Coord.pxVec(V2(CROSS_THICKNESS_PX, CROSS_SIZE_PX), windowSize)
	Graphics.drawRect(v_start, v_size, pos_color)
end

local function drawHitbox(
	GameState,
	worldPos: Vec.Vec2,
	worldSize: Vec.Vec2,
	color: { r: number, g: number, b: number, a: number }?
)
	if not GameState.show_hitboxes then
		return
	end

	local hitbox_color = color or { r = 1, g = 0, b = 0, a = 0.5 } -- Default red with transparency
	local screen_pos = coordHelpers.convertToScreenPos(GameState, worldPos, false)
	local screen_size = coordHelpers.convertToScreenSize(GameState, worldSize)

	-- Draw a rectangle outline for the hitbox
	Graphics.drawRect(screen_pos, screen_size, hitbox_color)
end

local function drawPathMarkers(GameState, list_path_markers: { PathMarker })
	if not GameState.cheating then
		return
	end

	for _, marker in ipairs(list_path_markers) do
		local screen_pos = coordHelpers.convertToScreenPos(GameState, marker.position, false)
		Graphics.drawCircle(screen_pos, 0.01, { r = 1, g = 1, b = 0, a = 1.0 }) -- Yellow color for path markers
	end
end

local function drawEnvironmentTile(GameState, tileName: string, position: Vec.Vec2, rotation: number)
	local tileInfo: Vec.Vec2? = levelDesign.sprite_table[tileName]
	if tileInfo == nil then
		Debug.fprint(tileName, " is not in table")
		return
	end
	local sprite_pos: Vec.Vec2 = tileInfo * levelDesign.sprite_size

	local dest_p1 = coordHelpers.convertToScreenPos(GameState, position, false)
	local dest_p2 = coordHelpers.convertToScreenPos(GameState, position + V2(1, 0), false)
	local dest_p3 = coordHelpers.convertToScreenPos(GameState, position + V2(1, 1), false)
	local dest_p4 = coordHelpers.convertToScreenPos(GameState, position + V2(0, 1), false)

	local permutated = mathHelpers.permute(dest_p1, dest_p2, dest_p3, dest_p4, rotation)

	environment:drawPart(
		permutated[1],
		permutated[2],
		permutated[3],
		permutated[4],
		sprite_pos,
		levelDesign.sprite_size
	)
end

-- Clear the currently displayed asset (Large Sign)
local function clear_displayed_asset(GameState: any)
	GameState.displayed_asset = nil
	GameState.displayed_asset_name = ""
	Debug.print("Cleared displayed asset")
end

-- Export the module functions
return {
	drawHitbox = drawHitbox,
	drawPosition = drawPosition,
	drawEnvironmentTile = drawEnvironmentTile,
	drawPathMarkers = drawPathMarkers,

	clear_displayed_asset = clear_displayed_asset,
}
