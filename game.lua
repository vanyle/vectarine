t = 0

function Update(time_delta)
    t += 1
    SetColor(0, 0, 0)
    Clear()
    if IsKeyDown("space") then
        t = 0
        SetColor(255, 0, 255)
    else
        SetColor(0, 0, 0)
    end
    DrawRect(100, 100, 200, 150)

    SetColor(0, 0, 255)
    DrawRect(0, 0, t % 200, 100)
end
