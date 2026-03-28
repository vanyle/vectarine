local Vec = require("@vectarine/vec")
local Vec4 = require("@vectarine/vec4")
local module = {}

local WidgetImpl = { type = "widget" }
WidgetImpl.__index = WidgetImpl
export type Widget = typeof(setmetatable({}, WidgetImpl))

--- Draw a widget at a given position and size, and returns its response
--- If you omit the size, the widget will be drawn at its preferred size
function WidgetImpl:draw(
	pos: Vec.Vec2,
	size: Vec.Vec2?
): { isHovered: boolean, isClicked: boolean, isJustClicked: boolean, desiredSize: Vec.Vec2? }
	error("Not implemented")
end

local BoxImpl = { type = "button" }
BoxImpl.__index = WidgetImpl
export type Box = typeof(setmetatable({}, BoxImpl))

--- Create a box. A box is a customizable widget. It can be a button, some text, or anything else.
--- You need to pass a draw function to draw the box.
--- The draw function is called in the coordinate system of the box, so to fill the box, you need
--- to do: `Graphics:drawRect(Vec.V2(-1, -1), Vec.V2(2, 2))`
function module.createBox(
	preferedSize: Vec.Vec2?,
	drawFn: (isHovered: boolean, isClicked: boolean, isJustClicked: boolean) -> ()
): Box
	error("Not implemented")
end

local VerticalScrollableImpl = { type = "vertical_scrollable" }
VerticalScrollableImpl.__index = VerticalScrollableImpl
export type VerticalScrollable = typeof(setmetatable({}, VerticalScrollableImpl))

--- Create a vertical scrollable area. It allows to scroll its content vertically if it exceeds the available height.
function module.verticalScrollable(
	size: Vec.Vec2,
	drawContent: (scrollPosition: number) -> (),
	drawScrollbar: ((scrollPosition: number, totalHeight: number) -> ())?
): VerticalScrollable
	error("Not implemented")
end

local TextImpl = { type = "text" }
TextImpl.__index = WidgetImpl
export type Text = typeof(setmetatable({}, TextImpl))

--- Create a text widget. It displays the text at the location provided by `alignmentX` and `alignmentY` in the parent widget.
--- If there is no parent widget, it does so relative to the screen. If no alignment is provided, it defaults to center.
--- You can offset the text from this location using the `offset` parameter. This can be used to draw outlines or shadows.
function module.createText(
	text: string,
	fontSize: number,
	color: Vec4.Vec4,
	alignmentX: ("left" | "center" | "right")?,
	alignmentY: ("top" | "center" | "bottom")?,
	offset: Vec.Vec2?
): Box
	error("Not implemented")
end

local ColumnImpl = { type = "column" }
ColumnImpl.__index = WidgetImpl
export type Column = typeof(setmetatable({}, ColumnImpl))

--- Create a column. A column is a vertical layout that can contain other widgets
function module.createColumn(
	padding: Vec.Vec2?,
	spacing: number,
	align: "left" | "center" | "right",
	widgets: { Widget }
): Column
	error("Not implemented")
end

local RowImpl = { type = "row" }
RowImpl.__index = WidgetImpl
export type Row = typeof(setmetatable({}, RowImpl))

--- Create a row. A row is a horizontal layout that can contain other widgets
function module.createRow(
	padding: Vec.Vec2?,
	spacing: number,
	align: "top" | "center" | "bottom",
	widgets: { Widget }
): Row
	error("Not implemented")
end

local SpacerImpl = { type = "spacer" }
SpacerImpl.__index = WidgetImpl
export type Spacer = typeof(setmetatable({}, SpacerImpl))

--- Create a spacer. A spacer is a widget that takes up space but does nothing
--- You can thing of it as a box with no draw function.
function module.createSpacer(preferedSize: Vec.Vec2): Spacer
	error("Not implemented")
end

return module
