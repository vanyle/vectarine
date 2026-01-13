local Debug = require("@vectarine/debug")
local print, fprint = Debug.print, Debug.fprint

local Canvas = require("@vectarine/canvas")
local Event = require("@vectarine/event")
local Graphics = require("@vectarine/graphics")
local Io = require("@vectarine/io")
local Loader = require("@vectarine/loader")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")
local V2 = Vec.V2

--- Constants
_G.BG_COLOR = Vec4.createColor(132 / 255, 160 / 255, 189 / 255, 1)
local _SNAKE_COLOR = Vec4.createColor(0.2, 0.8, 0, 1)
local HEAD_SNAKE_SEGMENT = { 1, 2, 3, 4 }
local STRAIGHT_SNAKE_SEGMENT = { 9, 10 }
local CORNER_SNAKE_SEGMENT = { 5, 6, 7, 8 }
local TAIL_SNAKE_SEGMENT = { 11, 12, 13, 14 }

--- Resources

local snakeImage = Loader.loadImage("textures/snake-tilemap.png", false)
local environmentImage = Loader.loadImage("textures/env.png", false)
local fontResource = Loader.loadFont("fonts/m6x11.ttf")
local shaderResource = Loader.loadShader("shaders/base.glsl")

local canvas = Canvas.createCanvas(1200, 800)
canvas:setShader(shaderResource)

Io.setResizeable(true)
-- Io.setWindowSize(800, 600)
-- Io.centerWindow()

-- Check value set by plugin
Debug.print("v = ", _G.lua_plugin_template_version)

--- Game state
local screenSize = Io.getScreenSize()
local aspectRatio = screenSize.x / screenSize.y

-- Make this persistent across script reloads
if _G.paused == nil then
	_G.paused = false
end
if _G.gameover == nil then
	_G.gameover = false
end

local score = 0
local Direction = V2(0, -1)
local shouldGrow = false

local scalingFactor = 0.2
local timer = 0 -- 1 second timer

local function isclose(v1: Vec.Vec2, v2: Vec.Vec2, threshold: number)
	return (v1 - v2):length() < threshold
end

local applyShaderEffectForFrames = 0

function Load()
	screenSize = Io.getScreenSize()
	aspectRatio = screenSize.x / screenSize.y
end

local function resetSnake()
	_G.Snake = {
		V2(4, 2),
		V2(5, 2),
		V2(5, 3),
		V2(5, 4),
	}
end

local function isPositionValid(pos: Vec.Vec2)
	for _, segment in ipairs(_G.Snake) do
		if isclose(pos, segment, 0.001) then
			return false
		end
	end
	return pos.x >= 0 and pos.x <= 5 and pos.y >= 0 and pos.y <= 5
end

local function placeCherry()
	local newCherryPos = V2(math.random(0, 5), math.random(0, 5))
	local tries = 0
	while not isPositionValid(newCherryPos) do
		newCherryPos = V2(math.random(0, 5), math.random(0, 5))
		tries = tries + 1
		if tries > 1000 then
			break
		end
	end
	_G.Cherry = newCherryPos
end

if not _G.Snake then
	print("Initializing snake")
	resetSnake()
end

if not _G.Cherry then
	print("Initializing cherry")
	placeCherry()
end

local function gamePosToScreen(pos: Vec.Vec2)
	-- (0-10) -> (-1 - 1)
	return Vec.V2((pos.x - 3) * scalingFactor / aspectRatio, (pos.y - 3) * scalingFactor)
end

local function drawSnake(pos: Vec.Vec2, tileId: number)
	local tileSize = 1 / 16
	local size = V2(scalingFactor / aspectRatio, scalingFactor)

	local p1 = gamePosToScreen(pos)
	local p2 = V2(p1.x + size.x, p1.y)
	local p3 = V2(p1.x + size.x, p1.y + size.y)
	local p4 = V2(p1.x, p1.y + size.y)

	snakeImage:drawPart(p1, p2, p3, p4, V2(tileSize * tileId + 0.001, 0), V2(tileSize - 0.002, 1))
end

local function drawEnvironment()
	local p1 = gamePosToScreen(V2(-1, -1))
	local size = Vec.V2(scalingFactor / aspectRatio, scalingFactor):scale(8)
	-- fprint("drawn.")
	environmentImage:draw(p1, size)
end

local function mod4(x: number)
	return ((x - 1 + 4) % 4) + 1
end

local keyEvent = Event.getKeyDownEvent()
keyEvent:on(function(key: string)
	print("Key pressed: ", key)
	if key == "Space" then
		if _G.gameover and _G.paused then
			_G.gameover = false
			score = 0
			Direction = Vec.V2(0, -1)
			resetSnake()
			placeCherry()
		end
		_G.paused = not _G.paused
	end
end)

function Update(timeDelta: number)
	timer = timer + timeDelta

	if applyShaderEffectForFrames > 0 then
		canvas:setShader(shaderResource)
		applyShaderEffectForFrames = applyShaderEffectForFrames - 1
	else
		canvas:setShader(nil)
	end

	canvas:paint(function()
		Graphics.clear(_G.BG_COLOR)

		drawEnvironment()

		-- Draw Snake
		local Snake = _G.Snake
		for i, segment in ipairs(Snake) do
			if i == 1 then
				local diff = segment - Snake[2]
				local segmentType = HEAD_SNAKE_SEGMENT[1]
				if diff.x == 1 then
					segmentType = HEAD_SNAKE_SEGMENT[4]
				elseif diff.x == -1 then
					segmentType = HEAD_SNAKE_SEGMENT[2]
				elseif diff.y == 1 then
					segmentType = HEAD_SNAKE_SEGMENT[1]
				elseif diff.y == -1 then
					segmentType = HEAD_SNAKE_SEGMENT[3]
				end
				drawSnake(Vec.V2(segment.x, segment.y), segmentType)
				continue
			end

			if i == #Snake then
				local diff = segment - Snake[i - 1]
				local segmentType = TAIL_SNAKE_SEGMENT[1]
				if diff.x == 1 then
					segmentType = TAIL_SNAKE_SEGMENT[2]
				elseif diff.x == -1 then
					segmentType = TAIL_SNAKE_SEGMENT[3]
				elseif diff.y == 1 then
					segmentType = TAIL_SNAKE_SEGMENT[4]
				elseif diff.y == -1 then
					segmentType = TAIL_SNAKE_SEGMENT[1]
				end
				drawSnake(Vec.V2(segment.x, segment.y), segmentType)
				continue
			end

			local prev = Snake[i - 1]
			local next = Snake[i + 1]
			local diff = next - prev
			local is_straight = false
			if diff.x == 0 or diff.y == 0 then
				is_straight = true
			end

			local segmentType = 0
			if is_straight then
				if diff.x == 0 then
					segmentType = STRAIGHT_SNAKE_SEGMENT[1]
				else
					segmentType = STRAIGHT_SNAKE_SEGMENT[2]
				end
			else
				if (prev.x < segment.x and next.y < segment.y) or (next.x < segment.x and prev.y < segment.y) then
					segmentType = CORNER_SNAKE_SEGMENT[2]
				elseif (prev.x < segment.x and next.y > segment.y) or (next.x < segment.x and prev.y > segment.y) then
					segmentType = CORNER_SNAKE_SEGMENT[1]
				elseif (prev.x > segment.x and next.y > segment.y) or (next.x > segment.x and prev.y > segment.y) then
					segmentType = CORNER_SNAKE_SEGMENT[3]
				elseif (prev.x > segment.x and next.y < segment.y) or (next.x > segment.x and prev.y < segment.y) then
					segmentType = CORNER_SNAKE_SEGMENT[4]
				end
			end

			drawSnake(Vec.V2(segment.x, segment.y), segmentType)
		end

		--fprint("SPOS: ", Snake[1])

		-- Draw the next direction
		local head = Snake[1]
		local nextPos = head + Direction
		drawSnake(nextPos, 15)

		-- Draw Cherry
		local cherry = _G.Cherry
		drawSnake(cherry, 0)

		-- End of rendering --
		-- Start of logic --

		local newDirection = Direction
		if Io.isKeyDown("Z") or Io.isKeyDown("Up") then
			newDirection = Vec.V2(0, 1)
		elseif Io.isKeyDown("S") or Io.isKeyDown("Down") then
			newDirection = Vec.V2(0, -1)
		elseif Io.isKeyDown("Q") or Io.isKeyDown("Left") then
			newDirection = Vec.V2(-1, 0)
		elseif Io.isKeyDown("D") or Io.isKeyDown("Right") then
			newDirection = Vec.V2(1, 0)
		end

		if not isclose(head + newDirection, Snake[2], 0.001) then
			Direction = newDirection
		end

		-- Move Snake
		if timer > 0.5 then
			if not _G.paused then
				local newHead = Snake[1] + Direction
				if isPositionValid(newHead) then
					table.insert(Snake, 1, newHead)
					if not shouldGrow then
						table.remove(Snake)
					end
					shouldGrow = false
				end
			end
			timer = 0
		end

		-- If no possible new position is valid, game over
		head = Snake[1]
		local possiblePositions = {
			head + V2(1, 0),
			head + V2(-1, 0),
			head + V2(0, 1),
			head + V2(0, -1),
		}
		local canMove = false
		for _, pos in ipairs(possiblePositions) do
			if isPositionValid(pos) then
				canMove = true
				break
			end
		end
		if not canMove then
			_G.paused = true
			_G.gameover = true
			score = #Snake
		end

		if _G.paused then
			if _G.gameover then
				if fontResource:isReady() then
					fontResource:drawText("GAME OVER", V2(-0.3, 0.4), 0.2, Vec4.WHITE)
					fontResource:drawText("Score: " .. tostring(score), V2(-0.2, 0.25), 0.15, Vec4.WHITE)
					fontResource:drawText("Press SPACE to restart", V2(-0.35, -0.1), 0.1, Vec4.WHITE)
				end
			else
				if fontResource:isReady() then
					fontResource:drawText("PAUSED", V2(-0.3, 0.3), 0.3, Vec4.WHITE)
				end
			end
		end

		-- Eat cherry
		if isclose(head, cherry, 0.001) then
			-- Grow snake
			print("Eating cherry at ", cherry)
			applyShaderEffectForFrames = 30
			shouldGrow = true
			-- Move cherry
			placeCherry()
		end
	end) -- end of canvas paint

	canvas:draw(V2(-1, -1), V2(2, 2))
end
