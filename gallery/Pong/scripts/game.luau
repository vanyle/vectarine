local Graphics = require('@vectarine/graphics')
local Io = require('@vectarine/io')
local Loader = require('@vectarine/loader')
local Persist = require('@vectarine/persist')
local Vec = require('@vectarine/vec')
local Vec4 = require('@vectarine/vec4')

local fontResource = Loader.loadFont("fonts/m6x11.ttf")

local Ball = Persist.onReload({
	position = Vec.V2(0, 0),
	velocity = Vec.V2(1, -1.3),
}, "ball")

local Player = Persist.onReload({
	position = 0,
	score = 0,
}, "player")

local Opponent = Persist.onReload({
	position = 0,
	score = 0,
}, "opponent")

const racketSize = 0.4
const ballSize = 0.1
local windowRatio = Io.getWindowSize().x / Io.getWindowSize().y 


function Update(deltaTime: number)
	Graphics.clear(Vec4.BLACK)
	if Graphics.drawSplashScreenIfNeeded({fontResource}, "PONG!") then
		return
	end
    windowRatio = Io.getWindowSize().x / Io.getWindowSize().y 

	-- Drawing
	Graphics.drawRect(Ball.position, Vec.V2(ballSize, ballSize*windowRatio), Vec4.RED)

	drawRectWithBorder(Vec.V2(-0.9, Player.position), Vec.V2(0.1, racketSize), Vec4.BLACK, Vec4.WHITE, 0.02)
	drawRectWithBorder(Vec.V2(0.8, Opponent.position), Vec.V2(0.1, racketSize), Vec4.BLACK, Vec4.WHITE, 0.02)

	local scoreText = Player.score .. " - " .. Opponent.score
	local measurements = fontResource:measureText(scoreText, 0.2)
	fontResource:drawText(scoreText, Vec.V2(-measurements.width / 2, 0), 0.2, Vec4.WHITE)

	-- Player input
	if Io.isKeyDown("Up") then
		Player.position += deltaTime
	end
	if Io.isKeyDown("Down") then
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
		if Ball.position.y > Player.position - ballSize*windowRatio and Ball.position.y < Player.position + racketSize then
			Ball.velocity.x = math.abs(Ball.velocity.x)
			Ball.velocity.y = 2 * math.random()
		end
	end

	if Ball.position.x > 1 then
		Player.score += 1
		resetBall()
	elseif Ball.position.x > 0.7 then
		if Ball.position.y > Opponent.position - ballSize*windowRatio and Ball.position.y < Opponent.position + racketSize then
			Ball.velocity.x = -math.abs(Ball.velocity.x)
			Ball.velocity.y = 2 * math.random()
		end
	end
end

function resetBall()
	Ball.position = Vec.V2(0, 0)
	Ball.velocity.y = math.random() * 2
end

function drawRectWithBorder(
	position: Vec.Vec2,
	size: Vec.Vec2,
	color: Vec4.Vec4,
	borderColor: Vec4.Vec4,
	borderWidth: number
)
	Graphics.drawRect(
		position - Vec.V2(borderWidth, borderWidth*windowRatio),
		size + Vec.V2(borderWidth, borderWidth*windowRatio):scale(2),
		borderColor
	)
	Graphics.drawRect(position, size, color)
end
