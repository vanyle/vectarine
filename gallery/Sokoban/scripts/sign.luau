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
							ui.text(vec.V2(0.9, 0.05), { align = "left", fitting = { size = 0.08 } }, function()
								return {
									text = "The Sign",
									color = vec4.WHITE,
								}
							end),
							ui.widget(vec.V2(0.0, 0.07), function() end),
							ui.text(vec.V2(0.9, 0.4), { align = "left", fitting = { size = 0.1 } }, function()
								local progress = math.floor(signManager.messageProgress)
								return {
									text = sign_message[signManager.currentMessage]:sub(1, progress),
									color = vec4.WHITE,
								}
							end),
							ui.text(vec.V2(0.9, 0.05), { align = "left", fitting = { size = 0.08 } }, function()
								return {
									text = "Press B to continue",
									color = vec4.WHITE,
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
