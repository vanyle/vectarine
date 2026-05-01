local Debug = require("@vectarine/debug")
local Graphics = require("@vectarine/graphics")
local Image = require("@vectarine/image")
local Io = require("@vectarine/io")
local Persist = require("@vectarine/persist")
local Physics = require("@vectarine/physics")
local Ui = require("@vectarine/ui")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")

--- This module handles interactive objects which are the object with the B button icon above them.
local module = {}

local doors = Persist.onReload({
	enabled = false
}, "doors")

export type Interactable = {
	tile: string,
	position: Vec.Vec2,
	dialog: { string },
	dialogIndex: number,
	goto: Vec.Vec2?, 
}

-- TODO: Generalize this to "InteractibleManager".
-- Chests, doors and npc are all interactible. You can only interact with one of them.
-- We can still put a dialog manager inside of this.
export type InteractableManager = {
	interactibles: { Interactable },
	tiles: Image.ImageWithTileset,
	icons: Image.ImageWithTileset,
	animationTimer: number,
	interactingObject: Interactable?,
}

function module.newInteractibleManager(tiles: Image.ImageWithTileset, icons: Image.ImageWithTileset): InteractableManager
	return {
		interactibles = {} :: { Interactable },
		tiles = tiles,
		icons = icons,
		animationTimer = 0,
		interactingObject = nil,
	}
end

function module.newInteractible(interactibleManager: InteractableManager, tile: string, position: Vec.Vec2, dialog: { string }, goto: Vec.Vec2?): Interactable
	local interactible = {
		tile = tile,
		position = position,
		dialog = dialog,
		dialogIndex = 1,
		goto = goto,
	}
	table.insert(interactibleManager.interactibles, interactible)
	return interactible
end

function module.drawInteractibles(interactibleManager: InteractableManager, deltaTime: number, playerPos: Vec.Vec2, player: { o: Physics.Object2 })
	interactibleManager.animationTimer += deltaTime

	local startedInteracting = false
	local interactableActivationRange = 2

	for _, interactable in interactibleManager.interactibles do
		interactibleManager.tiles:drawTile(interactable.tile, interactable.position + Vec.V2(0, 1), Vec.V2(1, -1))
	end

	for _, interactable in interactibleManager.interactibles do
		if (interactable.position - playerPos):length() < interactableActivationRange then
			if interactable.tile == "open_door" and not doors.enabled then
				continue
			end
			if interactable ~= interactibleManager.interactingObject then
				local p = math.sin(interactibleManager.animationTimer * 5) * 0.1
				interactibleManager.icons:drawTile("b", interactable.position + Vec.V2(0, p), Vec.V2(1, -1))
				if Io.isKeyJustPressed("B") then
					startedInteracting = true
					interactibleManager.interactingObject = interactable
					interactable.dialogIndex = 1
				end
				if interactable.tile == "npc" then
					interactibleManager.tiles:drawTile("npc2", interactable.position + Vec.V2(0, 1), Vec.V2(1, -1))
				end
				break
			end
		end
	end

	if interactibleManager.interactingObject and not startedInteracting then
		local interactable = interactibleManager.interactingObject
		-- Handle the interactions with the different npc types.
		if interactable.tile == "npc" then
			-- Interacting with an npc means speaking with it.
			Graphics.withoutTransformation(function()
				drawDialog(interactable.dialog[interactable.dialogIndex])
			end)
			if Io.isKeyJustPressed("B") then
				interactable.dialogIndex += 1
				if interactable.dialogIndex > #interactable.dialog then
					interactibleManager.interactingObject = nil
				end
			end
		elseif interactable.tile == "open_door" then
			if interactable.goto then
				Debug.print("Teleporting the player to: ", interactable.goto)
				player.o.position = interactable.goto
			end
			interactibleManager.interactingObject = nil
		end
	end
end

function drawDialog(text: string)
	Graphics.withoutTransformation(function()
		local dialogUi = Ui.stack({ alignX = "center", alignY = "start" }, {
			Ui.widget(Vec.V2(2, 2), function() end),
			Ui.stack({ alignX = "center", alignY = "center" }, {
				Ui.widget(Vec.V2(0.8, 0.5), function()
					Graphics.drawRect(Vec.V2(-1, -1), Vec.V2(0.8, 0.5), Vec4.WHITE)
				end),
				Ui.stack({ alignX = "center", alignY = "center" }, {
					Ui.widget(Vec.V2(0.75, 0.45), function()
						Graphics.drawRect(Vec.V2(-1, -1), Vec.V2(0.75, 0.45), Vec4.BLACK)
					end),
					Ui.text(Vec.V2(0.7, 0.4), { align = "left", fitting = { size = 0.1 } }, function()
						return { text = text, color = Vec4.WHITE }
					end),
				}),
			}),
		})

		dialogUi:draw({})
	end)
end

return module
