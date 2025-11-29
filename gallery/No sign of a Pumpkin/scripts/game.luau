local Canvas = require("@vectarine/canvas")
local Coord = require("@vectarine/coord")
local Debug = require("@vectarine/debug")
local Event = require("@vectarine/event")
local Graphics = require("@vectarine/graphics")
local Io = require("@vectarine/io")
local Loader = require("@vectarine/loader")
local Persist = require("@vectarine/persist")
local Vec = require("@vectarine/vec")
local image = require("@vectarine/image")

local LevelDesignType = require("level_design.luau")
local _, levelDesign = Loader.loadScript("scripts/level_design.luau", require("level_design.luau"))
local helperScript, eventHelpers = Loader.loadScript("scripts/event_helpers.luau", require("event_helpers.luau"))
local _, gameMechanics = Loader.loadScript("scripts/game_mechanics.luau", require("game_mechanics.luau"))
local _, soundsHelpers = Loader.loadScript("scripts/sounds_helpers.luau", require("sounds_helpers.luau"))
local _, animationHelpers = Loader.loadScript("scripts/animation_helpers.luau", require("animation_helpers.luau"))
local _, coordHelpers = Loader.loadScript("scripts/coord_helpers.luau", require("coord_helpers.luau"))
local _, drawHelpers = Loader.loadScript("scripts/draw_helpers.luau", require("draw_helpers.luau"))

local sign_image = Loader.loadImage("textures/Signs/small/Sign.png", false)

local tileset = Loader.loadTileset("levels/gloomy-forest.tsx")

if _G.isReady == nil then
	_G.isReady = false
end

-- Load the animation frames
local fire_spirit_animation = {}
for i = 1, 6 do
	local img = Loader.loadImage("textures/FireSpirit/fire spirit" .. tostring(i) .. ".png", false)
	table.insert(fire_spirit_animation, img)
end

local pumpkin_animation = {}
for i = 1, 6 do
	local img = Loader.loadImage("textures/PumpkinMan/PumpkinMan" .. tostring(i) .. ".png", false)
	table.insert(pumpkin_animation, img)
end

local menu_animation = {}
for i = 1, 14 do
	local img = Loader.loadImage("textures/Signs/menu/Menu" .. tostring(i) .. ".png", false)
	table.insert(menu_animation, img)
end

local watching_animation = {}
for i = 1, 14 do
	local img = Loader.loadImage("textures/Signs/watching/watching" .. tostring(i) .. ".png", false)
	table.insert(watching_animation, img)
end

local water_animation = {}
for i = 1, 14 do
	local img = Loader.loadImage("textures/Signs/water/water" .. tostring(i) .. ".png", false)
	table.insert(water_animation, img)
end

local baba_animation = {}
for i = 1, 14 do
	local img = Loader.loadImage("textures/Signs/baba/baba" .. tostring(i) .. ".png", false)
	table.insert(baba_animation, img)
end

local behind_animation = {}
for i = 1, 14 do
	local img = Loader.loadImage("textures/Signs/behind/behind" .. tostring(i) .. ".png", false)
	table.insert(behind_animation, img)
end

local directions_animation = {}
for i = 1, 14 do
	local img = Loader.loadImage("textures/Signs/directions/directions" .. tostring(i) .. ".png", false)
	table.insert(directions_animation, img)
end

local hiding_animation = {}
for i = 1, 14 do
	local img = Loader.loadImage("textures/Signs/hiding/hiding" .. tostring(i) .. ".png", false)
	table.insert(hiding_animation, img)
end

local moving_animation = {}
for i = 1, 14 do
	local img = Loader.loadImage("textures/Signs/moving/moving" .. tostring(i) .. ".png", false)
	table.insert(moving_animation, img)
end

local nightmare_animation = {}
for i = 1, 14 do
	local img = Loader.loadImage("textures/Signs/nightmare/nightmare" .. tostring(i) .. ".png", false)
	table.insert(nightmare_animation, img)
end

local quiet_animation = {}
for i = 1, 14 do
	local img = Loader.loadImage("textures/Signs/quiet/quiet" .. tostring(i) .. ".png", false)
	table.insert(quiet_animation, img)
end

local running_animation = {}
for i = 1, 14 do
	local img = Loader.loadImage("textures/Signs/running/running" .. tostring(i) .. ".png", false)
	table.insert(running_animation, img)
end

-- Animation dictionary for easy lookup
local animations = {
	watching = watching_animation,
	water = water_animation,
	menu = menu_animation,
	baba = baba_animation,
	behind = behind_animation,
	directions = directions_animation,
	hiding = hiding_animation,
	moving = moving_animation,
	nightmare = nightmare_animation,
	quiet = quiet_animation,
	running = running_animation,
}

local shadowShader = Loader.loadShader("shaders/simple_shadow.glsl")

local V2 = Vec.V2
local timer = 0.0
local canvas = Canvas.createCanvas(1200, 800)
local camera = Persist.onReload({
	position = V2(0, 0),
	zoom = 60,
}, "camera")

if _G.cheating == nil then
	_G.cheating = false
end

-- Consolidated Game State
local GameState = Persist.onReload({
	camera = camera,

	-- Debug flags
	cheating = _G.cheating or false,
	show_hitboxes = false,
	show_entity_positions = false,

	-- Pause/Animation state
	game_paused = false,
	waiting_for_input = false,
	playing_unpause_animation = false,
	unpause_animation_frame = 1,
	unpause_animation_timer = 0.0,
	death_blackscreen = false,
	last_sign_touched = false,

	-- Win state
	winning = false,
	win_timer = 0.0,
	win_initial_zoom = 0.0,
	win_initial_fuel = 0.0,
	win_music_played = false,

	-- Asset display
	displayed_asset = nil :: image.ImageResource?,
	displayed_asset_name = "",
}, "GameState")

-- Constants
local BLACKSCREEN_DURATION = 1.0 -- seconds of complete darkness
local POP_SOUND_DELAY = 0.3 -- seconds before blackscreen ends
local MINIMAL_FUEL_AMOUNT = 1.0 -- very small amount for dim light
local MINIMAL_FUEL_DURATION = 1.0 -- seconds of almost minimal fuel
local WINDUP_DURATION = 3.0 -- seconds to ramp up to full fuel

local ANIMATION_START_DELAY = 0.2 -- seconds after fire onset sound
local UNPAUSE_ANIMATION_FRAMES = 14

local REFUEL_AMOUNT = 40.0

local FIRE_CRACKLING_DIVISOR = 250
local LIGHT_ZONE_TRANSFORM_NUMERATOR = 25
local LIGHT_ZONE_TRANSFORM_DENOMINATOR = 99
local LIGHT_ZONE_TRANSFORM_OFFSET = 2450

local ASSET_WIDTH_PX = 294
local ASSET_HEIGHT_PX = 137

local DISPLAY_PADDING_PX = 50

local LARGE_SIGN_WAIT_DELAY = 1.0 -- seconds

local PUMPKIN_CREATE_OFFSET = 5

-- Win animation constants
local WIN_DURATION = 5.0 -- seconds for the win effect
local WIN_TARGET_ZOOM = 20
local WIN_TARGET_FUEL = 500.0 -- Very high fuel for bright light
local PLAYER_POSITION_START = V2(10, 10)

local windup_timer = Persist.onReload({ timer = 0.0 }, "windup_timer")
local INITIAL_FUEL_AMOUNT = 100.0 -- fuel amount after ramp-up
local player: LevelDesignType.Player = Persist.onReload({
	position = PLAYER_POSITION_START,
	size = 1.0,
	ratio = 1,

	hitbox_size = V2(0.8, 0.8),

	speed = 2.0, -- pixels per second

	animation_idx = 0,
	animation_images = fire_spirit_animation,
	frame_delay = 0.15,
	anim_time = 0,

	fuel = 0, -- Start at 0, will ramp up during windup
	max_fuel = 150.0,
	fuel_drain_rate = 1.0, -- fuel units per second
	fuel_death_threshold = -1,
}, "player")

-- Windup system management
function manage_windup(time_delta: number)
	local total_startup_time = BLACKSCREEN_DURATION + MINIMAL_FUEL_DURATION + WINDUP_DURATION

	if windup_timer.timer < total_startup_time then
		windup_timer.timer += time_delta

		if windup_timer.timer <= BLACKSCREEN_DURATION then
			-- Phase 1: Complete darkness
			player.fuel = 0
		elseif windup_timer.timer <= BLACKSCREEN_DURATION + MINIMAL_FUEL_DURATION then
			-- Phase 2: Almost minimal fuel (very dim light)
			player.fuel = MINIMAL_FUEL_AMOUNT
		else
			-- Phase 3: Gradual windup to full fuel
			local windup_start_time = BLACKSCREEN_DURATION + MINIMAL_FUEL_DURATION
			local windup_elapsed = windup_timer.timer - windup_start_time
			local windup_progress = math.clamp(windup_elapsed / WINDUP_DURATION, 0, 1)

			-- Smooth easing function for nice ramp-up
			local ease_progress = windup_progress * windup_progress * (3 - 2 * windup_progress) -- smoothstep
			-- Ramp from minimal fuel to full fuel
			player.fuel = MINIMAL_FUEL_AMOUNT + ease_progress * (INITIAL_FUEL_AMOUNT - MINIMAL_FUEL_AMOUNT)
		end
	end
end

function reset_windup()
	windup_timer.timer = 0.0
	player.fuel = 0
end

function manage_win_animation(time_delta: number)
	if not GameState.winning then
		return
	end

	GameState.win_timer += time_delta
	local progress = math.clamp(GameState.win_timer / WIN_DURATION, 0, 1)

	-- Smooth easing function for nice gradual effect
	local ease_progress = progress * progress * (3 - 2 * progress) -- smoothstep

	-- Gradually zoom out from initial zoom to target zoom
	-- GameState.camera.zoom = GameState.win_initial_zoom + ease_progress * (WIN_TARGET_ZOOM - GameState.win_initial_zoom)

	-- Gradually increase fuel for brighter light
	player.fuel = GameState.win_initial_fuel + ease_progress * (WIN_TARGET_FUEL - GameState.win_initial_fuel)

	-- End the win animation after duration and trigger music
	if GameState.win_timer >= WIN_DURATION then
		if not GameState.win_music_played then
			soundsHelpers.play_battle_music()

			-- Display baba sign permanently (no pause needed)
			local baba_animation_array = animations.baba or animations.watching
			GameState.displayed_asset = baba_animation_array[1] -- Use first frame
			GameState.displayed_asset_name = "baba"
			-- No pause - let the game continue with sign overlay

			GameState.win_music_played = true
		end
	end
end

function Restart()
	Debug.print("Restarting game...")
	player.position = PLAYER_POSITION_START
	-- Center camera around player from the start
	local half_player_size = animationHelpers.getDeformedPlayerSize(player, timer, false):scale(0.5)
	GameState.camera.position = player.position + half_player_size
	reset_windup() -- Reset windup system

	-- Clear all delayed actions and unpause the game
	if helperScript:isReady() then
		eventHelpers.clear_all_delayed_actions()
	end
	GameState.game_paused = false
	GameState.waiting_for_input = false
	GameState.playing_unpause_animation = false
	GameState.unpause_animation_frame = 1
	GameState.unpause_animation_timer = 0.0
	GameState.last_sign_touched = false
	GameState.winning = false
	GameState.win_timer = 0.0
	GameState.win_music_played = false
	-- Reset camera zoom to default
	-- GameState.camera.zoom = 60
	drawHelpers.clear_displayed_asset(GameState)

	-- Schedule pop sound to play before blackscreen ends
	if helperScript:isReady() then
		eventHelpers.schedule_action(
			BLACKSCREEN_DURATION - POP_SOUND_DELAY,
			soundsHelpers.play_pop_sound,
			"Play pop sound before blackscreen ends"
		)
	end

	levelDesign.remove_entities()
	levelDesign.spawn_entities()
end

-- Render sign overlay when game is paused
function render_sign_overlay()
	-- Show sign overlay if game is paused OR if we're winning and showing baba sign
	local should_show_sign = (GameState.game_paused and GameState.displayed_asset)
		or (GameState.winning and GameState.displayed_asset_name == "baba" and GameState.displayed_asset)

	if not (should_show_sign and (GameState.displayed_asset == nil or GameState.displayed_asset:isReady())) then
		return
	end

	local current_asset = GameState.displayed_asset
	if GameState.playing_unpause_animation then
		local base_asset_name = GameState.displayed_asset_name or "watching"
		local animation_array = animations[base_asset_name] or animations.watching
		local frame_index = math.min(GameState.unpause_animation_frame, #animation_array)
		current_asset = animation_array[frame_index]
	end

	if current_asset ~= nil and current_asset:isReady() then
		local currentWindowSize = Io.getWindowSize()
		local window_width, window_height = currentWindowSize.x, currentWindowSize.y
		local padding = DISPLAY_PADDING_PX
		local max_width, max_height = window_width - (padding * 2), window_height - (padding * 2)
		local scale_by_width, scale_by_height = max_width / ASSET_WIDTH_PX, max_height / ASSET_HEIGHT_PX
		local scale_factor = math.min(math.min(scale_by_width, scale_by_height), 2.0)
		local target_width_px, target_height_px = ASSET_WIDTH_PX * scale_factor, ASSET_HEIGHT_PX * scale_factor
		local display_size = Coord.pxVec(V2(target_width_px, target_height_px), currentWindowSize)
		local display_pos = Coord.px(V2(window_width / 2 - target_width_px / 2, padding), currentWindowSize)
		current_asset:draw(display_pos, display_size)
	end
end

function Update(time_delta: number)
	if not _G.isReady then
		if helperScript:isReady() then
			_G.isReady = true
			Debug.print("Game is ready!")
			Load()
		end
		return
	end

	-- Always manage delayed actions (even when paused, for unpause functionality)
	eventHelpers.manage_delayed_actions(time_delta)

	-- Check for input when waiting to unpause
	if GameState.waiting_for_input and eventHelpers.is_any_movement_input_pressed() then
		GameState.waiting_for_input = false

		-- Play fire onset sound for all signs (including menu)
		soundsHelpers.play_fire_onset_sound()

		-- Delay the animation start slightly (like the pop sound timing)
		eventHelpers.schedule_action(ANIMATION_START_DELAY, function()
			GameState.playing_unpause_animation = true
			GameState.unpause_animation_frame = 1
			GameState.unpause_animation_timer = 0.0
		end, "Start burning animation after fire onset sound")
	end

	-- Handle unpause animation
	if GameState.playing_unpause_animation then
		GameState.unpause_animation_timer += time_delta

		local FRAME_DELAY = 0.15 -- seconds per frame
		if GameState.unpause_animation_timer >= FRAME_DELAY / 1.5 then
			GameState.unpause_animation_timer = 0.0
			GameState.unpause_animation_frame += 1

			if GameState.unpause_animation_frame > UNPAUSE_ANIMATION_FRAMES then
				-- Animation complete - check if this is menu or normal sign
				if GameState.displayed_asset_name == "menu" then
					-- Menu animation complete: restart the game
					Restart()
				else
					-- Normal sign animation complete: refuel player and unpause the game
					local refuel_amount = REFUEL_AMOUNT -- Set amount to refuel
					player.fuel = math.min(player.fuel + refuel_amount, player.max_fuel)
					GameState.playing_unpause_animation = false
					GameState.game_paused = false

					-- Check if this was the last sign and activate cheat mode
					if GameState.last_sign_touched then
						win()
					else
						-- Only clear displayed asset if we're not winning
						drawHelpers.clear_displayed_asset(GameState)
					end

					Debug.print("Game unpaused!")
				end
			end
		end
	end

	-- Handle pause logic - skip game updates when paused
	if not GameState.game_paused then
		timer += time_delta
		manage_windup(time_delta)
		manage_win_animation(time_delta)

		if GameState.cheating then
			player.fuel = 200
		end
	end

	if shadowShader:isReady() then
		canvas:setShader(shadowShader)

		local pp = coordHelpers.convertToScreenPos(GameState, player.position + V2(1, 1):scale(0.5 * player.size), true)
		local ppGl = pp:gl()

		-- Disable pulsating effect during windup phase
		local total_startup_time = BLACKSCREEN_DURATION + MINIMAL_FUEL_DURATION + WINDUP_DURATION
		local is_windup_active = windup_timer.timer < total_startup_time
		local fireCracklingEffect = is_windup_active and 0
			or math.sin(5 * (timer - total_startup_time)) / FIRE_CRACKLING_DIVISOR

		-- Affine transformation: 1.0 -> 25.0, 100.0 -> 50.0 using y = (25/99)x + 2450/99
		local transformedFuel = (LIGHT_ZONE_TRANSFORM_NUMERATOR / LIGHT_ZONE_TRANSFORM_DENOMINATOR) * player.fuel
			+ LIGHT_ZONE_TRANSFORM_OFFSET / LIGHT_ZONE_TRANSFORM_DENOMINATOR
		local lightZoneSize = GameState.cheating and 2.0
			or math.clamp(transformedFuel, 0, player.max_fuel) / 100 + fireCracklingEffect
		canvas:setUniform("lightZone", lightZoneSize)
		canvas:setUniform("lightPosX", ppGl.x / 2)
		canvas:setUniform("lightPosY", ppGl.y / 2)

		local windowSize = Io.getWindowSize()
		canvas:setUniform("windowWidth", windowSize.x)
		canvas:setUniform("windowHeight", windowSize.y)
	end

	Graphics.clear({ r = 0, g = 0, b = 0, a = 1 })

	-- Check if we're in blackscreen phase
	if windup_timer.timer <= BLACKSCREEN_DURATION then
		-- Show menu during blackscreen if game is paused
		render_sign_overlay()
		return
	end

	-- Check if we're in death blackscreen
	if GameState.death_blackscreen then
		return -- Just show blackscreen, no menu
	end

	canvas:paint(function()
		Graphics.clear({ r = 0.6, g = 0.5, b = 0.2, a = 1 })

		-- Only update game entities when not paused
		if not GameState.game_paused then
			manage_pumpkins(time_delta)
			manage_player(time_delta)
			manage_sign(time_delta)

			-- read arrows + WASD (also checks common names like 'ArrowLeft')
			-- Center camera around player position plus half player size
			local half_player_size = animationHelpers.getDeformedPlayerSize(player, timer, false):scale(0.5)
			local target_camera_pos = player.position + half_player_size
			GameState.camera.position = GameState.camera.position
				+ (target_camera_pos - GameState.camera.position):scale(time_delta)
		end

		local player_screen_pos = coordHelpers.convertToScreenPos(GameState, player.position, true)
		local player_screen_size =
			coordHelpers.convertToScreenSize(GameState, animationHelpers.getDeformedPlayerSize(player, timer, true))

		local tilesAcrossScreenX = math.ceil(2 / coordHelpers.convertToScreenSize(GameState, V2(1, 0)):gl().x) + 1
		local tilesAcrossScreenY = math.ceil(math.abs(2 / coordHelpers.convertToScreenSize(GameState, V2(0, 1)):gl().y))
			+ 1
		local tileSizeScreen = coordHelpers.convertToScreenSize(GameState, V2(1, 1)):gl()
		local wp = coordHelpers.convertToWorldPos(GameState, V2(-1, 1) - tileSizeScreen, false):floor()

		Debug.timed("draw_environment", function()
			for y = 1, tilesAcrossScreenY do
				for x = 1, tilesAcrossScreenX do
					local worldPos = nil
					local tile_id = nil
					local tile_rot = nil
					local tile_name = nil
					Debug.timed("drawEnvironmentTile:getTileAt", function()
						worldPos = wp + V2(x, y)
						local t = levelDesign.getTileAt(wp.x + x, wp.y + y)
						tile_id = t[1]
						tile_rot = t[2]
						tile_name = levelDesign.sprite_ids[tile_id]
					end)
					Debug.timed("drawEnvironmentTile", function()
						drawHelpers.drawEnvironmentTile(GameState, tile_name, worldPos, tile_rot)
					end)
				end
			end
		end)

		-- Draw pumpkins
		for _, pumpkin in ipairs(levelDesign.list_pumpkin) do
			local pumpkin_screen_pos = coordHelpers.convertToScreenPos(GameState, pumpkin.position, false)
			local pumpkin_screen_size =
				coordHelpers.convertToScreenSize(GameState, V2(1, pumpkin.ratio):scale(pumpkin.size))
			-- Use delta-time based animation (pause-aware)
			local animation_delta = GameState.game_paused and 0 or time_delta
			local frameId = animationHelpers.animate(pumpkin, animation_delta, pumpkin.frame_delay)

			local img = pumpkin_animation[frameId]
			-- Debug.fprint("Pumpkin : Frame id = ", frameId, " img = ", img)
			if img ~= nil then
				img:draw(pumpkin_screen_pos, pumpkin_screen_size)
			end

			-- Draw pumpkin hitbox
			local pumpkin_hitbox_pos = pumpkin.position + V2(0.2, 0.2)
			local pumpkin_hitbox_size = V2(1.6, 1.6)
			drawHelpers.drawHitbox(GameState, pumpkin_hitbox_pos, pumpkin_hitbox_size, { r = 1, g = 0, b = 0, a = 0.3 })
			-- Draw pumpkin position marker
			drawHelpers.drawPosition(GameState, pumpkin.position, { r = 1, g = 0.5, b = 0.5, a = 1.0 })
		end

		-- Draw signs
		for _, sign in ipairs(levelDesign.list_sign) do
			local sign_screen_pos = coordHelpers.convertToScreenPos(GameState, sign.position, false)
			local sign_screen_size = coordHelpers.convertToScreenSize(GameState, V2(1, 1))
			sign_image:draw(sign_screen_pos, sign_screen_size)

			-- Draw sign hitbox
			local sign_hitbox_pos = sign.position + V2(0, 0)
			local sign_hitbox_size = V2(1, 1)
			drawHelpers.drawHitbox(GameState, sign_hitbox_pos, sign_hitbox_size, { r = 0, g = 0, b = 1, a = 0.3 })
			-- Draw sign position marker
			drawHelpers.drawPosition(GameState, sign.position, { r = 0.5, g = 0.5, b = 1, a = 1.0 })
		end

		-- Player animation (delta-time driven, pause-aware)
		Debug.fprint("Player pos: ", player.position)
		local animation_delta = GameState.game_paused and 0 or time_delta
		local frameId = animationHelpers.animate(player, animation_delta, player.frame_delay)
		local img = fire_spirit_animation[frameId]
		if img ~= nil then
			img:draw(player_screen_pos, player_screen_size)
		end

		-- Draw player hitbox
		local player_hitbox_pos = player.position + V2(0.1, 0.1)
		local player_hitbox_size = V2(0.8, 0.8)
		drawHelpers.drawHitbox(GameState, player_hitbox_pos, player_hitbox_size, { r = 0, g = 1, b = 0, a = 0.3 })
		-- Draw player position marker
		drawHelpers.drawPosition(GameState, player.position, { r = 0.5, g = 1, b = 0.5, a = 1.0 })

		if GameState.cheating then
			drawHelpers.drawPathMarkers(GameState, levelDesign.list_path_marker)
		end
	end)

	canvas:draw(V2(-1, -1), V2(2, 2))

	-- Display loaded asset during pause (outside canvas for overlay effect)
	render_sign_overlay()
end

local MAX_CHASE_DISTANCE = 5.0
local STAMINA_CHASE_SPEED_MULTIPLIER = 4.0
local WANDERING_SPEED_MULTIPLIER = 3.0 -- Quick wandering for unpredictable position

function is_marker_targetted(marker, pumpkin_list: { any }): boolean
	for _, pumpkin in ipairs(pumpkin_list) do
		if pumpkin.target == marker then
			return true
		end
	end
	return false
end

function manage_pumpkin(
	time_delta: number,
	pumpkin: {
		stamina: number,
		max_stamina: number,
		position: Vec.Vec2,
		speed: number,
		state: string,
		target: any,
		patrol_route: { any },
	}
)
	-- Pumpkin logic handled in manage_pumpkins
	-- Get close to player
	local p: Vec.Vec2 = player.position
	local pp: Vec.Vec2 = pumpkin.position
	local direction = (pp - p):normalized()
	local dist_to_player = (pp - p):length()

	if pumpkin.state == "chasing" then
		if dist_to_player > MAX_CHASE_DISTANCE then
			pumpkin.state = "wandering"
		else
			-- Continue chasing
			direction = (pp - p):normalized()
			pumpkin.stamina = math.max(pumpkin.stamina - time_delta, 0)
			local speed_multiplier = pumpkin.stamina > 0 and STAMINA_CHASE_SPEED_MULTIPLIER or 1.0
			Debug.fprint(speed_multiplier, " ; ", pumpkin.stamina)
			pumpkin.position = pumpkin.position - direction:scale(time_delta * pumpkin.speed * speed_multiplier)
		end
	else
		-- Not chasing
		if dist_to_player <= MAX_CHASE_DISTANCE and not GameState.win_music_played then
			pumpkin.state = "chasing"
			soundsHelpers.play_creepy_sound()
			pumpkin.target = nil
			pumpkin.stamina = math.max(pumpkin.max_stamina, pumpkin.stamina + time_delta)
			-- Find a target to wander towards
		elseif pumpkin.target == nil then
			-- Use pumpkin's individual patrol route instead of shared markers
			if pumpkin.patrol_route and #pumpkin.patrol_route > 0 then
				-- Find the closest marker from this pumpkin's assigned route
				local marker_distances = {}
				for _, marker in ipairs(pumpkin.patrol_route) do
					local dist = (marker.position - pumpkin.position):length()
					table.insert(marker_distances, { marker = marker, distance = dist })
				end
				table.sort(marker_distances, function(a, b)
					return a.distance < b.distance
				end)
				-- Assign closest marker from this pumpkin's route (no sharing conflicts)
				pumpkin.target = marker_distances[1].marker
			else
				-- Fallback to shared markers if no individual route assigned
				local marker_distances = {}
				for _, marker in ipairs(levelDesign.list_path_marker) do
					local dist = (marker.position - pumpkin.position):length()
					table.insert(marker_distances, { marker = marker, distance = dist })
				end
				table.sort(marker_distances, function(a, b)
					return a.distance < b.distance
				end)
				for _, entry in ipairs(marker_distances) do
					if not is_marker_targetted(entry.marker, levelDesign.list_pumpkin) then
						pumpkin.target = entry.marker
						break
					end
				end
			end
		end

		-- Move towards target if it exists
		if pumpkin.target ~= nil then
			-- If the target is targetted by another pumpkin, don't move for a bit.
			for _, other_pumpkin in ipairs(levelDesign.list_pumpkin) do
				if other_pumpkin.target == pumpkin.target then
					if other_pumpkin ~= pumpkin then
						return
					else
						break -- The first pumpkin is itself, continue
					end
				end
			end

			local target_pos = pumpkin.target.position
			direction = (pp - target_pos):normalized()
			pumpkin.position = pumpkin.position
				- direction:scale(time_delta * pumpkin.speed * WANDERING_SPEED_MULTIPLIER)
			-- Go to the next marker if close enough
			local target_dist = (pumpkin.position - target_pos):length()
			if target_dist < 0.5 then
				pumpkin.target = pumpkin.target.next_marker
			end
		end
	end

	-- If close enough, die to the pumpkin
	if not GameState.cheating and not GameState.win_music_played then
		local player_hitbox = gameMechanics.createHitbox(player :: any)
		local pumpkin_hitbox = gameMechanics.createHitbox(pumpkin :: any)

		if gameMechanics.checkHitboxCollision(player_hitbox, pumpkin_hitbox) then
			die()
		end
	end
end

function manage_pumpkins(time_delta: number)
	for i, pumpkin in ipairs(levelDesign.list_pumpkin) do
		manage_pumpkin(time_delta, pumpkin :: any)
	end
end

function manage_sign(time_delta: number)
	for i, sign in ipairs(levelDesign.list_sign) do
		local player_hitbox = gameMechanics.createHitbox(player :: any)
		local sign_hitbox = gameMechanics.createHitbox(sign :: any)

		local collision = gameMechanics.checkHitboxCollision(player_hitbox, sign_hitbox)

		if collision and not sign.destroyed then
			-- We allow you to not collect 2 signs before winning
			local is_last_sign = #levelDesign.list_sign <= 3

			-- Handle sign touch effects
			handle_sign_touch(sign.text or "")

			-- If this is the last sign, mark it for cheat mode activation after animation
			if is_last_sign then
				GameState.last_sign_touched = true
				Debug.print("Last sign touched! Cheat mode will activate after burning animation...")
			end

			-- Remove sign immediately
			table.remove(levelDesign.list_sign, i)
		end
	end
end

-- Handle different sign touch effects based on sign properties
function handle_sign_touch(asset_name: string)
	-- Load asset based on sign text - use first frame of preloaded animation
	local animation_array = animations[asset_name] or animations.watching

	-- Use first frame of the animation (frame 1)
	GameState.displayed_asset = animation_array[1]
	GameState.displayed_asset_name = asset_name or ""

	-- Special behavior for menu: restart game instead of normal sign behavior
	if asset_name == "menu" then
		GameState.game_paused = true
		GameState.waiting_for_input = false
		eventHelpers.schedule_action(LARGE_SIGN_WAIT_DELAY, function()
			GameState.waiting_for_input = true
		end, "Start waiting for input to begin game")
	else
		-- Default behavior: pause the game and schedule input monitoring after 1 seconds
		GameState.game_paused = true
		GameState.waiting_for_input = false
		eventHelpers.schedule_action(LARGE_SIGN_WAIT_DELAY, function()
			GameState.waiting_for_input = true
		end, "Start waiting for input")
	end
end

function isHitboxInWater(pos: Vec.Vec2, size: Vec.Vec2): boolean
	local p = pos:floor()
	local ps = (pos + size):floor()
	local tile1 = levelDesign.getTileTypeAtPosition(ps.x, ps.y)
	local tile2 = levelDesign.getTileTypeAtPosition(p.x, p.y)
	local tile3 = levelDesign.getTileTypeAtPosition(ps.x, p.y)
	local tile4 = levelDesign.getTileTypeAtPosition(p.x, ps.y)
	return tile1 == 3 or tile2 == 3 or tile3 == 3 or tile4 == 3
end

-- Manage player movement
function manage_player(time_delta: number)
	-- player movement
	local dx, dy = 0, 0
	if Io.isKeyDown("q") or Io.isKeyDown("left") then
		dx = dx - 1
	end
	if Io.isKeyDown("d") or Io.isKeyDown("right") then
		dx = dx + 1
	end
	if Io.isKeyDown("z") or Io.isKeyDown("up") then
		dy = dy - 1
	end
	if Io.isKeyDown("s") or Io.isKeyDown("down") then
		dy = dy + 1
	end

	-- normalize diagonal movement
	local futurePosition = player.position + V2(dx, dy):scale(player.speed * time_delta)

	local inWater = isHitboxInWater(futurePosition, V2(1, 1))
	if inWater then
		local p = (player.position + V2(0.5, 0.5)):floor()
		-- cannot go into water
		local currentTile = levelDesign.getTileTypeAtPosition(p.x, p.y)
		if currentTile == 3 then
			-- Allow movement to get out
			player.position = futurePosition
		end
	else
		player.position = futurePosition
	end

	-- manage fuel
	player.fuel = player.fuel - time_delta * player.fuel_drain_rate
	if player.fuel < player.fuel_death_threshold then
		die()
	end
end

function die()
	Debug.print("Player has died! Starting death sequence...")

	GameState.death_blackscreen = true
	soundsHelpers.play_candle_blow_sound()
	eventHelpers.schedule_action(1.5, function()
		soundsHelpers.play_death_music()
	end)
	eventHelpers.schedule_action(8.0, function()
		GameState.death_blackscreen = false
		Restart()
	end)
end

function win()
	Debug.print("Player has won the game! Starting victory sequence...")

	-- Initialize win state
	GameState.winning = true
	GameState.win_timer = 0.0
	GameState.win_initial_zoom = GameState.camera.zoom
	GameState.win_initial_fuel = player.fuel
end

function Load()
	handle_sign_touch("menu")

	-- First launch
	Debug.print("Game started")
end

Event.getConsoleCommandEvent():on(function(command)
	if command == "spawn" then
		local random_offset = V2(1, -1):scale(PUMPKIN_CREATE_OFFSET)
		levelDesign.create_pumpkin(V2(player.position.x, player.position.y) + random_offset, levelDesign.list_pumpkin)
	end
	if command == "clear" then
		levelDesign.list_pumpkin = {}
		_G.pumpkin_list = {}
	end
	if command == "hitboxes" then
		GameState.show_hitboxes = not GameState.show_hitboxes
		Debug.print("Hitboxes " .. (GameState.show_hitboxes and "enabled" or "disabled"))
	end
	if command == "positions" then
		GameState.show_entity_positions = not GameState.show_entity_positions
		Debug.print("Entity position markers " .. (GameState.show_entity_positions and "enabled" or "disabled"))
	end
	if command == "win" then
		win()
	end
	if command == "cheat" then
		GameState.cheating = not GameState.cheating
		_G.cheating = GameState.cheating
		Debug.print("Cheating mode " .. (GameState.cheating and "enabled" or "disabled"))
	end
	if command == "pause" then
		GameState.game_paused = not GameState.game_paused
		Debug.print("Game " .. (GameState.game_paused and "paused" or "unpaused"))
	end
	if command == "zoom" then
		GameState.camera.zoom = GameState.camera.zoom + 10
		Debug.print("Camera zoom set to " .. tostring(GameState.camera.zoom))
	end
	if command == "unzoom" then
		GameState.camera.zoom = GameState.camera.zoom - 10
		Debug.print("Camera zoom set to " .. tostring(GameState.camera.zoom))
	end
	if command == "reload" then
		Load()
		Debug.print("Game reloaded")
	end
end)
