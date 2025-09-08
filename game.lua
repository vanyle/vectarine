function Update(time_delta)
    SetColor(0, 255, 0)
    Clear()
    if IsKeyDown("space") then
        SetColor(255, 0, 255)
    else
        SetColor(0, 0, 0)
    end
    DrawRect(100, 100, 200, 150)

    SetColor(0, 0, 255)
end
