local Debug = require('@vectarine/debug')
local Event = require('@vectarine/event')
local Graphics = require('@vectarine/graphics')
local Io = require('@vectarine/io')
local Resources = require('@vectarine/resources')
local Vec = require('@vectarine/vec')
local print = Debug.print
local fprint = Debug.fprint

-- print("Other script: ", helper)

local V2 = Vec.V2

local t = 0
local spaceWasDown = false
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

_G.fullscreen = false
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

    local bg_color: Graphics.Color = { r = 1, g = 1, b = 1, a = 1 }
    Graphics.clear(bg_color)

    fprint(150*150)

    local slow = false
    if slow then
        local slow_factor = 150
        local s = V2(0.1, 0.1)
        for i = 0, slow_factor do
            for j = 0, slow_factor do
                local ratio = slow_factor / 2
                local n1 = math.noise(i / slow_factor * 10, j / slow_factor * 10, t / 100)
                local n2 = math.noise(j / slow_factor * 10, i / slow_factor * 10, t / 100)
                local n3 = math.noise(i / slow_factor * 10, t / 100, j / slow_factor * 10)
                local p = V2(-1 + i / ratio, -1 + j / ratio)
                local noise_color: Graphics.Color = { r = n1, g = n2, b = n3, a = 1 }
                Graphics.drawRect(p, s, noise_color)
            end
        end
    end

    local rect_color = {r = 0, g = 0, b = 1, a = 1}
    Graphics.drawRect(V2(0.7, -1), V2(0.3, 0.3), rect_color)

    if Resources.isResourceReady(helperResource) then
        fprint("Adding: ",helper.add_things(3, 2)) -- prints 1+2+3 = 6
    end
    
    rect_color = { r = 0, g = 0, b = 1, a = 1 }
    
    local v = V2(1, 2)
    local w = V2(1, 2)
    local r = v * w
    fprint("hello", r.x, r.y)

    if Io.isKeyDown("space") then
        rect_color = { r = 1, g = 0, b = 1, a = 1 }
    end

    if Io.isKeyDown("space") then
        if not spaceWasDown then
            spacePressed:dispatch("Hello from space!")
        end
        spaceWasDown = true
    else
        spaceWasDown = false
    end

    t = t + 1
    local m: Vec.Vec2 = Io.getMouse()
    -- fprint("Hello: " .. x .. "," .. y)

    local time_sum = 0
    for i, val in ipairs(_G.frame_times) do
        time_sum = time_sum + val
    end
    local avg_time = time_sum / #_G.frame_times

    table.insert(_G.frame_times, time_delta)
    if #_G.frame_times > 10 then
        table.remove(_G.frame_times, 1)
    end

    fprint("AVG Frame time = " .. math.floor(10000 * avg_time) / 10 .. "ms")
    fprint("AVG FPS = " .. math.floor(10 / avg_time) / 10)

    Graphics.drawCircle(m, 0.1, rect_color)

    -- drawImage(_G.logo, -0.2, -0.2, 0.4, 0.4)
    local text = "HY$@llo World!  ..."
    local textSize = 0.1

    -- Technique for drawing a box around text.
    local mesurement = Graphics.measureText(text, fontResource, textSize)
    local toBaseline = mesurement.height - mesurement.bearingY
    Graphics.drawRect(V2(-0.5, 0.5 + toBaseline), V2(mesurement.width, mesurement.height), { r = 0, g = 1, b = 0, a = 0.5 })

    -- Center of the screen
    if Resources.isResourceReady(logoResource) then
        fprint("read: ", Resources.isResourceReady(logoResource))
        Graphics.drawImage(logoResource, V2(-0.1, -0.1), V2(0.2, 0.2))
    end
    if Resources.isResourceReady(fontResource) then
        Graphics.drawText(text, fontResource, V2(-0.5, 0.5), textSize, { r = 0, g = 0, b = 0, a = 1 })
    end

    Graphics.drawArrow(V2(0, 0), V2(m.x, m.y):scale(0.5))
end

