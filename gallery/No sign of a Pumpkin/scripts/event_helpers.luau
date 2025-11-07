local Io = require("@vectarine/io")

local INITIAL_ACTION_ID = 1

-- Delayed action system (similar to setTimeout in JavaScript)
local delayed_actions = {}
local next_action_id = INITIAL_ACTION_ID

function schedule_action(delay: number, action: () -> (), description: string?): number
	local action_id = next_action_id
	next_action_id += 1
	table.insert(delayed_actions, {
		id = action_id,
		delay = delay,
		elapsed = 0.0,
		action = action,
		description = description or "Unknown action",
	})
	return action_id
end

function cancel_action(action_id: number)
	for i, delayed_action in ipairs(delayed_actions) do
		if delayed_action.id == action_id then
			table.remove(delayed_actions, i)
			return true
		end
	end
	return false
end

function manage_delayed_actions(time_delta: number)
	for i = #delayed_actions, 1, -1 do -- Iterate backwards to safely remove items
		local delayed_action = delayed_actions[i]
		delayed_action.elapsed += time_delta

		if delayed_action.elapsed >= delayed_action.delay then
			delayed_action.action()
			table.remove(delayed_actions, i)
		end
	end
end

function clear_all_delayed_actions()
	delayed_actions = {}
	next_action_id = INITIAL_ACTION_ID
end

-- Check if any movement input is pressed
function is_any_movement_input_pressed(): boolean
	return Io.isKeyDown("q") or Io.isKeyDown("d") or Io.isKeyDown("z") or Io.isKeyDown("s")
end

-- Export the module functions
return {
	schedule_action = schedule_action,
	cancel_action = cancel_action,
	manage_delayed_actions = manage_delayed_actions,
	clear_all_delayed_actions = clear_all_delayed_actions,
	is_any_movement_input_pressed = is_any_movement_input_pressed,
}
