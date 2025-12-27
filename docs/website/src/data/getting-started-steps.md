## Drawing a circle

Let's start this pong by drawing the ball as a red circle.

(0,0) means the center of the screen while (1,1) is the top-right edge.

```luau game.luau
local Debug = require("@vectarine/debug")
local Graphics = require("@vectarine/graphics")
local Vec4 = require("@vectarine/vec4")
local Vec = require("@vectarine/vec")

local Ball = {
	position = Vec.V2(0, 0),
}

function Update(deltaTime: number)
	Graphics.clear(Vec4.WHITE)
	Graphics.drawCircle(Ball.position, 0.1, Vec4.RED)
end
```

## Making the ball bounce around

Every frame, we add the velocity of the ball to its position to make it move.
When the ball reaches an edge, we flip the sign of the velocity to make it bounce.

```luau game.luau
local Debug = require("@vectarine/debug")
local Graphics = require("@vectarine/graphics")
local Vec4 = require("@vectarine/vec4")
local Vec = require("@vectarine/vec")

local Ball = {
	position = Vec.V2(0, 0),
	velocity = Vec.V2(1, 1.5),
}

function Update(deltaTime: number)
	Graphics.clear(Vec4.WHITE)
	Graphics.drawCircle(Ball.position, 0.1, Vec4.RED)

	Ball.position += Ball.velocity:scale(deltaTime)

	if Ball.position.y < -1 then
		Ball.velocity.y = math.abs(Ball.velocity.y)
	end
	if Ball.position.y > 1 then
		Ball.velocity.y = -math.abs(Ball.velocity.y)
	end

	if Ball.position.x < -1 then
		Ball.velocity.x = math.abs(Ball.velocity.x)
	end
	if Ball.position.x > 1 then
		Ball.velocity.x = -math.abs(Ball.velocity.x)
	end
end
```

## Persisting the position on reload

When typing and saving, it is convienient to preserve the value of variables instead of resetting everything.
Vectarine comes with a function called Persist.onReload which does that.

```luau game.luau
local Debug = require("@vectarine/debug")
local Graphics = require("@vectarine/graphics")
local Vec4 = require("@vectarine/vec4")
local Vec = require("@vectarine/vec")

local Persist = require("@vectarine/persist")

local Ball = Persist.onReload({
	position = Vec.V2(0, 0),
	velocity = Vec.V2(1, 1.5),
}, "ball")

function Update(deltaTime: number)
	Graphics.clear(Vec4.WHITE)
	Graphics.drawCircle(Ball.position, 0.1, Vec4.RED)

	Ball.position += Ball.velocity:scale(deltaTime)

	if Ball.position.y < -1 then
		Ball.velocity.y = math.abs(Ball.velocity.y)
	end
	if Ball.position.y > 1 then
		Ball.velocity.y = -math.abs(Ball.velocity.y)
	end

	if Ball.position.x < -1 then
		Ball.velocity.x = math.abs(Ball.velocity.x)
	end
	if Ball.position.x > 1 then
		Ball.velocity.x = -math.abs(Ball.velocity.x)
	end
end
```

## Let's add the player

We can use the Io module to get keyboard input and react to it.
To keep the code short, I'm moving the ball moving logic to a function called moveBall()

```luau game.luau
local Graphics = require("@vectarine/graphics")
local Vec4 = require("@vectarine/vec4")
local Vec = require("@vectarine/vec")
local Io = require("@vectarine/Io")
local Persist = require("@vectarine/persist")

local Ball = Persist.onReload({
	position = Vec.V2(0, 0),
	velocity = Vec.V2(1, 1.5),
}, "ball")

local Player = Persist.onReload({
	position = 0,
}, "Player")

local racketSize = 0.4

function Update(deltaTime: number)
	Graphics.clear(Vec4.WHITE)
	Graphics.drawCircle(Ball.position, 0.1, Vec4.RED)
	Graphics.drawRect(Vec.V2(-0.9, Player.position), Vec.V2(0.1, racketSize))

	if Io.isKeyDown("Up") then
		Player.position += deltaTime
	end
	if Io.isKeyDown("Down") then
		Player.position -= deltaTime
	end

	moveBall()
end
```

## And his opponent

The opponent is a simple AI which follows the ball.

```luau game.luau
local Graphics = require("@vectarine/graphics")
local Vec4 = require("@vectarine/vec4")
local Vec = require("@vectarine/vec")
local Io = require("@vectarine/Io")
local Persist = require("@vectarine/persist")

local Ball = Persist.onReload({
	position = Vec.V2(0, 0),
	velocity = Vec.V2(1, 1.5),
}, "ball")

local Player = Persist.onReload({
	position = 0,
}, "player")

local Opponent = Persist.onReload({
	position = 0,
}, "opponent")

local racketSize = 0.4

function Update(deltaTime: number)
	Graphics.clear(Vec4.WHITE)
	Graphics.drawCircle(Ball.position, 0.1, Vec4.RED)
	Graphics.drawRect(Vec.V2(-0.9, Player.position), Vec.V2(0.1, racketSize))
	Graphics.drawRect(Vec.V2(0.8, Opponent.position), Vec.V2(0.1, racketSize))

	if Io.isKeyDown("Up") then
		Player.position += deltaTime
	end
	if Io.isKeyDown("Down") then
		Player.position -= deltaTime
	end

	if Ball.position.y < Opponent.position + racketSize / 2 then
		Opponent.position -= deltaTime / 1.1
	end
	if Ball.position.y > Opponent.position + racketSize / 2 then
		Opponent.position += deltaTime / 1.1
	end

	moveBall()
end
```

## Adding collisions

The player and opponent rackets need to be able to touch the ball.
I'm putting the code to move the player and the opponent to a separate function to keep the code short, and expanding
back the moveBall function to add additional reset logic.

```luau game.luau
local Graphics = require("@vectarine/graphics")
local Vec4 = require("@vectarine/vec4")
local Vec = require("@vectarine/vec")
local Io = require("@vectarine/Io")
local Persist = require("@vectarine/persist")

-- Initialization stays the same

function resetBall()
	Ball.position = Vec.V2(0, 0)
	Ball.velocity.y = math.random() * 2
end

function Update(deltaTime: number)
	Graphics.clear(Vec4.WHITE)
	Graphics.drawCircle(Ball.position, 0.1, Vec4.RED)
	Graphics.drawRect(Vec.V2(-0.9, Player.position), Vec.V2(0.1, racketSize))
	Graphics.drawRect(Vec.V2(0.8, Opponent.position), Vec.V2(0.1, racketSize))

	movePlayer()
	moveOpponent()

	Ball.position += Ball.velocity:scale(deltaTime)
	if Ball.position.y < -1 then
		Ball.velocity.y = math.abs(Ball.velocity.y)
	end
	if Ball.position.y > 1 then
		Ball.velocity.y = -math.abs(Ball.velocity.y)
	end

    -- We add a resetBall to the ball moving logic
	if Ball.position.x < -1 then
		Opponent.score += 1
		resetBall()
	elseif Ball.position.x < -0.8 then
		if Ball.position.y > Player.position and Ball.position.y < Player.position + racketSize then
			Ball.velocity.x = math.abs(Ball.velocity.x)
		end
	end

	if Ball.position.x > 1 then
		Player.score += 1
		resetBall()
	elseif Ball.position.x > 0.8 then
		if Ball.position.y > Opponent.position and Ball.position.y < Opponent.position + racketSize then
			Ball.velocity.x = -math.abs(Ball.velocity.x)
		end
	end
end
```

## Adding a score

We can use the Text module to draw the score.

```luau game.luau
local Graphics = require("@vectarine/graphics")
local Vec4 = require("@vectarine/vec4")
local Vec = require("@vectarine/vec")
local Io = require("@vectarine/Io")
local Persist = require("@vectarine/persist")
local Text = require("@vectarine/text")

-- Initialization stays the same

local racketSize = 0.4

function resetBall()
	Ball.position = Vec.V2(0, 0)
	Ball.velocity.y = math.random() * 2
end

function Update(deltaTime: number)
	Graphics.clear(Vec4.WHITE)
	-- Drawing
	Graphics.drawCircle(Ball.position, 0.1, Vec4.RED)
	Graphics.drawRect(Vec.V2(-0.9, Player.position), Vec.V2(0.1, racketSize))
	Graphics.drawRect(Vec.V2(0.8, Opponent.position), Vec.V2(0.1, racketSize))

	local scoreText = Player.score .. " - " .. Opponent.score
	local measurements = Text.font:measureText(scoreText, 0.2)
	Text.font:drawText(scoreText, Vec.V2(-measurements.width / 2, 0), 0.2)

	movePlayer()
	moveOpponent()

	Ball.position += Ball.velocity:scale(deltaTime)
	if Ball.position.y < -1 then
		Ball.velocity.y = math.abs(Ball.velocity.y)
	end
	if Ball.position.y > 1 then
		Ball.velocity.y = -math.abs(Ball.velocity.y)
	end

	if Ball.position.x < -1 then
		Opponent.score += 1
		resetBall()
	elseif Ball.position.x < -0.8 then
		if Ball.position.y > Player.position and Ball.position.y < Player.position + racketSize then
			Ball.velocity.x = math.abs(Ball.velocity.x)
			Ball.velocity.y = 2 * math.random()
		end
	end

	if Ball.position.x > 1 then
		Player.score += 1
		resetBall()
	elseif Ball.position.x > 0.8 then
		if Ball.position.y > Opponent.position and Ball.position.y < Opponent.position + racketSize then
			Ball.velocity.x = -math.abs(Ball.velocity.x)
			Ball.velocity.y = 2 * math.random()
		end
	end
end
```
