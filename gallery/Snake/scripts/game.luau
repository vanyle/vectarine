const debug = require("@vectarine/debug")
_G.print = debug.print

const canvas = require("@vectarine/canvas")
const event = require("@vectarine/event")
const graphics = require("@vectarine/graphics")
const io = require("@vectarine/io")
const loader = require("@vectarine/loader")
const persist = require("@vectarine/persist")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")
const V2 = vec.V2

_G.BG_COLOR = vec4.createColor(132 / 255, 160 / 255, 189 / 255, 1)
const _SNAKE_COLOR = vec4.createColor(0.2, 0.8, 0, 1)
const HEAD_SNAKE_SEGMENT = { 1, 2, 3, 4 }
const STRAIGHT_SNAKE_SEGMENT = { 9, 10 }
const CORNER_SNAKE_SEGMENT = { 5, 6, 7, 8 }
const TAIL_SNAKE_SEGMENT = { 11, 12, 13, 14 }

--- Resources

const snakeImage = loader.loadImage("textures/snake-tilemap.png", false)
const environmentImage = loader.loadImage("textures/env.png", false)
const fontResource = loader.loadFont("fonts/m6x11.ttf")
const shaderResource = loader.loadShader("shaders/base.glsl")

const gameCanvas = canvas.createCanvas(1200, 800)
gameCanvas:setShader(shaderResource)

io.setResizeable(true)

-- Make this persistent across script reloads
local state = persist.onReload({
	paused = false,
	gameover = false,
}, "state")

local score = 0
local Direction = V2(0, -1)
local shouldGrow = false

const scalingFactor = 0.2
local timer = 0 -- 1 second timer

local function isclose(v1: vec.Vec2, v2: vec.Vec2, threshold: number)
	return (v1 - v2):length() < threshold
end

local applyShaderEffectForFrames = 0

local function resetSnake()
	_G.Snake = {
		V2(4, 2),
		V2(5, 2),
		V2(5, 3),
		V2(5, 4),
	}
end

local function isPositionValid(pos: vec.Vec2)
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
	debug.print("Initializing snake")
	resetSnake()
end

if not _G.Cherry then
	debug.print("Initializing cherry")
	placeCherry()
end

local function drawSnake(pos: vec.Vec2, tileId: number)
	local tileSize = 1 / 16
	local p2 = V2(pos.x + 1, pos.y)
	local p3 = V2(pos.x + 1, pos.y + 1)
	local p4 = V2(pos.x, pos.y + 1)
	snakeImage:drawPart(pos, p2, p3, p4, V2(tileSize * tileId + 0.001, 0), V2(tileSize - 0.002, 1))
end

local function drawEnvironment()
	environmentImage:draw(V2(-1, -1), V2(8, 8))
end

local function mod4(x: number)
	return ((x - 1 + 4) % 4) + 1
end

local keyEvent = event.getKeyDownEvent()
keyEvent:on("resetGame", function(key: string)
	debug.print("Key pressed: ", key)
	if key == "Space" then
		if state.gameover and state.paused then
			state.gameover = false
			score = 0
			Direction = vec.V2(0, -1)
			resetSnake()
			placeCherry()
		end
		state.paused = not state.paused
	end
end)

function Update(timeDelta: number)
	timer = timer + timeDelta

	if applyShaderEffectForFrames > 0 then
		gameCanvas:setShader(shaderResource)
		applyShaderEffectForFrames = applyShaderEffectForFrames - 1
	else
		gameCanvas:setShader(nil)
	end

	local Snake: { vec.Vec2 } = _G.Snake
	local head = Snake[1]
	local cherry: vec.Vec2 = _G.Cherry

	local windowSize = io.getWindowSize()
	local aspectRatio = windowSize.x / windowSize.y

	gameCanvas:paint(function()
		graphics.withTransformation(
			{ translation = V2(-3, -3), scale = V2(scalingFactor / aspectRatio, scalingFactor) },
			function()
				graphics.clear(_G.BG_COLOR)

				drawEnvironment()

				-- Draw Snake
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
						drawSnake(vec.V2(segment.x, segment.y), segmentType)
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
						drawSnake(vec.V2(segment.x, segment.y), segmentType)
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
						if
							(prev.x < segment.x and next.y < segment.y) or (next.x < segment.x and prev.y < segment.y)
						then
							segmentType = CORNER_SNAKE_SEGMENT[2]
						elseif
							(prev.x < segment.x and next.y > segment.y) or (next.x < segment.x and prev.y > segment.y)
						then
							segmentType = CORNER_SNAKE_SEGMENT[1]
						elseif
							(prev.x > segment.x and next.y > segment.y) or (next.x > segment.x and prev.y > segment.y)
						then
							segmentType = CORNER_SNAKE_SEGMENT[3]
						elseif
							(prev.x > segment.x and next.y < segment.y) or (next.x > segment.x and prev.y < segment.y)
						then
							segmentType = CORNER_SNAKE_SEGMENT[4]
						end
					end

					drawSnake(vec.V2(segment.x, segment.y), segmentType)
				end

				-- Draw the next direction
				local nextPos = head + Direction
				drawSnake(nextPos, 15)

				-- Draw Cherry
				drawSnake(cherry, 0)
			end
		)
		-- End of rendering --
		-- Start of logic --

		local newDirection = Direction
		if io.isKeyDown("Z") or io.isKeyDown("Up") then
			newDirection = vec.V2(0, 1)
		elseif io.isKeyDown("S") or io.isKeyDown("Down") then
			newDirection = vec.V2(0, -1)
		elseif io.isKeyDown("Q") or io.isKeyDown("Left") then
			newDirection = vec.V2(-1, 0)
		elseif io.isKeyDown("D") or io.isKeyDown("Right") then
			newDirection = vec.V2(1, 0)
		end

		if not isclose(head + newDirection, Snake[2], 0.001) then
			Direction = newDirection
		end

		-- Move Snake
		if timer > 0.5 then
			if not state.paused then
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
			state.paused = true
			state.gameover = true
			score = #Snake
		end

		if state.paused then
			if state.gameover then
				if fontResource:isReady() then
					fontResource:drawText("GAME OVER", V2(-0.3, 0.4), 0.2, vec4.WHITE)
					fontResource:drawText("Score: " .. tostring(score), V2(-0.2, 0.25), 0.15, vec4.WHITE)
					fontResource:drawText("Press SPACE to restart", V2(-0.35, -0.1), 0.1, vec4.WHITE)
				end
			else
				if fontResource:isReady() then
					fontResource:drawText("PAUSED", V2(-0.3, 0.3), 0.3, vec4.WHITE)
				end
			end
		end

		-- Eat cherry
		if isclose(head, cherry, 0.001) then
			-- Grow snake
			debug.print("Eating cherry at ", cherry)
			applyShaderEffectForFrames = 30
			shouldGrow = true
			-- Move cherry
			placeCherry()
		end
	end) -- end of canvas paint

	gameCanvas:draw(V2(-1, -1), V2(2, 2))
end
