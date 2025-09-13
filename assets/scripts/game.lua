function Update(time_delta)
    local bg_color = { r = 1, g = 0, b = 0, a = 255 }
    Clear(bg_color)

    local rect_color = { r = 0, g = 0, b = 1, a = 1 }

    if IsKeyDown("space") then
        rect_color = { r = 1, g = 0, b = 1, a = 1 }
    end

    DrawRect(-1, -1, 0.1, 0.2, rect_color)
    DrawRect(0, 0.2, 0.2, 0.1, rect_color)
end
