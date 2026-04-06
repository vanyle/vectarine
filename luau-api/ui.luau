local Image = require("@vectarine/image")
local Text = require("@vectarine/text")
local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")

--[[
# UI module

This module provides the building blocks for creating user interfaces for your games.
It is made for game interfaces and is thus very flexible to allow you to create playful and unique interfaces.

The core concept is a `Widget`, which is a rectangular area on the screen which has a size and can be drawn.

The main widget is the Generic widget obtained using the `ui.widget` function which is drawn using a function you provide.

You can compose widgets together using the `ui.column` and `ui.row` widgets.

Widgets are immutable: once they are created, you cannot change their content.
Use the generic widget and its draw function to create dynamic widgets that can react to game state or user input.
For example, you can wrap a column widget inside a generic widget to change its content by creating a new column widget and drawing it when changes are needed.
]]
local module = {}

export type Widget<T> = {
	--- Returns the preferred size of this widget.
	size: (self: Widget<T>) -> Vec.Vec2,
	--- Draws this widget at the current position in the batch.
	draw: (self: Widget<T>, extra: T) -> (),
	--- Return the event state from the last time this widget was drawn. You can use this to react to events on the widget (e.g. mouse hover, clicks).
	eventState: (self: Widget<T>) -> EventState,

	__tostring: (self: Widget<T>) -> string,
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
---@param drawFn (event: EventState, extra: T) -> () — called to draw the widget content
function module.widget<T>(size: Vec.Vec2, drawFn: (event: EventState, extra: T) -> ()): Widget<T>
	error("Implemented in native code")
end

--- Create a column (vertical layout) that stacks its children top to bottom.
--- The column's preferred size accounts for children sizes, gap between them, and padding.
--- Children are drawn from top to bottom in the order provided.
---@param options ContainerOptions — optional padding and gap
---@param children { Widget<T> } — ordered list of child widgets
function module.column<T>(options: ContainerOptions, children: { Widget<T> }): Widget<T>
	error("Implemented in native code")
end

type TextOptions = {
	--- Horizontal alignment of the text within the widget. Defaults to "center".
	align: ("left" | "center" | "right")?,
	-- Optional font to use for this text. If not provided, the default font will be used.
	font: Text.FontResource?,
}

--- Create a text widget that draws the given text. The size of the text is computed such that
--- the text fits on a single line within the given size.
--- The `getText` function is called each frame to get the text to draw and its color. You can use it to dynamically
--- change the text without needing to wrap the text widget in a generic widget.
function module.text<T>(
	size: Vec.Vec2,
	options: TextOptions,
	getText: { text: string, color: Vec4.Vec4 } | ((event: EventState, extra: T) -> { text: string, color: Vec4.Vec4 })
): Widget<T>
	error("Implemented in native code")
end

type ImageWidgetOptions<T> = {
	-- Should the aspect ratio of the image be preserved? If it is, there might be empty space on the sides of the image.
	preserveAspectRatio: boolean?,
	-- Optional color tint to apply to the image when drawing. Defaults to white (no tint).
	tint: ((event: EventState, extra: T) -> Vec4.Vec4)?,
	-- If provided, the image will be drawn in 9-slices to preserve its corners when scaled. This assumes that the image is
	-- designed for this and has the appropriate borders. If preserveAspectRatio is also true, this has no effect.
	-- The number is the ratio of the corners to the total image size (e.g. 0.2 means corners are 20% of the image size). Defaults to nil.
	nineSlicing: number?,
}

--- Create a widget that draws the given image. The image is scaled to fit the widget's size
function module.image<T>(size: Vec.Vec2, image: Image.ImageResource, options: ImageWidgetOptions<T>): Widget<T>
	error("Implemented in native code")
end

--- Create a row (horizontal layout) that places its children left to right.
--- The row's preferred size accounts for children sizes, gap between them, and padding.
--- Children are drawn from left to right in the order provided.
---@param options ContainerOptions — optional padding and gap
---@param children { Widget<T> } — ordered list of child widgets
function module.row<T>(options: ContainerOptions, children: { Widget<T> }): Widget<T>
	error("Implemented in native code")
end

--- Information passed to the scrollbar draw function.
export type ScrollbarInfo = {
	--- Scroll progress from 0 (top) to 1 (bottom).
	scrollRatio: number,
	--- Fraction of the content that is visible (0..1). Can be used to size the scrollbar thumb.
	visibleRatio: number,
	--- Width of the scrollable area's view.
	viewWidth: number,
	--- Height of the scrollable area's view.
	viewHeight: number,
	--- Total height of the content being scrolled.
	contentHeight: number,
}

--- Create a scrollable area that shows a portion of its content and allows *vertical* scrolling to see the rest.
--- If the content is smaller than the view size, the content is shown fully, centered and no scrollbar is shown.
--- If the content is larger than the view size, mouse wheel scrolling is enabled.
--- The optional `drawScrollbar` function is called each frame (only when scrolling is needed) to draw a custom scrollbar.
--- It receives a `ScrollbarInfo` table with the current scroll state. Drawing coordinates are in the scrollable area's local space.
---@param content Widget — the content widget to scroll
---@param viewSize Vec.Vec2 — the visible area size
---@param drawScrollbar ((info: ScrollbarInfo) -> number)? — optional custom scrollbar drawing function that receives the current scroll state and returns the desired scrollbar ratio.
function module.scrollableArea<T>(
	viewSize: Vec.Vec2,
	content: Widget<T>,
	drawScrollbar: ((info: ScrollbarInfo) -> number)?
): Widget<T>
	error("Implemented in native code")
end

type StackOptions = {
	alignX: ("start" | "center" | "end")?,
	alignY: ("start" | "center" | "end")?,
}

--- Create a stack that places its children on top of each other. The size of the stack is the maximum size of its children.
--- Children are drawn in the order provided, so the first child is on the bottom and the last child is on the top.
--- This can be used to show text with a background image for example, or to draw text twice for a shadow effect.
---@param options StackOptions — optional alignment of children within the stack
function module.stack<T>(options: StackOptions, content: { Widget<T> }): Widget<T>
	error("Implemented in native code")
end

--- Debugging function that makes all widgets drawn inside to show a debugging outline.
--- This can be used to visualize the layout of widgets and understand what is taking up space on the screen.
--- You can set toggle to true / false to quickly enable or disable the outline.
function module.withDebugOutline<T>(drawFn: () -> (), toggle: boolean?): ()
	error("Implemented in native code")
end

return module
