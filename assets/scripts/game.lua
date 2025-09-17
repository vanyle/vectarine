local t = 0

Global.fullscreen = false
if Global.fullscreen then
    local screen = getScreenSize()
    dprint("Screen: ", screen)
    setFullscreen(true)
    setWindowSize(screen.x, screen.y)

    local v = V2(1, 2)

    local w = V2(1, 2)
    local r = v + w
else
    setFullscreen(false)
    setWindowSize(800, 600)
end


function Load()
    dprint("Loading ...")
    Global.logo = loadImage("textures/logo.png")
    Global.font = loadFont("fonts/arial.ttf")
    Global.fullscreen = false
    Global.frame_times = {}
end

function Update(time_delta)
    local bg_color = { r = 1, g = 1, b = 1, a = 1 }
    clear(bg_color)

    local rect_color = { r = 0, g = 0, b = 1, a = 1 }

    local v = V2(1, 2)
    local w = V2(1, 2)
    local r = v * w
    fprint("hello", r.x, r.y)

    if isKeyDown("space") then
        rect_color = { r = 1, g = 0, b = 1, a = 1 }
    end

    t = t + 1
    local m = getMouse()
    -- fprint("Hello: " .. x .. "," .. y)

    local time_sum = 0
    for i, v in ipairs(Global.frame_times) do
        time_sum = time_sum + v
    end
    local avg_time = time_sum / #Global.frame_times

    table.insert(Global.frame_times, time_delta)
    if #Global.frame_times > 10 then
        table.remove(Global.frame_times, 1)
    end

    fprint("AVG Frame time = " .. math.floor(10000 * avg_time) / 10 .. "ms")
    fprint("AVG FPS = " .. math.floor(10 / avg_time) / 10)

    drawCircle(m, 0.1, rect_color)

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
                drawRect(p, s, rect_color)
            end
        end
    end
    -- drawImage(Global.logo, -0.2, -0.2, 0.4, 0.4)
    local text = "HY$@llo World!  ..."
    local textSize = 0.1

    -- Technique for drawing a box around text.
    local mesurement = measureText(text, Global.font, textSize)
    local toBaseline = mesurement.height - mesurement.bearingY
    drawRect(V2(-0.5, 0.5 + toBaseline), V2(mesurement.width, mesurement.height), { r = 0, g = 1, b = 0, a = 0.5 })

    -- Center of the screen
    drawCircle(V2(0, 0), 0.1, { r = 0.5, g = 0, b = 0.5, a = 1 })

    drawText(text, Global.font, V2(-0.5, 0.5), textSize, { r = 0, g = 0, b = 0, a = 1 })
end
