local Graphics = require("@vectarine/graphics")
local Io = require("@vectarine/io")
local Loader = require("@vectarine/loader")
local Persist = require("@vectarine/persist")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")
const V2 = Vec.V2

const fontResource = Loader.loadFont("fonts/m6x11.ttf")

const TOP = 1
const BOTTOM = -1
const PLAYER_Y = -0.8

const PLAYER_SPEED = 1.4
const PLAYER_START_HEALTH = 3
const PLAYER_CELL = V2(0.024, 0.024)
const PLAYER_HALF = V2(0.084, 0.072)

const PROJECTILE_SPEED = 2.0
const PROJECTILE_SIZE = V2(0.02, 0.08)
const PROJECTILE_HALF = V2(0.012, 0.04)
const SHOOT_COOLDOWN = 0.22

const ALIEN_CELL = V2(0.015, 0.015)
const ALIEN_HALF = V2(0.078, 0.058)
const ALIEN_BASE_SPEED = 0.22
const WAVE_BASE_COUNT = 4
const WAVE_DELAY = 1.2

-- Colors
const COLOR_BG = Vec4.createColor(0.04, 0.04, 0.09, 1)
const COLOR_PLAYER = Vec4.createColor(0.35, 0.9, 0.45, 1)
const COLOR_PLAYER_DARK = Vec4.createColor(0.15, 0.55, 0.3, 1)
const COLOR_FLAME = Vec4.createColor(1, 0.6, 0.1, 1)
const COLOR_PROJECTILE = Vec4.createColor(1, 0.95, 0.3, 1)
const COLOR_TEXT = Vec4.WHITE
const COLOR_DIM = Vec4.createColor(0.6, 0.6, 0.7, 1)
const COLOR_HEART = Vec4.createColor(0.95, 0.2, 0.3, 1)

const WAVE_COLORS = {
	Vec4.createColor(0.9, 0.3, 0.9, 1),
	Vec4.createColor(0.3, 0.8, 0.95, 1),
	Vec4.createColor(0.95, 0.5, 0.25, 1),
	Vec4.createColor(0.5, 0.9, 0.4, 1),
}

const PLAYER_GRID = {
	"0001000",
	"0011100",
	"0111110",
	"1111111",
	"1101011",
	"0100010",
}

const ALIEN_GRID = {
	"  1     1  ",
	"   1   1   ",
	"  1111111  ",
	" 11 111 11 ",
	"111 111 111",
	"1 1111111 1",
	"1 1     1 1",
	" 1 1   1 1 ",
}

const ALIEN_GRID2 = {
	"           ",
	"    111    ",
	"  111 111  ",
	" 111   111 ",
	"11111 11111",
	"1 1111111 1",
	"1 1 1 1 1 1",
	"   1   1   ",
}

const HEART_GRID = {
	"0110110",
	"1111111",
	"1111111",
	"0111110",
	"0011100",
	"0001000",
}

-- MARK: Game state (persisted across hot reloads)

type Alien = {
	pos: Vec.Vec2,
	baseX: number,
	speed: number,
	phase: number,
	wobble: number,
	sprite_type: number,
	color: Vec4.Vec4,
	dead: boolean,
}

type Projectile = {
	pos: Vec.Vec2,
	dead: boolean,
}

local function makeInitialState()
	return {
		phase = "playing", -- "playing" | "gameover"
		health = PLAYER_START_HEALTH,
		score = 0,
		wave = 0,
		playerX = 0,
		aliens = {} :: { Alien },
		projectiles = {} :: { Projectile },
		shootTimer = 0,
		waveDelay = WAVE_DELAY,
		time = 0,
	}
end

local State = Persist.onReload(makeInitialState(), "alienShooterState")

-- Recomputed every frame from the window size.
local aspectRatio = 1
local fieldHalfWidth = 1

-- MARK: Helpers

local function clamp(value: number, low: number, high: number): number
	if value < low then
		return low
	elseif value > high then
		return high
	end
	return value
end

local function overlaps(c1: Vec.Vec2, half1: Vec.Vec2, c2: Vec.Vec2, half2: Vec.Vec2): boolean
	return math.abs(c1.x - c2.x) <= half1.x + half2.x and math.abs(c1.y - c2.y) <= half1.y + half2.y
end

--- Draws a blocky pixel-art sprite centered on `center`.
local function drawPixelSprite(center: Vec.Vec2, cell: Vec.Vec2, rows: { string }, color: Vec4.Vec4)
	local height = #rows
	local width = #rows[1]
	local originX = center.x - (width * cell.x) / 2
	local originY = center.y - (height * cell.y) / 2
	for r = 1, height do
		local row = rows[r]
		for c = 1, width do
			if string.sub(row, c, c) == "1" then
				local cellX = originX + (c - 1) * cell.x
				local cellY = originY + (height - r) * cell.y
				Graphics.drawRect(V2(cellX, cellY), cell, color)
			end
		end
	end
end

local function playerCenter(): Vec.Vec2
	return V2(State.playerX, PLAYER_Y)
end

-- MARK: Game logic

local function resetGame()
	State.phase = "playing"
	State.health = PLAYER_START_HEALTH
	State.score = 0
	State.wave = 0
	State.playerX = 0
	State.aliens = {}
	State.projectiles = {}
	State.shootTimer = 0
	State.waveDelay = WAVE_DELAY
end

local function spawnWave(waveNumber: number)
	local count = WAVE_BASE_COUNT + waveNumber
	local cols = math.min(count, 6)
	local speed = ALIEN_BASE_SPEED + (waveNumber - 1) * 0.045
	local color = WAVE_COLORS[((waveNumber - 1) % #WAVE_COLORS) + 1]
	local sprite_type = if waveNumber % 2 == 1 then 1 else 2
	local spreadX = fieldHalfWidth * 0.82

	for i = 0, count - 1 do
		local col = i % cols
		local row = math.floor(i / cols)
		local t = if cols > 1 then col / (cols - 1) else 0.5
		local baseX = -spreadX + t * (2 * spreadX)
		table.insert(State.aliens, {
			pos = V2(baseX, TOP + 0.15 + row * 0.22),
			baseX = baseX,
			speed = speed,
			sprite_type = sprite_type,
			phase = math.random() * math.pi * 2,
			wobble = 0.1 + math.random() * 0.12,
			color = color,
			dead = false,
		})
	end
end

local function shoot()
	table.insert(State.projectiles, {
		pos = V2(State.playerX, PLAYER_Y + PLAYER_HALF.y + 0.02),
		dead = false,
	})
	State.shootTimer = SHOOT_COOLDOWN
end

local function updatePlayer(dt: number)
	if Io.isKeyDown("Left") then
		State.playerX -= PLAYER_SPEED * dt
	elseif Io.isKeyDown("Right") then
		State.playerX += PLAYER_SPEED * dt
	end
	State.playerX = clamp(State.playerX, -fieldHalfWidth + PLAYER_HALF.x, fieldHalfWidth - PLAYER_HALF.x)

	State.shootTimer -= dt
	if Io.isKeyDown("Space") and State.shootTimer <= 0 then
		shoot()
	end
end

local function updateProjectiles(dt: number)
	local survivors = {}
	for _, p in State.projectiles do
		p.pos.y += PROJECTILE_SPEED * dt
		if not p.dead and p.pos.y < TOP + 0.1 then
			table.insert(survivors, p)
		end
	end
	State.projectiles = survivors
end

local function updateAliens(dt: number)
	for _, a in State.aliens do
		a.pos.y -= a.speed * dt
		a.pos.x = a.baseX + math.sin(State.time * 2 + a.phase) * a.wobble
	end
end

local function handleCollisions()
	-- Projectiles destroy aliens.
	for _, p in State.projectiles do
		for _, a in State.aliens do
			if not a.dead and not p.dead and overlaps(p.pos, PROJECTILE_HALF, a.pos, ALIEN_HALF) then
				a.dead = true
				p.dead = true
				State.score += 10
				break
			end
		end
	end

	-- Aliens touching the player cost health.
	local center = playerCenter()
	for _, a in State.aliens do
		if not a.dead and overlaps(center, PLAYER_HALF, a.pos, ALIEN_HALF) then
			a.dead = true
			State.health -= 1
		end
	end

	-- Rebuild alien list, removing dead ones and those that escaped past the bottom.
	local survivors = {}
	for _, a in State.aliens do
		if a.dead then
			continue
		end
		if a.pos.y < BOTTOM - 0.15 then
			State.health -= 1
			continue
		end
		table.insert(survivors, a)
	end
	State.aliens = survivors

	-- Drop spent projectiles.
	local liveProjectiles = {}
	for _, p in State.projectiles do
		if not p.dead then
			table.insert(liveProjectiles, p)
		end
	end
	State.projectiles = liveProjectiles

	if State.health <= 0 then
		State.health = 0
		State.phase = "gameover"
	end
end

local function updateWaves(dt: number)
	if #State.aliens > 0 then
		return
	end
	State.waveDelay -= dt
	if State.waveDelay <= 0 then
		State.wave += 1
		spawnWave(State.wave)
		State.waveDelay = WAVE_DELAY
	end
end

-- MARK: Drawing

local function drawPlayer()
	local center = playerCenter()

	-- Engine flame flickers based on time.
	local flicker = 0.5 + 0.5 * math.abs(math.sin(State.time * 22))
	local flameHeight = 0.05 * flicker
	local flameY = center.y - PLAYER_HALF.y - flameHeight
	Graphics.drawRect(V2(center.x - 0.03, flameY), V2(0.02, flameHeight), COLOR_FLAME)
	Graphics.drawRect(V2(center.x + 0.01, flameY), V2(0.02, flameHeight), COLOR_FLAME)

	drawPixelSprite(center, PLAYER_CELL, PLAYER_GRID, COLOR_PLAYER)
	-- Cockpit accent.
	Graphics.drawRect(V2(center.x - 0.012, center.y - 0.01), V2(0.024, 0.03), COLOR_PLAYER_DARK)
end

local function drawAliens()
	for _, a in State.aliens do
		if a.sprite_type == 1 then
			drawPixelSprite(a.pos, ALIEN_CELL, ALIEN_GRID, a.color)
		else
			drawPixelSprite(a.pos, ALIEN_CELL, ALIEN_GRID2, a.color)
		end
	end
end

local function drawProjectiles()
	for _, p in State.projectiles do
		local corner = p.pos - PROJECTILE_HALF
		Graphics.drawRect(corner, PROJECTILE_SIZE, COLOR_PROJECTILE)
		Graphics.drawRect(V2(p.pos.x - 0.004, corner.y), V2(0.008, PROJECTILE_SIZE.y), Vec4.WHITE)
	end
end

local function drawWorld()
	Graphics.withTransformation({ scale = V2(1 / aspectRatio, 1) }, function()
		drawAliens()
		drawProjectiles()
		drawPlayer()
	end)
end

local function drawHud()
	fontResource:drawText("SCORE " .. State.score, V2(-0.97, 0.90), 0.09, COLOR_TEXT)
	fontResource:drawText("WAVE " .. State.wave, V2(-0.97, 0.84), 0.07, COLOR_DIM)

	Graphics.withTransformation({ scale = V2(1 / aspectRatio, 1) }, function()
		for i = 1, State.health do
			local x = fieldHalfWidth - 0.09 - (i - 1) * 0.14
			drawPixelSprite(V2(x, 0.9), V2(0.016, 0.016), HEART_GRID, COLOR_HEART)
		end
	end)
end

local function drawCenteredText(text: string, y: number, size: number, color: Vec4.Vec4)
	local measurements = fontResource:measureText(text, size)
	fontResource:drawText(text, V2(-measurements.width / 2, y), size, color)
end

local function drawGameOver()
	-- Dim the frozen scene.
	Graphics.drawRect(V2(-1, -1), V2(2, 2), Vec4.createColor(0, 0, 0, 0.6))
	drawCenteredText("GAME OVER", 0.35, 0.18, COLOR_HEART)
	drawCenteredText("Final Score: " .. State.score, 0.05, 0.1, COLOR_TEXT)
	drawCenteredText("Reached Wave " .. State.wave, -0.1, 0.08, COLOR_DIM)
	drawCenteredText("Press Space to play again", -0.4, 0.08, COLOR_TEXT)
end

-- MARK: Entry point

function Update(deltaTime: number)
	Graphics.clear(COLOR_BG)
	if Graphics.drawSplashScreenIfNeeded({ fontResource }, "Alien Shooter") then
		return
	end

	aspectRatio = Io.getWindowSize().x / Io.getWindowSize().y
	fieldHalfWidth = aspectRatio
	State.time += deltaTime

	if State.phase == "playing" then
		updatePlayer(deltaTime)
		updateProjectiles(deltaTime)
		updateAliens(deltaTime)
		handleCollisions()
		updateWaves(deltaTime)
	elseif Io.isKeyJustPressed("Space") then
		resetGame()
	end

	drawWorld()
	drawHud()

	if State.phase == "gameover" then
		drawGameOver()
	end
end
