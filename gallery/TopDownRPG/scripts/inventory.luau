--[[
The bag of the player: items, stacks, equipment and consumables.

The bag has a limited number of slots (BAG_SIZE, more with the explorer's satchel). Items of the same kind stack
up to their `stack` size (see data/items.luau). Equipped items stay in the bag.
Artefacts don't go in the bag: they become permanent powers (`player.artefacts`).
]]
const items = require("./data/items")
const loader = require("@vectarine/loader")
const types = require("./types")

const module = loader.init() or {}

module.BAG_SIZE = 12
module.SATCHEL_SLOTS = 6

function module.capacity(player: types.Player): number
	return module.BAG_SIZE + (if player.artefacts.satchel then module.SATCHEL_SLOTS else 0)
end

function module.count(player: types.Player, itemId: string): number
	local total = 0
	for _, stack in player.inventory do
		if stack.id == itemId then
			total += stack.count
		end
	end
	return total
end

--- Add items to a list of stacks (the bag or a chest). Returns how many could not be added.
function module.addTo(stacks: { types.ItemStack }, capacity: number, itemId: string, count: number): number
	local maxStack = items.items[itemId].stack or 1
	local left = count
	for _, stack in stacks do
		if left == 0 then
			break
		end
		if stack.id == itemId and stack.count < maxStack then
			local added = math.min(left, maxStack - stack.count)
			stack.count += added
			left -= added
		end
	end
	while left > 0 and #stacks < capacity do
		local added = math.min(left, maxStack)
		table.insert(stacks, { id = itemId, count = added })
		left -= added
	end
	return left
end

--- Give items to the player. Artefacts become powers. Returns how many did not fit in the bag.
function module.give(player: types.Player, itemId: string, count: number?): number
	if items.items[itemId].kind == "artefact" then
		player.artefacts[itemId] = true
		return 0
	end
	return module.addTo(player.inventory, module.capacity(player), itemId, count or 1)
end

--- Remove items from a list of stacks. Returns true if there were enough.
function module.removeFrom(stacks: { types.ItemStack }, itemId: string, count: number): boolean
	local total = 0
	for _, stack in stacks do
		if stack.id == itemId then
			total += stack.count
		end
	end
	if total < count then
		return false
	end
	local left = count
	for i = #stacks, 1, -1 do
		local stack = stacks[i]
		if stack.id == itemId and left > 0 then
			local removed = math.min(left, stack.count)
			stack.count -= removed
			left -= removed
			if stack.count == 0 then
				table.remove(stacks, i)
			end
		end
	end
	return true
end

function module.take(player: types.Player, itemId: string, count: number?): boolean
	local ok = module.removeFrom(player.inventory, itemId, count or 1)
	-- An item that is no longer in the bag can't stay equipped
	for slot, equipped in player.equipment :: { [string]: string } do
		if equipped == itemId and module.count(player, itemId) == 0 then
			(player.equipment :: any)[slot] = nil
		end
	end
	return ok
end

-- MARK: Equipment

--- The slot an item goes in, if it can be equipped
function module.slotOf(itemId: string): types.Slot?
	local kind = items.items[itemId].kind
	if kind == "weapon" or kind == "armor" or kind == "charm" then
		return kind :: types.Slot
	end
	return nil
end

--- Can the player equip the item? Returns false and the reason if not.
function module.canEquip(player: types.Player, itemId: string): (boolean, string?)
	local requires = items.items[itemId].requires
	if requires then
		for stat, value in requires do
			if (player.stats :: any)[stat] < value then
				return false, `Needs {value} {stat}`
			end
		end
	end
	return true, nil
end

--- Equip the item, or unequip it if it is already equipped. Returns a message for the player.
--- A charm goes in a free charm slot; when both are taken, it replaces the one worn the longest.
function module.toggleEquip(player: types.Player, itemId: string): string?
	local slot = module.slotOf(itemId)
	if not slot then
		return nil
	end
	local equipment = player.equipment
	local name = items.items[itemId].name
	if module.isEquipped(player, itemId) then
		for key, equipped in equipment :: { [string]: string } do
			if equipped == itemId then
				(equipment :: any)[key] = nil
			end
		end
		return `{name} unequipped`
	end
	local ok, reason = module.canEquip(player, itemId)
	if not ok then
		return reason
	end
	if slot == "charm" then
		if equipment.charm and equipment.charm2 then
			equipment.charm = equipment.charm2
			equipment.charm2 = itemId
		elseif equipment.charm then
			equipment.charm2 = itemId
		else
			equipment.charm = itemId
		end
	else
		(equipment :: any)[slot] = itemId
	end
	return `{name} equipped`
end

--- The sum of a bonus ("light", "luck", "regen") given by the charms worn
function module.charmBonus(player: types.Player, key: string): number
	local total = 0
	for _, charm in { player.equipment.charm, player.equipment.charm2 } do
		local bonus = items.items[charm :: string].bonus :: any
		if bonus then
			total += bonus[key] or 0
		end
	end
	return total
end

function module.isEquipped(player: types.Player, itemId: string): boolean
	local equipment = player.equipment
	return equipment.weapon == itemId or equipment.armor == itemId or equipment.charm == itemId or equipment.charm2 == itemId
end

--- The weapon used to attack: the equipped one, or the fists.
function module.weapon(player: types.Player): string
	return player.equipment.weapon or "fist"
end

return module
