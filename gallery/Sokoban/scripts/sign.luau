const debug = require("@vectarine/debug")
const graphics = require("@vectarine/graphics")
const io = require("@vectarine/io")
const loader = require("@vectarine/loader")
const ui = require("@vectarine/ui")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")
const worldTypes = require("./world")

const module = loader.init() or {}

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
	[2] = "In case of mistakes, press " .. io.getKeyName("Z") .. " to undo",
	[3] = "Find the treasure to win!",
}

function module.drawDialog(world: worldTypes.IWorld, signManager: SignManager)
	if io.isKeyJustPressed("B") then
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
		graphics.withoutTransformation(function()
			local dialogUi = ui.stack({ alignX = "center", alignY = "start" }, {
				ui.widget(vec.V2(2, 2), function() end),
				-- We can probably draw a sign image to the left of the text and a text above which says "Sign"
				ui.stack({ alignX = "center", alignY = "center" }, {
					ui.widget(vec.V2(1.0, 0.7), function()
						graphics.drawRect(vec.V2(-1, -1), vec.V2(1.0, 0.7), vec4.WHITE)
					end),
					ui.stack({ alignX = "center", alignY = "center" }, {
						ui.widget(vec.V2(0.95, 0.65), function()
							graphics.drawRect(vec.V2(-1, -1), vec.V2(0.95, 0.65), vec4.BLACK)
						end),
						ui.column({}, {
							ui.text("The Sign", {
								align = "left",
								color = vec4.WHITE,
								fontSize = 0.08,
								maxWidth = 0.9,
								maxHeight = 0.05,
							}),
							ui.widget(vec.V2(0.0, 0.07), function() end),
							ui.text(
								function()
									local progress = math.floor(signManager.messageProgress)
									return sign_message[signManager.currentMessage]:sub(1, progress)
								end,
								{ align = "left", fontSize = 0.1, color = vec4.WHITE, maxWidth = 0.9, maxHeight = 0.4 }
							),
							ui.text("Press B to continue", {
								align = "left",
								color = vec4.WHITE,
								fontSize = 0.08,
								maxWidth = 0.9,
								maxHeight = 0.05,
							}),
						}),
					}),
				}),
			})

			dialogUi:draw({})
		end)
	end
end

return module
