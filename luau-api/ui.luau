local Vec = require("@vectarine/vec")

--[[
# UI module

This module provides the building blocks for creating user interfaces for your games.
It is made for game interfaces and is thus very flexible to allow you to create playful and unique interfaces.

The core concept is a `Widget`, which is a rectangular area on the screen which has a size and can be drawn.

The main widget is the Generic widget obtained using the `ui.widget` function which is drawn using a function you provide.

You can compose widgets together using the `ui.column` and `ui.row` widgets.
]]
local module = {}

export type Widget = {
	--- Returns the preferred size of this widget.
	size: (self: Widget) -> Vec.Vec2,
	--- Draws this widget at the current position in the batch.
	draw: (self: Widget) -> (),
	--- Return the event state from the last time this widget was drawn. You can use this to react to events on the widget (e.g. mouse hover, clicks).
	eventState: (self: Widget) -> EventState,
}

export type EventState = {
	--- Whether the mouse is currently inside this widget's area.
	isMouseInside: boolean,
	--- Whether the left mouse button is currently down on this widget.
	isMouseDown: boolean,
	--- Whether the left mouse button was just pressed on this widget
	isMouseJustPressed: boolean,
	--- Whether the left mouse button was just released on this widget
	isMouseJustReleased: boolean,
	--- Whether the mouse just entered this widget's area
	isMouseJustEntered: boolean,
	--- Whether the mouse just exited this widget's area
	isMouseJustExited: boolean,
}

export type Padding = {
	top: number?,
	bottom: number?,
	left: number?,
	right: number?,
}

--- Options for container widgets (column, row).
export type ContainerOptions = {
	--- Padding around the container's content. Can be a single number (uniform on all sides)
	--- or a table with optional top/bottom/left/right fields. Defaults to 0.
	padding: (Padding | number)?,
	--- Distance between children. Defaults to 0.
	gap: number?,
	--- Cross-axis alignment of children, similar to CSS `align-items`.
	--- For a column: aligns children horizontally ("start" = left, "center", "end" = right).
	--- For a row: aligns children vertically ("start" = bottom, "center", "end" = top).
	--- Defaults to "start".
	align: ("start" | "center" | "end")?,
}

--- Create a generic widget with a fixed preferred size and a custom draw function.
--- Can be used to create custom widgets or draw parts of the game in the UI.
--- Can also be used for buttons, or to make part of the UI react to events
---@param size Vec.Vec2 — the preferred size of this widget
---@param drawFn (event: EventState) -> () — called to draw the widget content
function module.widget(size: Vec.Vec2, drawFn: (event: EventState) -> ()): Widget
	error("Implemented in native code")
end

--- Create a column (vertical layout) that stacks its children top to bottom.
--- The column's preferred size accounts for children sizes, gap between them, and padding.
--- Children are drawn from top to bottom in the order provided.
---@param options ContainerOptions — optional padding and gap
---@param children { Widget } — ordered list of child widgets
function module.column(options: ContainerOptions, children: { Widget }): Widget
	error("Implemented in native code")
end

--- Create a row (horizontal layout) that places its children left to right.
--- The row's preferred size accounts for children sizes, gap between them, and padding.
--- Children are drawn from left to right in the order provided.
---@param options ContainerOptions — optional padding and gap
---@param children { Widget } — ordered list of child widgets
function module.row(options: ContainerOptions, children: { Widget }): Widget
	error("Implemented in native code")
end

return module
