local debug = require('@vectarine/debug')
local graphics = require('@vectarine/graphics')
local image = require('@vectarine/image')
local io = require('@vectarine/io')
local loader = require('@vectarine/loader')
local persist = require('@vectarine/persist')
local physics = require('@vectarine/physics')
local ui = require('@vectarine/ui')
local vec = require('@vectarine/vec')
local vec4 = require('@vectarine/vec4')

--- This module handles interactive objects which are the object with the B button icon above them.
local module = loader.init() or {}

local doors = persist.onReload({
	enabled = false
}, "doors")

export type Interactable = {
	tile: string,
	position: vec.Vec2,
	dialog: { string },
	dialogIndex: number,
	goto: vec.Vec2?, 
}

-- TODO: Generalize this to "InteractibleManager".
-- Chests, doors and npc are all interactible. You can only interact with one of them.
-- We can still put a dialog manager inside of this.
export type InteractableManager = {
	interactibles: { Interactable },
	tiles: image.ImageWithTileset,
	icons: image.ImageWithTileset,
	animationTimer: number,
	interactingObject: Interactable?,
}

function module.newInteractibleManager(tiles: image.ImageWithTileset, icons: image.ImageWithTileset): InteractableManager
	return {
		interactibles = {} :: { Interactable },
		tiles = tiles,
		icons = icons,
		animationTimer = 0,
		interactingObject = nil,
	}
end

function module.newInteractible(interactibleManager: InteractableManager, tile: string, position: vec.Vec2, dialog: { string }, goto: vec.Vec2?): Interactable
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

function module.drawInteractibles(interactibleManager: InteractableManager, deltaTime: number, playerPos: vec.Vec2, player: { o: physics.Object2 })
	interactibleManager.animationTimer += deltaTime

	local startedInteracting = false
	local interactableActivationRange = 2

	for _, interactable in interactibleManager.interactibles do
		interactibleManager.tiles:drawTile(interactable.tile, interactable.position + vec.V2(0, 1), vec.V2(1, -1))
	end

	for _, interactable in interactibleManager.interactibles do
		if (interactable.position - playerPos):length() < interactableActivationRange then
			if interactable.tile == "open_door" and not doors.enabled then
				continue
			end
			if interactable ~= interactibleManager.interactingObject then
				local p = math.sin(interactibleManager.animationTimer * 5) * 0.1
				interactibleManager.icons:drawTile("b", interactable.position + vec.V2(0, p), vec.V2(1, -1))
				if io.isKeyJustPressed("B") then
					startedInteracting = true
					interactibleManager.interactingObject = interactable
					interactable.dialogIndex = 1
				end
				if interactable.tile == "npc" then
					interactibleManager.tiles:drawTile("npc2", interactable.position + vec.V2(0, 1), vec.V2(1, -1))
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
			graphics.withoutTransformation(function()
				drawDialog(interactable.dialog[interactable.dialogIndex])
			end)
			if io.isKeyJustPressed("B") then
				interactable.dialogIndex += 1
				if interactable.dialogIndex > #interactable.dialog then
					interactibleManager.interactingObject = nil
				end
			end
		elseif interactable.tile == "open_door" then
			if interactable.goto then
				debug.print("Teleporting the player to: ", interactable.goto)
				player.o.position = interactable.goto
			end
			interactibleManager.interactingObject = nil
		end
	end
end

function drawDialog(text: string)
	graphics.withoutTransformation(function()
		local dialogUi = ui.stack({ alignX = "center", alignY = "start" }, {
			ui.widget(vec.V2(2, 2), function() end),
			ui.stack({ alignX = "center", alignY = "center" }, {
				ui.widget(vec.V2(0.8, 0.5), function()
					graphics.drawRect(vec.V2(-1, -1), vec.V2(0.8, 0.5), vec4.WHITE)
				end),
				ui.stack({ alignX = "center", alignY = "center" }, {
					ui.widget(vec.V2(0.75, 0.45), function()
						graphics.drawRect(vec.V2(-1, -1), vec.V2(0.75, 0.45), vec4.BLACK)
					end),
					ui.text(text, { align = "left", fontSize = 0.1, color = vec4.WHITE, maxWidth = 0.7, maxHeight = 0.4 }),
				}),
			}),
		})

		dialogUi:draw({})
	end)
end

return module
