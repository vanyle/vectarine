local Coord = require("@vectarine/coord")
local Debug = require("@vectarine/debug")
local Event = require("@vectarine/event")
local Graphics = require("@vectarine/graphics")
local Io = require("@vectarine/io")
local Loader = require("@vectarine/loader")
local Persist = require("@vectarine/persist")
local Screen = require("@vectarine/screen")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")
local print = Debug.print
local fprint = Debug.fprint

local V2 = Vec.V2

print("You can display messages in the console using the print function.")
print("You can enter commands in the console. Try typing 'zoom' and pressing Enter.")

local textSize = 0.1

local CONSOLE_COMMAND = Event.getConsoleCommandEvent()
CONSOLE_COMMAND:on(function(command: string)
	print("Console command received: " .. command)
	if command == "zoom" then
		textSize = textSize * 1.2
		print("Increased text size to " .. tostring(textSize))
	elseif command == "shrink" then
		textSize = textSize / 1.2
		print("Decreased text size to " .. tostring(textSize))
	end
end)

local logoResource = Loader.loadImage("textures/logo.png")
local _missing = Loader.loadImage("textures/this_file_is_missing.png")
local fontResource = Loader.loadFont("fonts/arial.ttf")

local circle = Persist.onReload(Vec4.createColor(0, 0, 0, 1), "circle")

Io.setResizeable(true) -- The base window size can be small for some people

function drawTextCentered(
	text: string,
	position: Coord.ScreenPosition,
	textSize: number | Coord.ScreenVec,
	color: Vec4.Vec4
)
	local textMeasure = fontResource:measureText(text, textSize)
	fontResource:drawText(text, position - Coord.glVec(V2(textMeasure.width / 2, 0)), textSize, color)
end

local timer = 0.0
local screen0: Screen.Screen = Screen.newScreen("secret_screen", function()
	timer += 2.0 / 60.0
	Graphics.drawRect(Coord.gl(V2(-1, -1)), Coord.glVec(V2(2, 2)), Vec4.createColor(0.2, 0.4, 1, 1))

	fontResource:drawText(
		"You found the secret screen!",
		Coord.gl(V2(-1, -1)) + Coord.pxVec(V2(20, -20)),
		0.16,
		Vec4.createColor(0.7, 0.7, 0.7, 1)
	)

	local function getOffset(angle)
		return Vec.fromAngle(angle, 100 + math.sin(timer / 3.4) * 20)
	end
	local p1 = Coord.CENTER + Coord.pxVec(getOffset(timer))
	local p2 = Coord.CENTER + Coord.pxVec(getOffset(timer + math.pi / 2))
	local p3 = Coord.CENTER + Coord.pxVec(getOffset(timer + math.pi))
	local p4 = Coord.CENTER + Coord.pxVec(getOffset(timer + 3 * math.pi / 2))
	logoResource:drawPart(p1, p2, p3, p4, V2(0, 0), V2(1, 1))
end)

local screen1: Screen.Screen = Screen.newScreen("screen_1", function()
	Graphics.drawRect(Coord.gl(V2(-1, -1)), Coord.glVec(V2(2, 2)), Vec4.createColor(0.2, 0, 0, 1))
	if logoResource:isReady() then
		logoResource:draw(Coord.gl(V2(0, 0.5)) - Coord.pxVec(V2(100, 100)), Coord.pxVec(V2(200, 200)))
	end
	drawTextCentered("Welcome to Vectarine", Coord.CENTER, 0.16, Vec4.WHITE)
	fontResource:drawText(
		"Press SPACE to continue",
		Coord.gl(V2(-1, -1)) + Coord.pxVec(V2(20, -20)),
		0.16,
		Vec4.createColor(0.7, 0.7, 0.7, 1)
	)
end)
local screen2: Screen.Screen = Screen.newScreen("screen_2", function()
	Graphics.drawRect(Coord.gl(V2(-1, -1)), Coord.glVec(V2(2, 2)), Vec4.createColor(0.1, 0.4, 0.5, 1))
	drawTextCentered("Open the console with Ctrl+1 or the tools menu", Coord.CENTER, textSize, Vec4.WHITE)
	fontResource:drawText(
		"Press BACK to go back",
		Coord.gl(V2(-1, -1 + 0.16)) + Coord.pxVec(V2(20, -20)),
		0.16,
		Vec4.createColor(0.7, 0.7, 0.7, 1)
	)
	fontResource:drawText(
		"Press SPACE to continue",
		Coord.gl(V2(-1, -1)) + Coord.pxVec(V2(20, -20)),
		0.16,
		Vec4.createColor(0.7, 0.7, 0.7, 1)
	)
end)

local screen3: Screen.Screen = Screen.newScreen("screen_3", function()
	Graphics.drawRect(Coord.gl(V2(-1, -1)), Coord.glVec(V2(2, 2)), Vec4.createColor(0, 0.2, 0, 1))
	drawTextCentered("Open the resources with Ctrl+2 or the tools menu", Coord.CENTER, textSize, Vec4.WHITE)
	drawTextCentered(
		"Resources are files your project needs like scripts, images, sounds, etc.",
		Coord.CENTER + Coord.glVec(V2(0, -0.12)),
		0.08,
		Vec4.WHITE
	)
	drawTextCentered(
		"You can open a resource by clicking on the blue link",
		Coord.CENTER + Coord.glVec(V2(0, -0.12 * 2)),
		0.08,
		Vec4.WHITE
	)
	drawTextCentered(
		"You can check for missing resources, show debug info and more",
		Coord.CENTER + Coord.glVec(V2(0, -0.12 * 3)),
		0.08,
		Vec4.WHITE
	)

	fontResource:drawText(
		"Press SPACE to continue",
		Coord.gl(V2(-1, -1)) + Coord.pxVec(V2(20, -20)),
		0.16,
		Vec4.createColor(0.7, 0.7, 0.7, 1)
	)
end)

local screen4: Screen.Screen = Screen.newScreen("screen_4", function()
	Graphics.drawRect(Coord.gl(V2(-1, -1)), Coord.glVec(V2(2, 2)), Vec4.createColor(0.7, 0.5, 0, 1))
	drawTextCentered(
		"Open the watcher with Ctrl+3 or the tools menu",
		Coord.CENTER + Coord.glVec(V2(0, 0.2)),
		textSize,
		Vec4.WHITE
	)
	drawTextCentered(
		"Use the watcher to see variables and edit them",
		Coord.CENTER + Coord.glVec(V2(0, 0.2 - 0.12)),
		0.10,
		Vec4.WHITE
	)
	drawTextCentered(
		"Search for 'circle', 'mouse_position' or 'fullscreen'!",
		Coord.CENTER + Coord.glVec(V2(0, 0.2 - 0.12 * 2)),
		0.10,
		Vec4.WHITE
	)
	Graphics.drawCircle(Coord.CENTER + Coord.glVec(V2(0, -0.4)), 0.3, circle)
end)

local circlePosX = 0.0
local circleDirection = 1.0
local screen5: Screen.Screen = Screen.newScreen("screen_5", function()
	Graphics.drawRect(Coord.gl(V2(-1, -1)), Coord.glVec(V2(2, 2)), Vec4.createColor(0.2, 0.5, 0.3, 1))
	drawTextCentered(
		"When you edit scripts, the project updates automatically!",
		Coord.CENTER + Coord.glVec(V2(0, 0.2)),
		0.10,
		Vec4.WHITE
	)
	drawTextCentered(
		"Open game.luau from the resource tab and edit this text!",
		Coord.CENTER + Coord.glVec(V2(0, 0.2 - 0.12)),
		0.10,
		Vec4.WHITE
	)
	drawTextCentered(
		"You can use Ctrl+F to find where this text is in the script",
		Coord.CENTER + Coord.glVec(V2(0, 0.2 - 0.12 * 2)),
		0.10,
		Vec4.WHITE
	)

	drawTextCentered(
		"Try to make the circle below move!",
		Coord.CENTER + Coord.glVec(V2(0, 0.2 - 0.12 * 3)),
		0.10,
		Vec4.WHITE
	)
	local circleSpeed = 0.0 -- Change the speed to make it move!

	local scale = 300
	Graphics.drawCircle(Coord.CENTER + Coord.glVec(V2(circlePosX / scale, -0.6)), 0.3, circle)
	circlePosX += circleDirection * circleSpeed
	if circlePosX > scale then
		circleDirection = -1.0
	elseif circlePosX < -scale then
		circleDirection = 1.0
	end
end)

local screen6: Screen.Screen = Screen.newScreen("screen_6", function()
	Graphics.drawRect(Coord.gl(V2(-1, -1)), Coord.glVec(V2(2, 2)), Vec4.createColor(0.2, 0.5, 0.3, 1))
	drawTextCentered(
		"When you edit any resource, it updates!",
		Coord.CENTER + Coord.glVec(V2(0, 0.2)),
		0.10,
		Vec4.WHITE
	)
	drawTextCentered(
		"Try editing this image (logo.png) and saving!",
		Coord.CENTER + Coord.glVec(V2(0, 0.2 - 0.12)),
		0.10,
		Vec4.WHITE
	)
	if logoResource:isReady() then
		logoResource:draw(Coord.gl(V2(0, -0.5)) - Coord.pxVec(V2(100, 100)), Coord.pxVec(V2(200, 200)))
	end
end)

local screen7: Screen.Screen = Screen.newScreen("screen_7", function()
	Graphics.drawRect(Coord.gl(V2(-1, -1)), Coord.glVec(V2(2, 2)), Vec4.createColor(0.2, 0.5, 0.3, 1))
	drawTextCentered("You are ready to make games!", Coord.CENTER + Coord.glVec(V2(0, 0.2)), 0.10, Vec4.WHITE)
	drawTextCentered(
		"Read the vectarine guide for more information.",
		Coord.CENTER + Coord.glVec(V2(0, 0.2 - 0.12 * 4)),
		0.10,
		Vec4.WHITE
	)
	fontResource:drawText(
		"Press BACK to go back",
		Coord.gl(V2(-1, -1 + 0.16)) + Coord.pxVec(V2(20, -20)),
		0.16,
		Vec4.createColor(0.7, 0.7, 0.7, 1)
	)
	fontResource:drawText(
		"Press SPACE to continue",
		Coord.gl(V2(-1, -1)) + Coord.pxVec(V2(20, -20)),
		0.16,
		Vec4.createColor(0.7, 0.7, 0.7, 1)
	)
end)

Screen.setCurrentScreen(screen1)

local screens: { Screen.Screen | nil } = { screen0, screen1, screen2, screen3, screen4, screen5, screen6, screen7 }

local currentScreenIndex = Persist.onReload({ index = 2 }, "current_screen_index")
local fullscreen = Persist.onReload({ is_on = false }, "fullscreen")
local frame_times = Persist.onReload({}, "frame_times")
local mouse_position = Persist.onReload({ x = 0.0, y = 0.0 }, "mouse_position")

function Load()
	print("Loading ...")
	fullscreen.is_on = false
	table.clear(frame_times)
end

function Update(time_delta: number)
	fprint("You can also use fprint (frame print) to show messages that change every frame!")
	local bg_color: Vec4.Vec4 = Vec4.createColor(0, 0, 0.2, 1)
	Graphics.clear(bg_color)

	if fullscreen.is_on then
		local screen = Io.getScreenSize()
		Io.setFullscreen("desktop")
		Io.setWindowSize(screen.x, screen.y)
	else
		-- Io.setWindowSize(800, 600)
		Io.setFullscreen(false)
	end

	mouse_position.x = Io.getMouse().x
	mouse_position.y = Io.getMouse().y

	local screen: Screen.Screen? = Screen.getCurrentScreen()
	if screen == nil then
		return
	end

	local target_screen = screens[currentScreenIndex.index]
	if target_screen ~= nil and screen ~= target_screen then
		Screen.setCurrentScreen(target_screen, {
			duration = 1.0,
			transition_style = "toon",
		})
	end

	if Io.isKeyJustPressed("Space") then
		textSize = 0.1
		currentScreenIndex.index = currentScreenIndex.index + 1
		if currentScreenIndex.index > #screens then
			currentScreenIndex.index = 2
		end
	end

	if Io.isKeyJustPressed("Backspace") then
		textSize = 0.1
		currentScreenIndex.index = currentScreenIndex.index - 1
		if currentScreenIndex.index < 1 then
			currentScreenIndex.index = 1
		end
	end

	Screen.drawCurrentScreen()
end
