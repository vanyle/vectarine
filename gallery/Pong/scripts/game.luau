const graphics = require("@vectarine/graphics")
const io = require("@vectarine/io")
const loader = require("@vectarine/loader")
const persist = require("@vectarine/persist")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

const fontResource = loader.loadFont("fonts/m6x11.ttf")

const Ball = persist.onReload({
	position = vec.V2(0, 0),
	velocity = vec.V2(1, -1.3),
}, "ball")

const Player = persist.onReload({
	position = 0,
	score = 0,
}, "player")

const Opponent = persist.onReload({
	position = 0,
	score = 0,
}, "opponent")

const racketSize = 0.4
const ballSize = 0.1
local windowRatio = io.getWindowSize().x / io.getWindowSize().y

function Update(deltaTime: number)
	graphics.clear(vec4.BLACK)
	if graphics.drawSplashScreenIfNeeded({ fontResource }, "PONG!") then
		return
	end
	windowRatio = io.getWindowSize().x / io.getWindowSize().y

	-- Drawing
	graphics.drawRect(Ball.position, vec.V2(ballSize, ballSize * windowRatio), vec4.RED)

	drawRectWithBorder(vec.V2(-0.9, Player.position), vec.V2(0.1, racketSize), vec4.BLACK, vec4.WHITE, 0.02)
	drawRectWithBorder(vec.V2(0.8, Opponent.position), vec.V2(0.1, racketSize), vec4.BLACK, vec4.WHITE, 0.02)

	local scoreText = Player.score .. " - " .. Opponent.score
	local measurements = fontResource:measureText(scoreText, 0.2)
	fontResource:drawText(scoreText, vec.V2(-measurements.width / 2, 0), 0.2, vec4.WHITE)

	-- Player input
	if io.isKeyDown("Up") then
		Player.position += deltaTime
	end
	if io.isKeyDown("Down") then
		Player.position -= deltaTime
	end
	Player.position = math.clamp(Player.position, -1, 1 - racketSize)

	-- Opponent movement
	if Ball.position.y < Opponent.position + racketSize / 2 then
		Opponent.position -= deltaTime / 1.1
	end
	if Ball.position.y > Opponent.position + racketSize / 2 then
		Opponent.position += deltaTime / 1.1
	end
	Opponent.position = math.clamp(Opponent.position, -1, 1 - racketSize)

	-- Physics
	Ball.position += Ball.velocity:scale(deltaTime)
	if Ball.position.y < -1 then
		Ball.velocity.y = math.abs(Ball.velocity.y)
	end
	if Ball.position.y > 1 then
		Ball.velocity.y = -math.abs(Ball.velocity.y)
	end

	if Ball.position.x < -1.1 then
		Opponent.score += 1
		resetBall()
	elseif Ball.position.x < -0.8 then
		if
			Ball.position.y > Player.position - ballSize * windowRatio
			and Ball.position.y < Player.position + racketSize
		then
			Ball.velocity.x = math.abs(Ball.velocity.x)
			Ball.velocity.y = 2 * math.random()
		end
	end

	if Ball.position.x > 1 then
		Player.score += 1
		resetBall()
	elseif Ball.position.x > 0.7 then
		if
			Ball.position.y > Opponent.position - ballSize * windowRatio
			and Ball.position.y < Opponent.position + racketSize
		then
			Ball.velocity.x = -math.abs(Ball.velocity.x)
			Ball.velocity.y = 2 * math.random()
		end
	end
end

function resetBall()
	Ball.position = vec.V2(0, 0)
	Ball.velocity.y = math.random() * 2
end

function drawRectWithBorder(
	position: vec.Vec2,
	size: vec.Vec2,
	color: vec4.Vec4,
	borderColor: vec4.Vec4,
	borderWidth: number
)
	graphics.drawRect(
		position - vec.V2(borderWidth, borderWidth * windowRatio),
		size + vec.V2(borderWidth, borderWidth * windowRatio):scale(2),
		borderColor
	)
	graphics.drawRect(position, size, color)
end
