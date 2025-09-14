local t = 0

function Load()
    dprint("Loading ...")
    Global.logo = loadImage("textures/logo.png")
    Global.frame_times = {}
end

function Update(time_delta)
    local bg_color = { r = 1, g = 1, b = 1, a = 1 }
    clear(bg_color)

    local rect_color = { r = 0, g = 0, b = 1, a = 1 }

    if isKeyDown("space") then
        rect_color = { r = 1, g = 0, b = 1, a = 1 }
    end

    t = t + 1
    local m = mouse()
    local w = windowSize()
    local x = (m.x / w.x) * 2 - 1
    local y = (-m.y / w.y) * 2 + 1
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

    drawRect(x, y, 0.1, 0.1, rect_color)

    local slow = false
    if slow then
        local slow_factor = 150
        for i = 0, slow_factor do
            for j = 0, slow_factor do
                local ratio = slow_factor / 2
                rect_color.g = (i + t * 3) % 255 / 255
                rect_color.b = (j + t) % 255 / 255
                drawRect(-1 + i / ratio, -1 + j / ratio, 0.1, 0.1, rect_color)
            end
        end
    end

    drawImage(Global.logo, -0.2, -0.2, 0.4, 0.4)
end
