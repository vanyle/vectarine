local Coord = require('@vectarine/coord')
local Debug = require('@vectarine/debug')
local Event = require('@vectarine/event')
local Graphics = require('@vectarine/graphics')
local Io = require('@vectarine/io')
local Resources = require('@vectarine/resources')
local Screen = require('@vectarine/screen')
local Vec = require('@vectarine/vec')
local print = Debug.print
local fprint = Debug.fprint

-- print("Other script: ", helper)

local V2 = Vec.V2

local spacePressed: Event.Event<string> = Event.newEvent("custom")
spacePressed:on(function(data: string)
    print("Received event with data: " .. data)
end)

local KEY_DOWN = Event.getKeyDownEvent()
KEY_DOWN:on(function(key: string)
    print("Key down: " .. key)
end)

local CONSOLE_COMMAND = Event.getConsoleCommandEvent()
CONSOLE_COMMAND:on(function(command: string)
    print("Console command received: " .. command)
end)

local RESOURCE_LOADED = Event.getResourceLoadedEvent()
RESOURCE_LOADED:on(function(id: Resources.ResourceId)
    -- print("Resource loaded: ", id)
end)

local logoResource = Resources.loadImage("textures/logo.png")
local fontResource = Resources.loadFont("fonts/arial.ttf")
local helperResource, helper = Resources.loadScript("scripts/helper.luau", require('helper.luau'))

Graphics.clear({r=1, g=0, b=0, a=1})

local menuScreen = Screen.newScreen("Menu", function()
    Graphics.drawRect(Coord.gl(-1, -1), Coord.glDelta(2,2), {r=0.2, g=0, b=0, a=1})
    Graphics.drawText("Main Menu", fontResource, Coord.gl(-0.5, 0), 0.32, {r=1, g=1, b=1, a=1})
    Graphics.drawText("Press SPACE to start", fontResource, Coord.gl(-0.5, -0.3), 0.16, {r=0.7, g=0.7, b=0.7, a=1})
end)
local gameScreen = Screen.newScreen("Game", function()
    Graphics.drawRect(Coord.gl(-1, -1), Coord.glDelta(2,2), {r=0, g=0.2, b=0, a=1})
    Graphics.drawText("Playing G", fontResource, Coord.gl(-0.5, 0), 0.32, {r=1, g=1, b=1, a=1})
    Graphics.drawText("Press ESC for menu", fontResource, Coord.gl(-0.5, -0.3), 0.16, {r=0.7, g=0.7, b=0.7, a=1})

    if Resources.isResourceReady(logoResource) then
        fprint("read: ", Resources.isResourceReady(logoResource))
        Graphics.drawImage(logoResource, Coord.gl(0, 0.5) - Coord.pxDelta(100, 100), Coord.pxDelta(200, 200))
    end
end)
Screen.setCurrentScreen(menuScreen)

if _G.fullscreen == nil then
    _G.fullscreen = false
end
_G.frame_times = {}

if _G.fullscreen then
    local screen = Io.getScreenSize()
    print("Screen: ", screen)
    Io.setFullscreen(true)
    Io.setWindowSize(screen.x, screen.y)

    local v = V2(1, 2)
    local w = V2(1, 2)
    local _r = v:cmul(w)
else
    Io.setFullscreen(false)
    Io.setWindowSize(800, 600)
end

function Load()
    print("Loading ...")

    -- subId:unsubscribe()

    _G.fullscreen = false
    _G.frame_times = {}
end

function Update(time_delta: number)

    local bg_color: Graphics.Color = { r = 0, g = 0, b = 0.2, a = 1 }
    Graphics.clear(bg_color)

    if Resources.isResourceReady(helperResource) then
        fprint(helper.add_things(1, 2))
    end

    local screen = Screen.getCurrentScreen()
    if screen ~= nil then
    if screen:name() == "Menu" and Io.isKeyDown("Space") then
        -- Transition to game screen with fade effect
        Screen.setCurrentScreen(gameScreen, {
            duration = 1.0,
            transition_style = "slide_up"
        })
    elseif screen:name() == "Game" and Io.isKeyDown("Escape") then
        -- Transition back to menu with slide effect
        Screen.setCurrentScreen(menuScreen, {
            duration = 1.0,
            transition_style = "toon"
        })
    end
end

    -- Draw the current screen (handles transitions automatically)
    Screen.drawCurrentScreen()
end

