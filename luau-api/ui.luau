local Vec = require("@vectarine/vec")

--[[

]]
local module = {}

--- A UI widget. Widgets can be composed together using `ui.column` and `ui.row`.
export type Widget = {
	--- Returns the preferred size of this widget.
	size: (self: Widget) -> Vec.Vec2,
	--- Draws this widget at the current position in the batch.
	draw: (self: Widget) -> (),
}

--- Create a generic widget with a fixed preferred size and a custom draw function.
--- The draw function is called when the widget is drawn and should issue draw calls
--- for the widget's content. The drawing is done in the current batch transform space.
---@param size Vec.Vec2 — the preferred size of this widget
---@param drawFn () -> () — called to draw the widget content
function module.widget(size: Vec.Vec2, drawFn: () -> ()): Widget
	error("Not implemented")
end

--- Create a column (vertical layout) that stacks its children top to bottom.
--- Children are laid out from top to bottom in order using each child's preferred size.
---@param children { Widget } — ordered list of child widgets
function module.column(children: { Widget }): Widget
	error("Not implemented")
end

--- Create a row (horizontal layout) that places its children left to right.
--- Children are laid out from left to right in order using each child's preferred size.
---@param children { Widget } — ordered list of child widgets
function module.row(children: { Widget }): Widget
	error("Not implemented")
end

return module
