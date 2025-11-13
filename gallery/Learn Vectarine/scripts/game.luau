local Coord = require("@vectarine/coord")
local Debug = require("@vectarine/debug")
local Event = require("@vectarine/event")
local Graphics = require("@vectarine/graphics")
local Io = require("@vectarine/io")
local Persist = require("@vectarine/persist")
local Resources = require("@vectarine/resources")
local Screen = require("@vectarine/screen")
local Vec = require("@vectarine/vec")
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

local logoResource = Resources.loadImage("textures/logo.png")
local _missing = Resources.loadImage("textures/this_file_is_missing.png")
local fontResource = Resources.loadFont("fonts/arial.ttf")

local circle = Persist.onReload({ r = 0.0, g = 0.0, b = 0.0, a = 1.0 }, "circle")

function drawTextCentered(text: string, position: Coord.ScreenPosition, textSize: number, color)
	local textMeasure = Graphics.measureText(text, fontResource, textSize)
	Graphics.drawText(text, fontResource, position - Coord.glVec(V2(textMeasure.width / 2, 0)), textSize, color)
end

local timer = 0.0
local screen0: Screen.Screen = Screen.newScreen("secret_screen", function()
	timer += 2.0 / 60.0
	Graphics.drawRect(Coord.gl(V2(-1, -1)), Coord.glVec(V2(2, 2)), { r = 0.2, g = 0.4, b = 1, a = 1 })

	Graphics.drawText(
		"You found the secret screen!",
		fontResource,
		Coord.gl(V2(-1, -1)) + Coord.pxVec(V2(20, -20)),
		0.16,
		{ r = 0.7, g = 0.7, b = 0.7, a = 1 }
	)

	local function getOffset(angle)
		return V2(math.cos(angle), math.sin(angle)):scale(100 + math.sin(timer / 3.4) * 20)
	end
	local p1 = Coord.CENTER + Coord.pxVec(getOffset(timer))
	local p2 = Coord.CENTER + Coord.pxVec(getOffset(timer + math.pi / 2))
	local p3 = Coord.CENTER + Coord.pxVec(getOffset(timer + math.pi))
	local p4 = Coord.CENTER + Coord.pxVec(getOffset(timer + 3 * math.pi / 2))
	Graphics.drawImagePart(logoResource, p1, p2, p3, p4, V2(0, 0), V2(1, 1))
end)

local screen1: Screen.Screen = Screen.newScreen("screen_1", function()
	Graphics.drawRect(Coord.gl(V2(-1, -1)), Coord.glVec(V2(2, 2)), { r = 0.2, g = 0, b = 0, a = 1 })
	if Resources.isResourceReady(logoResource) then
		Graphics.drawImage(logoResource, Coord.gl(V2(0, 0.5)) - Coord.pxVec(V2(100, 100)), Coord.pxVec(V2(200, 200)))
	end
	drawTextCentered("Welcome to Vectarine", Coord.CENTER, 0.16, { r = 1, g = 1, b = 1, a = 1 })
	Graphics.drawText(
		"Press SPACE to continue",
		fontResource,
		Coord.gl(V2(-1, -1)) + Coord.pxVec(V2(20, -20)),
		0.16,
		{ r = 0.7, g = 0.7, b = 0.7, a = 1 }
	)
end)
local screen2: Screen.Screen = Screen.newScreen("screen_2", function()
	Graphics.drawRect(Coord.gl(V2(-1, -1)), Coord.glVec(V2(2, 2)), { r = 0.1, g = 0.4, b = 0.5, a = 1 })
	drawTextCentered(
		"Open the console with Ctrl+1 or the tools menu",
		Coord.CENTER,
		textSize,
		{ r = 1, g = 1, b = 1, a = 1 }
	)
	Graphics.drawText(
		"Press BACK to go back",
		fontResource,
		Coord.gl(V2(-1, -1 + 0.16)) + Coord.pxVec(V2(20, -20)),
		0.16,
		{ r = 0.7, g = 0.7, b = 0.7, a = 1 }
	)
	Graphics.drawText(
		"Press SPACE to continue",
		fontResource,
		Coord.gl(V2(-1, -1)) + Coord.pxVec(V2(20, -20)),
		0.16,
		{ r = 0.7, g = 0.7, b = 0.7, a = 1 }
	)
end)

local screen3: Screen.Screen = Screen.newScreen("screen_3", function()
	Graphics.drawRect(Coord.gl(V2(-1, -1)), Coord.glVec(V2(2, 2)), { r = 0, g = 0.2, b = 0, a = 1 })
	drawTextCentered(
		"Open the resources with Ctrl+2 or the tools menu",
		Coord.CENTER,
		textSize,
		{ r = 1, g = 1, b = 1, a = 1 }
	)
	drawTextCentered(
		"Resources are files your project needs like scripts, images, sounds, etc.",
		Coord.CENTER + Coord.glVec(V2(0, -0.12)),
		0.08,
		{ r = 1, g = 1, b = 1, a = 1 }
	)
	drawTextCentered(
		"You can open a resource by clicking on the blue link",
		Coord.CENTER + Coord.glVec(V2(0, -0.12 * 2)),
		0.08,
		{ r = 1, g = 1, b = 1, a = 1 }
	)
	drawTextCentered(
		"You can check for missing resources, show debug info and more",
		Coord.CENTER + Coord.glVec(V2(0, -0.12 * 3)),
		0.08,
		{ r = 1, g = 1, b = 1, a = 1 }
	)

	Graphics.drawText(
		"Press SPACE to continue",
		fontResource,
		Coord.gl(V2(-1, -1)) + Coord.pxVec(V2(20, -20)),
		0.16,
		{ r = 0.7, g = 0.7, b = 0.7, a = 1 }
	)
end)

local screen4: Screen.Screen = Screen.newScreen("screen_4", function()
	Graphics.drawRect(Coord.gl(V2(-1, -1)), Coord.glVec(V2(2, 2)), { r = 0.7, g = 0.5, b = 0, a = 1 })
	drawTextCentered(
		"Open the watcher with Ctrl+3 or the tools menu",
		Coord.CENTER + Coord.glVec(V2(0, 0.2)),
		textSize,
		{ r = 1, g = 1, b = 1, a = 1 }
	)
	drawTextCentered(
		"Use the watcher to see variables and edit them",
		Coord.CENTER + Coord.glVec(V2(0, 0.2 - 0.12)),
		0.10,
		{ r = 1, g = 1, b = 1, a = 1 }
	)
	drawTextCentered(
		"Search for 'circle', 'mouse_position' or 'fullscreen'!",
		Coord.CENTER + Coord.glVec(V2(0, 0.2 - 0.12 * 2)),
		0.10,
		{ r = 1, g = 1, b = 1, a = 1 }
	)
	Graphics.drawCircle(Coord.CENTER + Coord.glVec(V2(0, -0.4)), 0.3, circle)
end)

local circlePosX = 0.0
local circleDirection = 1.0
local screen5: Screen.Screen = Screen.newScreen("screen_5", function()
	Graphics.drawRect(Coord.gl(V2(-1, -1)), Coord.glVec(V2(2, 2)), { r = 0.2, g = 0.5, b = 0.3, a = 1 })
	drawTextCentered(
		"When you edit scripts, the project updates automatically!",
		Coord.CENTER + Coord.glVec(V2(0, 0.2)),
		0.10,
		{ r = 1, g = 1, b = 1, a = 1 }
	)
	drawTextCentered(
		"Open game.luau from the resource tab and edit this text!",
		Coord.CENTER + Coord.glVec(V2(0, 0.2 - 0.12)),
		0.10,
		{ r = 1, g = 1, b = 1, a = 1 }
	)
	drawTextCentered(
		"You can use Ctrl+F to find where this text is in the script",
		Coord.CENTER + Coord.glVec(V2(0, 0.2 - 0.12 * 2)),
		0.10,
		{ r = 1, g = 1, b = 1, a = 1 }
	)

	drawTextCentered(
		"Try to make the circle below move!",
		Coord.CENTER + Coord.glVec(V2(0, 0.2 - 0.12 * 3)),
		0.10,
		{ r = 1, g = 1, b = 1, a = 1 }
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
	Graphics.drawRect(Coord.gl(V2(-1, -1)), Coord.glVec(V2(2, 2)), { r = 0.2, g = 0.5, b = 0.3, a = 1 })
	drawTextCentered(
		"When you edit any resource, it updates!",
		Coord.CENTER + Coord.glVec(V2(0, 0.2)),
		0.10,
		{ r = 1, g = 1, b = 1, a = 1 }
	)
	drawTextCentered(
		"Try editing this image (logo.png) and saving!",
		Coord.CENTER + Coord.glVec(V2(0, 0.2 - 0.12)),
		0.10,
		{ r = 1, g = 1, b = 1, a = 1 }
	)
	if Resources.isResourceReady(logoResource) then
		Graphics.drawImage(logoResource, Coord.gl(V2(0, -0.5)) - Coord.pxVec(V2(100, 100)), Coord.pxVec(V2(200, 200)))
	end
end)

local screen7: Screen.Screen = Screen.newScreen("screen_7", function()
	Graphics.drawRect(Coord.gl(V2(-1, -1)), Coord.glVec(V2(2, 2)), { r = 0.2, g = 0.5, b = 0.3, a = 1 })
	drawTextCentered(
		"You are ready to make games!",
		Coord.CENTER + Coord.glVec(V2(0, 0.2)),
		0.10,
		{ r = 1, g = 1, b = 1, a = 1 }
	)
	drawTextCentered(
		"Read the vectarine guide for more information.",
		Coord.CENTER + Coord.glVec(V2(0, 0.2 - 0.12 * 4)),
		0.10,
		{ r = 1, g = 1, b = 1, a = 1 }
	)
	Graphics.drawText(
		"Press BACK to go back",
		fontResource,
		Coord.gl(V2(-1, -1 + 0.16)) + Coord.pxVec(V2(20, -20)),
		0.16,
		{ r = 0.7, g = 0.7, b = 0.7, a = 1 }
	)
	Graphics.drawText(
		"Press SPACE to continue",
		fontResource,
		Coord.gl(V2(-1, -1)) + Coord.pxVec(V2(20, -20)),
		0.16,
		{ r = 0.7, g = 0.7, b = 0.7, a = 1 }
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

local spaceJustPressed = false
local backJustPressed = false
function Update(time_delta: number)
	fprint("You can also use fprint (frame print) to show messages that change every frame!")
	local bg_color: Graphics.Color = { r = 0, g = 0, b = 0.2, a = 1 }
	Graphics.clear(bg_color)

	if fullscreen.is_on then
		local screen = Io.getScreenSize()
		Io.setFullscreen("desktop")
		Io.setWindowSize(screen.x, screen.y)
	else
		Io.setFullscreen(false)
		Io.setWindowSize(800, 600)
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

	if Io.isKeyDown("Space") then
		textSize = 0.1
		if not spaceJustPressed then
			currentScreenIndex.index = currentScreenIndex.index + 1
			if currentScreenIndex.index > #screens then
				currentScreenIndex.index = 2
			end
		end
		spaceJustPressed = true
	else
		spaceJustPressed = false
	end

	if Io.isKeyDown("Backspace") then
		textSize = 0.1
		if not backJustPressed then
			currentScreenIndex.index = currentScreenIndex.index - 1
			if currentScreenIndex.index < 1 then
				currentScreenIndex.index = 1
			end
		end
		backJustPressed = true
	else
		backJustPressed = false
	end

	Screen.drawCurrentScreen()
end
