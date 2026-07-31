const coord = require("@vectarine/coord")
const debug = require("@vectarine/debug")
const event = require("@vectarine/event")
const graphics = require("@vectarine/graphics")
const io = require("@vectarine/io")
const loader = require("@vectarine/loader")
const persist = require("@vectarine/persist")
const ui = require("@vectarine/ui")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

const print = debug.print
const fprint = debug.fprint

const V2 = vec.V2

print("You can display messages in the console using the print function.")
print("You can enter commands in the console. Try typing 'zoom' or 'shrink' and pressing Enter.")

local textSize = 0.1

local CONSOLE_COMMAND = event.getConsoleCommandEvent()
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

const logoResource = loader.loadImage("textures/logo.png")
const _missing = loader.loadImage("textures/this_file_is_missing.png")
const fontResource = loader.loadFont("fonts/arial.ttf")

local circle = persist.onReload(vec4.createColor(0, 0, 0, 1), "circle_color")

io.setResizeable(true) -- The base window size can be small for some people

local time = 0.0
function drawTextCentered(
	text: string,
	position: coord.ScreenPosition,
	textSize: number | coord.ScreenVec,
	color: vec4.Vec4,
	dance: boolean?
)
	local textMeasure = fontResource:measureText(text, textSize)
	if dance then
		local offset = 0
		time += 1.0 / 60.0
		for i = 1, text:len() do
			const char = text:sub(i, i)
			const offsetPos = coord.glVec(vec.V2(offset, math.sin(time * 1.5 + i / 3.0) * 0.04))
			offset += fontResource:measureText(char, textSize).width * 1.1
			fontResource:drawText(
				char,
				position + offsetPos - coord.glVec(V2(textMeasure.width / 2, 0)),
				textSize,
				color
			)
		end
	else
		fontResource:drawText(text, position - coord.glVec(V2(textMeasure.width / 2, 0)), textSize, color)
	end
end

local timer = 0.0

local secret_screen: ui.Widget<any> = ui.widget(V2(2, 2), function()
	timer += 2.0 / 60.0
	graphics.drawRect(coord.gl(V2(-1, -1)), coord.glVec(V2(2, 2)), vec4.createColor(0.2, 0.4, 1, 1))

	fontResource:drawText(
		"You found the secret screen!",
		coord.gl(V2(-1, -1)) + coord.pxVec(V2(20, -20)),
		0.16,
		vec4.createColor(0.7, 0.7, 0.7, 1)
	)

	local function getOffset(angle)
		return vec.fromAngle(angle, 100 + math.sin(timer / 3.4) * 20)
	end
	local p1 = coord.CENTER + coord.pxVec(getOffset(timer))
	local p2 = coord.CENTER + coord.pxVec(getOffset(timer + math.pi / 2))
	local p3 = coord.CENTER + coord.pxVec(getOffset(timer + math.pi))
	local p4 = coord.CENTER + coord.pxVec(getOffset(timer + 3 * math.pi / 2))
	logoResource:drawPart(p1, p2, p3, p4, V2(0, 0), V2(1, 1))
end)

local screen1: ui.Widget<any> = ui.widget(V2(2, 2), function()
	graphics.drawRect(coord.gl(V2(-1, -1)), coord.glVec(V2(2, 2)), vec4.createColor(0.2, 0, 0, 1))
	if logoResource:isReady() then
		logoResource:draw(coord.gl(V2(0, 0.5)) - coord.pxVec(V2(100, 100)), coord.pxVec(V2(200, 200)))
	end
	drawTextCentered("Welcome to Vectarine", coord.CENTER, textSize, vec4.WHITE, true)
	fontResource:drawText(
		"Press SPACE to continue",
		coord.gl(V2(-1, -1)) + coord.pxVec(V2(20, -20)),
		0.16,
		vec4.createColor(0.7, 0.7, 0.7, 1)
	)
end)

local screen2: ui.Widget<any> = ui.widget(V2(2, 2), function()
	graphics.drawRect(coord.gl(V2(-1, -1)), coord.glVec(V2(2, 2)), vec4.createColor(0.1, 0.4, 0.5, 1))
	drawTextCentered("Open the console with Ctrl+1 or the tools menu", coord.CENTER, textSize, vec4.WHITE)
	fontResource:drawText(
		"Press BACK to go back",
		coord.gl(V2(-1, -1 + 0.16)) + coord.pxVec(V2(20, -20)),
		0.16,
		vec4.createColor(0.7, 0.7, 0.7, 1)
	)
	fontResource:drawText(
		"Press SPACE to continue",
		coord.gl(V2(-1, -1)) + coord.pxVec(V2(20, -20)),
		0.16,
		vec4.createColor(0.7, 0.7, 0.7, 1)
	)
end)

local screen3: ui.Widget<any> = ui.widget(V2(2, 2), function()
	graphics.drawRect(coord.gl(V2(-1, -1)), coord.glVec(V2(2, 2)), vec4.createColor(0, 0.2, 0, 1))
	drawTextCentered("Open the resources with Ctrl+2 or the tools menu", coord.CENTER, textSize, vec4.WHITE)
	drawTextCentered(
		"Resources are files your project needs like scripts, images, sounds, etc.",
		coord.CENTER + coord.glVec(V2(0, -0.12)),
		0.08,
		vec4.WHITE
	)
	drawTextCentered(
		"You can open a resource by clicking on the blue link",
		coord.CENTER + coord.glVec(V2(0, -0.12 * 2)),
		0.08,
		vec4.WHITE
	)
	drawTextCentered(
		"You can check for missing resources, show debug info and more",
		coord.CENTER + coord.glVec(V2(0, -0.12 * 3)),
		0.08,
		vec4.WHITE
	)

	fontResource:drawText(
		"Press SPACE to continue",
		coord.gl(V2(-1, -1)) + coord.pxVec(V2(20, -20)),
		0.16,
		vec4.createColor(0.7, 0.7, 0.7, 1)
	)
end)

local screen4: ui.Widget<any> = ui.widget(V2(2, 2), function()
	graphics.drawRect(coord.gl(V2(-1, -1)), coord.glVec(V2(2, 2)), vec4.createColor(0.7, 0.5, 0, 1))
	drawTextCentered(
		"Open the watcher with Ctrl+3 or the tools menu",
		coord.CENTER + coord.glVec(V2(0, 0.2)),
		textSize,
		vec4.WHITE
	)
	drawTextCentered(
		"Use the watcher to see variables and edit them",
		coord.CENTER + coord.glVec(V2(0, 0.2 - 0.12)),
		0.10,
		vec4.WHITE
	)
	drawTextCentered(
		"Search for 'circle', 'mouse_position' or 'fullscreen'!",
		coord.CENTER + coord.glVec(V2(0, 0.2 - 0.12 * 2)),
		0.10,
		vec4.WHITE
	)
	graphics.drawCircle(coord.CENTER + coord.glVec(V2(0, -0.4)), 0.3, circle)
end)

local circlePosX = 0.0
local circleDirection = 1.0
local screen5: ui.Widget<any> = ui.widget(V2(2, 2), function()
	graphics.drawRect(coord.gl(V2(-1, -1)), coord.glVec(V2(2, 2)), vec4.createColor(0.2, 0.5, 0.3, 1))
	drawTextCentered(
		"When you edit scripts, the project updates automatically!",
		coord.CENTER + coord.glVec(V2(0, 0.2)),
		0.10,
		vec4.WHITE
	)
	drawTextCentered(
		"Open game.luau from the resource tab and edit this text!",
		coord.CENTER + coord.glVec(V2(0, 0.2 - 0.12)),
		0.10,
		vec4.WHITE
	)
	drawTextCentered(
		"You can use Ctrl+F to find where this text is in the script",
		coord.CENTER + coord.glVec(V2(0, 0.2 - 0.12 * 2)),
		0.10,
		vec4.WHITE
	)

	drawTextCentered(
		"Try to make the circle below move!",
		coord.CENTER + coord.glVec(V2(0, 0.2 - 0.12 * 3)),
		0.10,
		vec4.WHITE
	)
	local circleSpeed = 0.0 -- Change the speed to make it move!

	local scale = 300
	graphics.drawCircle(coord.CENTER + coord.glVec(V2(circlePosX / scale, -0.6)), 0.3, circle)
	circlePosX += circleDirection * circleSpeed
	if circlePosX > scale then
		circleDirection = -1.0
	elseif circlePosX < -scale then
		circleDirection = 1.0
	end
end)

local screen6: ui.Widget<any> = ui.widget(V2(2, 2), function()
	graphics.drawRect(coord.gl(V2(-1, -1)), coord.glVec(V2(2, 2)), vec4.createColor(0.2, 0.5, 0.3, 1))
	drawTextCentered(
		"When you edit any resource, it updates!",
		coord.CENTER + coord.glVec(V2(0, 0.2)),
		0.10,
		vec4.WHITE
	)
	drawTextCentered(
		"Try editing this image (logo.png) and saving!",
		coord.CENTER + coord.glVec(V2(0, 0.2 - 0.12)),
		0.10,
		vec4.WHITE
	)
	if logoResource:isReady() then
		logoResource:draw(coord.gl(V2(0, -0.5)) - coord.pxVec(V2(100, 100)), coord.pxVec(V2(200, 200)))
	end
end)

local screen7: ui.Widget<any> = ui.widget(V2(2, 2), function()
	graphics.drawRect(coord.gl(V2(-1, -1)), coord.glVec(V2(2, 2)), vec4.createColor(0.2, 0.5, 0.3, 1))
	drawTextCentered("You are ready to make games!", coord.CENTER + coord.glVec(V2(0, 0.2)), 0.10, vec4.WHITE)
	drawTextCentered(
		"Read the vectarine guide for more information.",
		coord.CENTER + coord.glVec(V2(0, 0.2 - 0.12 * 4)),
		0.10,
		vec4.WHITE
	)
	fontResource:drawText(
		"Press BACK to go back",
		coord.gl(V2(-1, -1 + 0.16)) + coord.pxVec(V2(20, -20)),
		0.16,
		vec4.createColor(0.7, 0.7, 0.7, 1)
	)
	fontResource:drawText(
		"Press SPACE to continue",
		coord.gl(V2(-1, -1)) + coord.pxVec(V2(20, -20)),
		0.16,
		vec4.createColor(0.7, 0.7, 0.7, 1)
	)
end)

local screens: { [string]: ui.Widget<any> } = {
	["0"] = secret_screen,
	["1"] = screen1,
	["2"] = screen2,
	["3"] = screen3,
	["4"] = screen4,
	["5"] = screen5,
	["6"] = screen6,
	["7"] = screen7,
}
local screenCount = 8

local currentScreenIndex = persist.onReload({ index = 1 }, "current_screen_index")

local widget: ui.TabWidget<any> = ui.tabs(screens)
widget:setActiveTab(tostring(currentScreenIndex.index))

local fullscreen = persist.onReload({ is_on = false }, "fullscreen")

local function Once()
	_G.onceCalled = true
	io.centerWindow()
end
if not _G.onceCalled then
	Once()
end

function Update(time_delta: number)
	fprint("You can also use fprint (frame print) to show messages that change every frame!")
	local bg_color: vec4.Vec4 = vec4.createColor(0, 0, 0.2, 1)
	graphics.clear(bg_color)

	if fullscreen.is_on then
		local screen = io.getScreenSize()
		io.setFullscreen("desktop")
		io.setWindowSize(screen.x, screen.y)
	else
		-- Io.setWindowSize(800, 600)
		io.setFullscreen(false)
	end

	_G.mouse_position = io.getMouse()

	local current_tab = widget:getActiveTab()
	if current_tab ~= tostring(currentScreenIndex.index) then
		widget:setActiveTab(tostring(currentScreenIndex.index), {
			duration = 1.0,
			animationType = "toon",
		})
	end

	if io.isKeyJustPressed("Space") then
		textSize = 0.1
		currentScreenIndex.index = currentScreenIndex.index + 1
		if currentScreenIndex.index > screenCount - 1 then
			currentScreenIndex.index = 1
		end
	end

	if io.isKeyJustPressed("Backspace") then
		textSize = 0.1
		currentScreenIndex.index = currentScreenIndex.index - 1
		if currentScreenIndex.index < 0 then
			currentScreenIndex.index = 0
		end
	end

	widget:draw()
end
