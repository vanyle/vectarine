local t = 0

function Update(time_delta)
    local bg_color = { r = 1, g = 0, b = 0, a = 255 }
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
    fprint("Hello: " .. x .. "," .. y)

    drawRect(x, y, 0.1, 0.2, rect_color)
    drawRect(0, 0.2, 0.2, 0.1, rect_color)
end
