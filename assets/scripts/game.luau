local Event = require('@vectarine/event')
local Graphics = require('@vectarine/graphics')
local Io = require('@vectarine/io')
local Resources = require('@vectarine/resources')
local Vec = require('@vectarine/vec')

local V2 = Vec.V2

local t = 0
local spaceWasDown = false
local spacePressed: Event.Event<string> = Event.newEvent("custom")
spacePressed:on(function(data: string)
    Io.print("Received event with data: " .. data)
end)

Global.logo = Resources.loadImage("textures/logo.png")
Global.font = Resources.loadFont("fonts/arial.ttf")
Global.other_script = Resources.loadScript("scripts/other_script.lua")

Graphics.clear({r=1, g=0, b=0, a=1})

Global.fullscreen = false
if Global.fullscreen then
    local screen = Io.getScreenSize()
    Io.print("Screen: ", screen)
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
    Io.print("Loading ...")

    -- subId:unsubscribe()

    Global.fullscreen = false
    Global.frame_times = {}
end

function Update(time_delta)
    local bg_color: Graphics.Color = { r = 1, g = 1, b = 1, a = 1 }
    Graphics.clear(bg_color)

    local rect_color: Graphics.Color = { r = 0, g = 0, b = 1, a = 1 }

    local v = V2(1, 2)
    local w = V2(1, 2)
    local r = v * w
    Io.fprint("hello", r.x, r.y)

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
    for i, val in ipairs(Global.frame_times) do
        time_sum = time_sum + val
    end
    local avg_time = time_sum / #Global.frame_times

    table.insert(Global.frame_times, time_delta)
    if #Global.frame_times > 10 then
        table.remove(Global.frame_times, 1)
    end

    Io.fprint("AVG Frame time = " .. math.floor(10000 * avg_time) / 10 .. "ms")
    Io.fprint("AVG FPS = " .. math.floor(10 / avg_time) / 10)

    Graphics.drawCircle(m, 0.1, rect_color)

    local slow = false
    if slow then
        local slow_factor = 150
        local s = V2(0.1, 0.1)
        for i = 0, slow_factor do
            for j = 0, slow_factor do
                local ratio = slow_factor / 2
                rect_color.g = (i + t * 3) % 255 / 255
                rect_color.b = (j + t) % 255 / 255
                local p = V2(-1 + i / ratio, -1 + j / ratio)
                Graphics.drawRect(p, s, rect_color)
            end
        end
    end
    -- drawImage(Global.logo, -0.2, -0.2, 0.4, 0.4)
    local text = "HY$@llo World!  ..."
    local textSize = 0.1

    -- Technique for drawing a box around text.
    local mesurement = Graphics.measureText(text, Global.font, textSize)
    local toBaseline = mesurement.height - mesurement.bearingY
    Graphics.drawRect(V2(-0.5, 0.5 + toBaseline), V2(mesurement.width, mesurement.height), { r = 0, g = 1, b = 0, a = 0.5 })

    -- Center of the screen
    if Resources.isResourceReady(Global.logo) then
        Io.fprint("read: ", Resources.isResourceReady(Global.logo))
        Graphics.drawImagePart(Global.logo, V2(0.1, 0.1), V2(-0.1, 0.1), V2(-0.1, -0.1), V2(0.1, -0.1), V2(0, 0), V2(1, 1))
    end
    if Resources.isResourceReady(Global.font) then
        Graphics.drawText(text, Global.font, V2(-0.5, 0.5), textSize, { r = 0, g = 0, b = 0, a = 1 })
    end

    Graphics.drawArrow(V2(0, 0), V2(m.x, m.y):scale(0.5))
end
