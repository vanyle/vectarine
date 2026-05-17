local Debug = require("@vectarine/debug")
local Graphics = require("@vectarine/graphics")
local Io = require("@vectarine/io")
local Ui = require("@vectarine/ui")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")
local WorldTypes = require("./world")

local module = {}

export type SignManager = {
	currentMessage: number,
	messageProgress: number,
}

function module.create(): SignManager
	return {
		currentMessage = 0,
		messageProgress = 0,
	}
end

local sign_message = {
	[1] = "Use the arrow keys to move.",
	[2] = "In case of mistakes, press " .. Io.getKeyName("Z") .. " to undo",
	[3] = "Find the treasure to win!",
}

function module.drawDialog(world: WorldTypes.IWorld, signManager: SignManager)
	if Io.isKeyJustPressed("B") then
		signManager.currentMessage = signManager.currentMessage + 1
		signManager.messageProgress = 0
	end
	if
		signManager.currentMessage <= #sign_message
		and signManager.messageProgress < #sign_message[signManager.currentMessage]
	then
		signManager.messageProgress = signManager.messageProgress + 0.3
	end
	if signManager.currentMessage > #sign_message then
		world.readingSign = false
		signManager.currentMessage = 0
		signManager.messageProgress = 0
	else
		Graphics.withoutTransformation(function()
			local dialogUi = Ui.stack({ alignX = "center", alignY = "start" }, {
				Ui.widget(Vec.V2(2, 2), function() end),
				-- We can probably draw a sign image to the left of the text and a text above which says "Sign"
				Ui.stack({ alignX = "center", alignY = "center" }, {
					Ui.widget(Vec.V2(1.0, 0.7), function()
						Graphics.drawRect(Vec.V2(-1, -1), Vec.V2(1.0, 0.7), Vec4.WHITE)
					end),
					Ui.stack({ alignX = "center", alignY = "center" }, {
						Ui.widget(Vec.V2(0.95, 0.65), function()
							Graphics.drawRect(Vec.V2(-1, -1), Vec.V2(0.95, 0.65), Vec4.BLACK)
						end),
						Ui.column({}, {
							Ui.text(Vec.V2(0.9, 0.05), { align = "left", fitting = { size = 0.08 } }, function()
								return {
									text = "The Sign",
									color = Vec4.WHITE,
								}
							end),
							Ui.widget(Vec.V2(0.0, 0.07), function() end),
							Ui.text(Vec.V2(0.9, 0.4), { align = "left", fitting = { size = 0.1 } }, function()
								local progress = math.floor(signManager.messageProgress)
								return {
									text = sign_message[signManager.currentMessage]:sub(1, progress),
									color = Vec4.WHITE,
								}
							end),
							Ui.text(Vec.V2(0.9, 0.05), { align = "left", fitting = { size = 0.08 } }, function()
								return {
									text = "Press B to continue",
									color = Vec4.WHITE,
								}
							end),
						}),
					}),
				}),
			})

			dialogUi:draw({})
		end)
	end
end

return module
