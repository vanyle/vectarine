local coord = require("@vectarine/coord")
local image = require("@vectarine/image")
local text = require("@vectarine/text")
local vec = require("@vectarine/vec")
local vec4 = require("@vectarine/vec4")

--[[
Builds user interfaces for your games, such as menus, settings screens and inventories, from widgets that you combine.

It is made for game interfaces and is thus very flexible to allow you to create playful and unique interfaces.

The core concept is a `Widget`, which is a rectangular area on the screen which has a size and can be drawn.

The main widget is the Generic widget obtained using the `ui.widget` function which is drawn using a function you provide.

Widgets can be combined: some take other widgets as children and lay them out, others show text or images, scroll
their content or let the player pick a value. A widget is drawn with `Widget:draw`, usually once per frame in `Update`.

Widgets are immutable: once they are created, you cannot change their content.
Use the generic widget and its draw function to create dynamic widgets that can react to game state or user input.
For example, you can wrap a column widget inside a generic widget to change its content by creating a new column widget and drawing it when changes are needed.

Sizes are in screen coordinates, where the whole screen is 2 units wide and 2 units high (see `coord`). A widget has its
own coordinate system: its bottom left corner is at (-1, -1) and it extends to the right and up by its size. By
default, a widget is drawn with its bottom left corner at the bottom left corner of the screen. To place it somewhere
else, draw it inside `graphics.withTransformation`.

A widget that takes other widgets as children keeps its own copy of each of them. Drawing it draws these copies, so
reading `Widget:eventState` or calling `TabWidget:setActiveTab` on the original child does not affect what is shown.

# Example

```luau
const ui = require("@vectarine/ui")
const graphics = require("@vectarine/graphics")
const vec = require("@vectarine/vec")
const vec4 = require("@vectarine/vec4")

local title = ui.text("Paused", { fontSize = 0.15, color = vec4.WHITE })
local spacer = ui.spacer(vec.V2(0, 0.1))
local button = ui.widget(vec.V2(0.6, 0.15), function(event: ui.EventState, extra: any)
	local color = if event.isMouseInside then vec4.RED else vec4.GRAY
	graphics.drawRect(vec.V2(-1, -1), vec.V2(0.6, 0.15), color)
end)
local menu = ui.column({ align = "center", padding = 0.1 }, { title, spacer, button })

function Update(deltaTime: number)
	graphics.clear(vec4.BLACK)
	-- Moves the bottom left corner of the menu from the bottom left of the screen to the center
	graphics.withTransformation({ translation = vec.V2(1, 1) }, function()
		menu:draw()
	end)
end
```

# Common mistakes

- A widget that takes children keeps a copy of each of them. Calling `Widget:eventState` or `TabWidget:setActiveTab` on the original
  child does not affect what is drawn. Read the mouse state in the draw function of a `ui.widget` instead.
- A widget is drawn with its bottom left corner at the bottom left of the screen. Use `graphics.withTransformation` to
  place it somewhere else.
- Widgets are immutable. To change a list or a layout, build a new widget instead of looking for a setter.
]]
local module = {}

--- A rectangular area of the screen with a size, that can draw itself and react to the mouse.
---
--- You get one from the functions of this module, such as `ui.widget`, `ui.column` or `ui.text`. The type parameter
--- `T` is the type of the extra value that you give to `Widget:draw` and that is passed down to all the widgets inside
--- it. Use `any` if you don't need it.
export type Widget<T> = {
	--- Returns the size of this widget, as a `Vec2` in screen coordinates.
	---
	--- The size of a widget with children is computed from the sizes of its children, and the
	--- size of a text widget from its text, so this is not free for big interfaces: store the result instead of calling
	--- it many times. `extra` is passed down to the children that need it, such as text widgets that use a function.
	---
	--- @param extra The extra value of the interface, given to the functions of the widgets inside it. Nothing
	--- is sent when you omit it.
	--- @return The width in `x` and the height in `y`.
	---
	--- # Example
	---
	--- ```luau
	--- const ui = require("@vectarine/ui")
	--- const vec = require("@vectarine/vec")
	---
	--- local box = ui.spacer(vec.V2(0.5, 0.25))
	--- local row = ui.row({ gap = 0.1 }, { box, box })
	--- print(row:size()) -- 1.1, 0.25
	--- ```
	size: (self: Widget<T>, extra: T) -> vec.Vec2,
	--- Draws this widget with its bottom left corner at the origin of the current transformation.
	---
	--- Without a transformation, this is the bottom left corner of the screen. Use `graphics.withTransformation` to
	--- move, scale or rotate it. Drawing also updates the state that `Widget:eventState` returns: the mouse is
	--- tested against the widget as it is drawn, including its transformation.
	---
	--- Widgets with children draw them, and pass `extra` to them. A child only sees mouse events when the mouse is
	--- inside its parent.
	---
	--- @param extra A value of your choice, such as the game state, given to the draw functions of the widgets inside.
	---
	--- # Errors
	---
	--- Raises an error if the widget is drawn while it is already being drawn, for example by a generic widget that
	--- draws itself in its own draw function.
	---
	--- # Example
	---
	--- ```luau
	--- const ui = require("@vectarine/ui")
	--- const graphics = require("@vectarine/graphics")
	--- const vec = require("@vectarine/vec")
	--- const vec4 = require("@vectarine/vec4")
	---
	--- local square = ui.widget(vec.V2(0.5, 0.5), function(event: ui.EventState, score: number)
	--- 	graphics.drawRect(vec.V2(-1, -1), vec.V2(0.5, 0.5), vec4.RED)
	--- end)
	---
	--- function Update(deltaTime: number)
	--- 	square:draw(120)
	--- end
	--- ```
	draw: (self: Widget<T>, extra: T) -> (),
	--- Returns the mouse state of this widget the last time it was drawn.
	---
	--- Use it to react to hovering and clicks outside of the draw function. Before the first draw, every field is
	--- `false`. A widget that was not drawn this frame keeps the state of its last draw. The draw function of a
	--- generic widget receives the same table as its first argument.
	---
	--- @return A table of booleans, see `EventState`.
	---
	--- # Example
	---
	--- ```luau
	--- const ui = require("@vectarine/ui")
	--- const graphics = require("@vectarine/graphics")
	--- const vec = require("@vectarine/vec")
	--- const vec4 = require("@vectarine/vec4")
	---
	--- local button = ui.widget(vec.V2(0.5, 0.2), function()
	--- 	graphics.drawRect(vec.V2(-1, -1), vec.V2(0.5, 0.2), vec4.BLUE)
	--- end)
	---
	--- function Update(deltaTime: number)
	--- 	button:draw()
	--- 	if button:eventState().isMouseJustReleased then
	--- 		print("Clicked!")
	--- 	end
	--- end
	--- ```
	eventState: (self: Widget<T>) -> EventState,

	--- Returns a short description of the widget and of what it contains, such as `Column(Text, Image)`.
	---
	--- This is what `print` and `tostring` show. It helps to understand the structure of an interface when debugging.
	__tostring: (self: Widget<T>) -> string,
}

--- The state of the mouse relative to a widget, as of the last time it was drawn.
---
--- A generic widget receives it in its draw function (`ui.widget`), the functions of text and image widgets receive it,
--- and `Widget:eventState` returns it. All fields are `false` when the mouse is outside the widget, when the widget is
--- inside a parent that the mouse is not in, and before the first draw. The mouse is the real mouse, following the
--- transformation (move, scale, rotation) the widget is drawn with. Fields that start with `isMouseJust` are `true` for a
--- single draw.
export type EventState = {
	--- Whether the mouse is currently inside this widget's area.
	isMouseInside: boolean,
	--- Whether the left mouse button is currently down on this widget.
	isMouseDown: boolean,
	--- Whether the left mouse button was just pressed on this widget.
	isMouseJustPressed: boolean,
	--- Whether the left mouse button was just released on this widget. It is `true` when the mouse is inside the widget
	--- and the button was down at the previous draw, which makes it a good test for a click.
	isMouseJustReleased: boolean,
	--- Whether the mouse just entered this widget's area.
	isMouseJustEntered: boolean,
	--- Whether the mouse just exited this widget's area.
	isMouseJustExited: boolean,
}

--- The padding of a container on each side, in screen coordinates.
---
--- It is the table form of the `padding` option of `ContainerOptions`. A side that you leave out is 0.
---
--- # Example
---
--- ```luau
--- const ui = require("@vectarine/ui")
---
--- const vec = require("@vectarine/vec")
---
--- local panel = ui.column({ padding = { top = 0.1, bottom = 0.1, left = 0.05, right = 0.05 } }, { ui.spacer(vec.V2(1, 1)) })
--- ```
export type Padding = {
	--- The space above the content.
	top: number?,
	--- The space below the content.
	bottom: number?,
	--- The space on the left of the content.
	left: number?,
	--- The space on the right of the content.
	right: number?,
}

--- Options for the widgets that lay out children in a line, `ui.column` and `ui.row`.
---
--- Pass a table with the fields you need to `ui.column` or `ui.row`. All fields are optional, and `{}` uses the defaults.
---
--- # Example
---
--- ```luau
--- const ui = require("@vectarine/ui")
---
--- const vec = require("@vectarine/vec")
---
--- local item = ui.spacer(vec.V2(0.5, 0.1))
--- local list = ui.column({ padding = 0.1, gap = 0.05, align = "center" }, { item, item })
--- ```
export type ContainerOptions = {
	--- Padding around the container's content. Can be a single number (uniform on all sides)
	--- or a table with optional top/bottom/left/right fields. Defaults to 0.
	padding: (Padding | number)?,
	--- Distance between children. Defaults to 0.
	gap: number?,
	--- Cross-axis alignment of children, similar to CSS `align-items`.
	--- For a column: aligns children horizontally (`"start"` is left, `"center"`, `"end"` is right).
	--- For a row: aligns children vertically (`"start"` is bottom, `"center"`, `"end"` is top).
	--- Defaults to `"start"`. Any other string behaves like `"start"`.
	align: ("start" | "center" | "end")?,
}

--- Creates a generic widget, with a fixed size and a function that draws it.
---
--- This is the base of every custom widget: use it to draw something of your own in an interface, such as a button, an
--- icon or a part of the game, and to make a part of the interface react to the mouse. Combine it with the other
--- widgets by giving it to a widget that takes children, such as `ui.column`.
---
--- `drawFn` is called each time the widget is drawn. It draws in the widget's own coordinates: the bottom left corner of
--- the widget is (-1, -1) and the top right corner is (-1 + width, -1 + height). It receives the `EventState` of the
--- widget, to react to the mouse, and the extra value given to `Widget:draw`.
---
--- The size is fixed when the widget is created. To get a widget that changes size, create a new widget when needed.
---
--- @param size The size of the widget as a `Vec2`, in screen coordinates (the whole screen is 2 by 2).
--- @param drawFn The function that draws the widget. It receives the `EventState` and the extra value.
--- @return The new `Widget`.
---
--- # Example
---
--- ```luau
--- const ui = require("@vectarine/ui")
--- const graphics = require("@vectarine/graphics")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- local button = ui.widget(vec.V2(0.6, 0.2), function(event: ui.EventState, extra: any)
--- 	local color = if event.isMouseDown then vec4.BLUE elseif event.isMouseInside then vec4.RED else vec4.GRAY
--- 	graphics.drawRect(vec.V2(-1, -1), vec.V2(0.6, 0.2), color)
--- end)
---
--- function Update(deltaTime: number)
--- 	button:draw()
--- end
--- ```
---
--- # See also
---
--- - `graphics.withTransformation`: move or scale a widget when drawing it.
--- - `ui.column`: combine widgets.
function module.widget<T>(size: vec.Vec2, drawFn: (event: EventState, extra: T) -> ()): Widget<T>
	error("Implemented in native code")
end

--- Creates a column, a vertical layout that places its children from top to bottom.
---
--- The first child is at the top. The column is as wide as its widest child and as tall as the sum of the heights of its
--- children, plus the gaps between them and the padding. The `align` option sets where the children that are narrower
--- than the column go. Pass the extra value of `Widget:draw` to the children.
---
--- An empty column has the size of its padding only.
---
--- @param options The padding, gap and alignment, see `ContainerOptions`. Use `{}` for the defaults.
--- @param children The widgets to stack, from top to bottom.
--- @return The new column `Widget`.
---
--- # Example
---
--- ```luau
--- const ui = require("@vectarine/ui")
--- const vec4 = require("@vectarine/vec4")
---
--- local menu = ui.column({ gap = 0.05, align = "center" }, {
--- 	ui.text("My game", { fontSize = 0.2, color = vec4.WHITE }),
--- 	ui.text("Play", { fontSize = 0.1, color = vec4.WHITE }),
--- 	ui.text("Quit", { fontSize = 0.1, color = vec4.WHITE }),
--- })
--- ```
---
--- # See also
---
--- - `ui.row`: the horizontal version.
--- - `ui.stack`: put widgets on top of each other.
function module.column<T>(options: ContainerOptions, children: { Widget<T> }): Widget<T>
	error("Implemented in native code")
end

--- Creates a widget that takes up space and draws nothing.
---
--- Use it to leave a gap between widgets, or to push widgets apart in a `ui.row` or a `ui.column`. For a regular gap
--- between all children, the `gap` option of the container is simpler.
---
--- @param size The size of the empty space, as a `Vec2` in screen coordinates.
--- @return The new `Widget`.
---
--- # Example
---
--- ```luau
--- const ui = require("@vectarine/ui")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- local row = ui.row({}, {
--- 	ui.text("Left", { color = vec4.WHITE }),
--- 	ui.spacer(vec.V2(0.5, 0)),
--- 	ui.text("Right", { color = vec4.WHITE }),
--- })
--- ```
function module.spacer<any>(size: vec.Vec2): Widget<any>
	error("Implemented in native code")
end

--- Options for text widgets, allows you to customize the color, font, size and alignment of the text.
---
--- Pass a table with the fields you need as the second argument of `ui.text`. All fields are optional.
---
--- # Example
---
--- ```luau
--- const ui = require("@vectarine/ui")
--- const vec4 = require("@vectarine/vec4")
---
--- local description = ui.text("A long text that wraps.", {
--- 	fontSize = 0.08,
--- 	color = vec4.WHITE,
--- 	align = "left",
--- 	maxWidth = 0.8,
--- 	maxHeight = 0.4,
--- })
--- ```
type TextOptions<T> = {
	--- Horizontal alignment of the text within the widget. Defaults to `"center"`.
	align: ("left" | "center" | "right")?,
	--- The font to use for this text. Defaults to the default font, `text.font`.
	font: text.FontResource?,
	--- The color of the text, or a function that returns the color. The function receives the `EventState` and
	--- the extra value, so the color can change when the mouse hovers the text. Defaults to black.
	color: (vec4.Vec4 | ((event: EventState, extra: T) -> vec4.Vec4))?,
	--- The font size of the text, in screen coordinates, which is also the height of a line. Defaults to 0.1, so a line
	--- takes 5% of the screen height.
	fontSize: number?,

	--- The width of the text widget. When set, the text is wrapped to fit within this width: lines break at spaces and
	--- at `"\n"`, and a word wider than the width stays on its own line and overflows. Defaults to the width of the text.
	maxWidth: number?,
	--- The height of the text widget. When set, the text is clipped with an ellipsis (`"..."`) on its last line to fit
	--- within this height, with at least one line always shown. Defaults to the number of lines needed, or one line
	--- without `maxWidth`.
	maxHeight: number?,
}

--- Creates a widget that draws text.
---
--- `text` is the string to draw, or a function that returns it. The function is called each time the widget is drawn
--- and measured, with the `EventState` and the extra value of `Widget:draw`. Use it to change the text without
--- rebuilding the widget, for example to show a score or to react to hovering. It should not raise errors, and should be
--- quick, since it is also called to compute the size of the widget.
---
--- The size of the widget depends on the options:
---
--- - Without `maxWidth` and `maxHeight`, it is the width of the text and the height of one line (`fontSize`).
--- - With only `maxWidth`, the text wraps and the height is the number of lines times `fontSize`.
--- - With both, the size is exactly `maxWidth` by `maxHeight`, wrapping and clipping the text with `"..."`.
--- - With only `maxHeight`, the width is the width of the text.
---
--- If the font is not loaded yet, nothing is drawn, and the widget has a width and a height of 0.
---
--- @param text The text, or a function `(event, extra) -> string` that returns it.
--- @param options The font, color, size and alignment, see `TextOptions`. Use `{}` for the defaults.
--- @return The new `Widget`.
---
--- # Errors
---
--- Raises an error if `text` is neither a string nor a function.
---
--- # Example
---
--- ```luau
--- const ui = require("@vectarine/ui")
--- const vec4 = require("@vectarine/vec4")
---
--- local score = 0
--- local label = ui.text(function()
--- 	return "Score: " .. score
--- end, { fontSize = 0.1, color = vec4.WHITE, align = "left" })
--- ```
---
--- # See also
---
--- - `FontResource:drawText`: draw text outside of a UI.
--- - `ui.stack`: put text over an image.
function module.text<T>(text: string | ((event: EventState, extra: T) -> string), options: TextOptions<T>): Widget<T>
	error("Implemented in native code")
end

--- Options for image widgets, to keep the proportions, tint or slice the image.
---
--- Pass a table with the fields you need as the third argument of `ui.image`. All fields are optional.
---
--- # Example
---
--- ```luau
--- const ui = require("@vectarine/ui")
--- const loader = require("@vectarine/loader")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- local icon = loader.loadImage("images/icon.png")
--- local widget = ui.image(vec.V2(0.2, 0.2), icon, {
--- 	preserveAspectRatio = true,
--- 	tint = function() return vec4.WHITE end,
--- })
--- ```
type ImageWidgetOptions<T> = {
	--- Should the aspect ratio of the image be preserved? If it is, there might be empty space on the sides of the
	--- image, and the image is centered in the widget. Defaults to `false`, which stretches the image to the widget.
	preserveAspectRatio: boolean?,
	--- A function that returns a color tint to apply to the image when drawing. It receives the `EventState` and the
	--- extra value. Defaults to white (no tint).
	tint: ((event: EventState, extra: T) -> vec4.Vec4)?,
	--- If provided, the image will be drawn in 9-slices to preserve its corners when scaled. This assumes that the image is
	--- designed for this and has the appropriate borders. If preserveAspectRatio is also `true`, this has no effect.
	--- The number is the ratio of the corners to the total image size (e.g. 0.2 means corners are 20% of the image size).
	--- Defaults to `nil`, which means no slicing.
	nineSlicing: number?,
}

--- Creates a widget that draws an image.
---
--- The image is stretched to the size of the widget, unless `preserveAspectRatio` is set. Use `nineSlicing` for
--- buttons and panels that must keep sharp corners at any size.
---
--- If the image is not loaded yet, nothing is drawn, but the widget still takes its space.
---
--- @param size The size of the widget as a `Vec2`, in screen coordinates.
--- @param image The image to draw, from `loader.loadImage`.
--- @param options The proportions, tint and slicing, see `ImageWidgetOptions`. Use `{}` for the defaults.
--- @return The new `Widget`.
---
--- # Example
---
--- ```luau
--- const ui = require("@vectarine/ui")
--- const loader = require("@vectarine/loader")
--- const vec = require("@vectarine/vec")
---
--- local panelImage = loader.loadImage("images/panel.png")
--- local panel = ui.image(vec.V2(1, 0.6), panelImage, { nineSlicing = 0.2 })
--- ```
function module.image<T>(size: vec.Vec2, image: image.ImageResource, options: ImageWidgetOptions<T>): Widget<T>
	error("Implemented in native code")
end

--- Creates a row, a horizontal layout that places its children from left to right.
---
--- The first child is on the left. The row is as tall as its tallest child and as wide as the sum of the widths of its
--- children, plus the gaps between them and the padding. The `align` option sets where the children that are shorter
--- than the row go. Pass the extra value of `Widget:draw` to the children.
---
--- An empty row has the size of its padding only.
---
--- @param options The padding, gap and alignment, see `ContainerOptions`. Use `{}` for the defaults.
--- @param children The widgets to place, from left to right.
--- @return The new row `Widget`.
---
--- # Example
---
--- ```luau
--- const ui = require("@vectarine/ui")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- local heart = ui.spacer(vec.V2(0.1, 0.1))
--- local lives = ui.row({ gap = 0.02, align = "center" }, {
--- 	heart,
--- 	heart,
--- 	ui.text("x2", { fontSize = 0.1, color = vec4.WHITE }),
--- })
--- ```
function module.row<T>(options: ContainerOptions, children: { Widget<T> }): Widget<T>
	error("Implemented in native code")
end

--- Information passed to the scrollbar draw function.
---
--- It is the table received by the `drawScrollbar` function of `ui.scrollableArea`. Sizes are in screen coordinates.
---
--- # Example
---
--- ```luau
--- const ui = require("@vectarine/ui")
--- const vec = require("@vectarine/vec")
---
--- local list = ui.scrollableArea(vec.V2(0.8, 0.6), ui.spacer(vec.V2(0.8, 2)), function(info: ui.ScrollbarInfo)
--- 	print(info.visibleRatio)
--- 	return info.scrollRatio
--- end)
--- ```
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

--- Creates a scrollable area that shows a portion of its content and allows *vertical* scrolling to see the rest.
---
--- The area has a fixed size, `viewSize`, and clips the content to it, on both axes. The content starts at the top. If
--- the content is not taller than the view, it stays at the top and no scrollbar is shown. If it is taller, the mouse
--- wheel scrolls it while the mouse is inside the area, and a scrollbar is drawn on the right edge. The default
--- scrollbar is a gray track with a thumb that can be dragged with the mouse.
---
--- To draw your own scrollbar, pass `drawScrollbar`. It is called each frame, only when scrolling is needed, after the
--- content is drawn and without clipping. It draws in the coordinates of the area (its bottom left corner is (-1, -1)),
--- and receives a `ScrollbarInfo` with the current scroll state. It must return the scroll ratio to use, from 0 (top)
--- to 1 (bottom), clamped to that range: return `info.scrollRatio` to leave the scroll unchanged, or a new value to
--- scroll, for example when the user drags your scrollbar.
---
--- The height of the content is measured without an extra value, so text functions of the content receive `nil` as
--- extra when they are measured.
---
--- @param viewSize The size of the visible area, as a `Vec2` in screen coordinates.
--- @param content The widget to scroll.
--- @param drawScrollbar A function that draws a custom scrollbar and returns the scroll ratio. Defaults to the
--- built-in scrollbar.
--- @return The new `Widget`, whose size is `viewSize`.
---
--- # Example
---
--- ```luau
--- const ui = require("@vectarine/ui")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- local lines = {}
--- for i = 1, 30 do
--- 	table.insert(lines, ui.text("Line " .. i, { fontSize = 0.08, color = vec4.WHITE, align = "left" }))
--- end
--- local log = ui.scrollableArea(vec.V2(0.8, 0.6), ui.column({}, lines))
--- ```
function module.scrollableArea<T>(
	viewSize: vec.Vec2,
	content: Widget<T>,
	drawScrollbar: ((info: ScrollbarInfo) -> number)?
): Widget<T>
	error("Implemented in native code")
end

--- Options for stack widgets, to align the children that are smaller than the stack.
---
--- Pass a table with the fields you need to `ui.stack`. All fields are optional.
---
--- # Example
---
--- ```luau
--- const ui = require("@vectarine/ui")
---
--- const vec = require("@vectarine/vec")
---
--- local layers = ui.stack({ alignX = "center", alignY = "end" }, { ui.spacer(vec.V2(1, 1)) })
--- ```
type StackOptions = {
	--- Horizontal alignment of the children: `"start"` is left, `"center"`, `"end"` is right. Defaults to `"start"`.
	alignX: ("start" | "center" | "end")?,
	--- Vertical alignment of the children: `"start"` is bottom, `"center"`, `"end"` is top. Defaults to `"start"`.
	alignY: ("start" | "center" | "end")?,
}

--- Creates a stack, a widget that places its children on top of each other.
---
--- The size of the stack is the maximum width and the maximum height of its children. Children are drawn in the order
--- provided, so the first child is on the bottom and the last child is on the top. A child smaller than the stack is
--- placed according to `alignX` and `alignY`.
---
--- This can be used to show text with a background image for example, or to draw text twice for a shadow effect.
---
--- @param options The alignment of the children, see `StackOptions`. Use `{}` for the defaults.
--- @param content The widgets to superpose, from the bottom one to the top one.
--- @return The new `Widget`.
---
--- # Example
---
--- ```luau
--- const ui = require("@vectarine/ui")
--- const loader = require("@vectarine/loader")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- local background = ui.image(vec.V2(0.8, 0.2), loader.loadImage("images/button.png"), { nineSlicing = 0.2 })
--- local label = ui.text("Play", { fontSize = 0.1, color = vec4.WHITE })
--- local button = ui.stack({ alignX = "center", alignY = "center" }, { background, label })
--- ```
function module.stack<T>(options: StackOptions, content: { Widget<T> }): Widget<T>
	error("Implemented in native code")
end

--- Options for slider widgets, to set the range, step and callback of the slider.
---
--- Pass a table with the fields you need as the second argument of `ui.slider`.
---
--- # Example
---
--- ```luau
--- const ui = require("@vectarine/ui")
---
--- local options = {
--- 	min = 0,
--- 	max = 10,
--- 	initialValue = 5,
--- 	step = 1,
--- 	onChange = function(value: number)
--- 		print(value)
--- 	end,
--- }
--- ```
type SliderOptions = {
	--- The minimum value of the slider. Defaults to 0.
	min: number?,
	--- The maximum value of the slider. Defaults to 1.
	max: number?,
	--- The initial value of the slider, clamped between min and max. Defaults to min.
	initialValue: number?,
	--- The step size of the slider. If provided, the slider value will snap to multiples of this step size between min and max.
	--- Defaults to 0.01. A step of 0 or less gives a continuous slider.
	step: number?,
	--- Callback function that is called when the slider value changes. The new value is passed as an argument. It is
	--- not called when the slider is created.
	onChange: ((value: number) -> ())?,
}

--- Creates a slider, a widget that lets the player choose a value by dragging a handle along a track horizontally.
---
--- You provide the appearance with two widgets: `track`, drawn at the left of the slider and centered vertically,
--- and `handle`, which moves along the track. The handle travels over the width of the slider minus its own width.
---
--- The player picks a value by pressing the mouse on the slider, which moves the handle there, and dragging. The
--- drag continues while the button is down, even if the mouse leaves the slider. The value snaps to `step`. The slider
--- has no function to read its value: keep it yourself from the `onChange` callback.
---
--- @param size The size of the slider as a `Vec2` in screen coordinates. The handle moves over its width.
--- @param options The range, initial value, step and callback, see `SliderOptions`.
--- @param track The widget drawn as the track.
--- @param handle The widget drawn as the handle.
--- @return The new `Widget`.
---
--- # Example
---
--- ```luau
--- const ui = require("@vectarine/ui")
--- const graphics = require("@vectarine/graphics")
--- const vec = require("@vectarine/vec")
--- const vec4 = require("@vectarine/vec4")
---
--- local volume = 0.5
--- local track = ui.widget(vec.V2(0.6, 0.03), function()
--- 	graphics.drawRect(vec.V2(-1, -1), vec.V2(0.6, 0.03), vec4.GRAY)
--- end)
--- local handle = ui.widget(vec.V2(0.05, 0.1), function()
--- 	graphics.drawRect(vec.V2(-1, -1), vec.V2(0.05, 0.1), vec4.RED)
--- end)
--- local slider = ui.slider(vec.V2(0.6, 0.1), {
--- 	min = 0,
--- 	max = 1,
--- 	initialValue = volume,
--- 	onChange = function(value: number)
--- 		volume = value
--- 	end,
--- }, track, handle)
--- ```
function module.slider<T>(size: vec.Vec2, options: SliderOptions, track: Widget<T>, handle: Widget<T>): Widget<T>
	error("Implemented in native code")
end

--- Describes the animation played by `TabWidget:setActiveTab` when it switches tab.
---
--- Pass a table with both fields. The built-in animations reveal the new tab over the old one, without moving the
--- content, and the mouse is ignored by the old tab while it plays.
export type TabAnimation<T> = {
	--- Duration of the animation in seconds.
	duration: number,
	--- The animation: `"slideLeft"`, `"slideRight"`, `"slideUp"` and `"slideDown"` reveal the new tab from the right, left,
	--- bottom and top edge respectively; `"toon"` reveals it with a circle that grows from the center. You can also give
	--- a function that draws the transition itself: it is called each frame during the animation with the extra value,
	--- the old widget, the new widget (each can be `nil` if the tab does not exist) and the progress from 0 to 1, and
	--- must draw both widgets itself. Any other string raises an error.
	animationType: "slideLeft"
		| "slideRight"
		| "slideUp"
		| "slideDown"
		| "toon"
		| (T, Widget<T>, Widget<T>, number) -> (), -- custom animation function with T, old widget, new widget and progress (0 to 1)
}

--- A widget that shows one of several widgets and can switch between them.
---
--- You get one from `ui.tabs`. It is a `Widget` with extra methods to switch between tabs.
export type TabWidget<T> = {
	--- Switches to the tab with the given name, optionally with an animation.
	---
	--- The content of the tab is drawn and its events are processed from then on. Does nothing if this tab is already
	--- the active one. If no tab has this name, no error is raised: nothing is drawn and the widget has a size of 0.
	---
	--- @param tabName The name of the tab, a key of the table given to `ui.tabs`.
	--- @param animation The transition to play, see `TabAnimation`. Defaults to `nil`, which switches immediately.
	---
	--- # Errors
	---
	--- Raises an error if it is called while the tab widget is being drawn, if `animation` has no `duration` or no
	--- `animationType`, or if `animationType` is a string that is not one of the built-in animations.
	---
	--- # Example
	---
	--- ```luau
	--- const ui = require("@vectarine/ui")
	--- const vec = require("@vectarine/vec")
	---
	--- local pages = ui.tabs({
	--- 	menu = ui.spacer(vec.V2(1, 1)),
	--- 	options = ui.spacer(vec.V2(1, 1)),
	--- })
	--- pages:setActiveTab("options", { duration = 0.3, animationType = "slideLeft" })
	--- ```
	setActiveTab: (self: TabWidget<T>, tabName: string, animation: TabAnimation<T>?) -> (),
	--- Returns the name of the currently active tab. During a transition, this will return the new tab name.
	---
	--- @return The name of the tab, or the empty string for the tab named `""`.
	---
	--- # Example
	---
	--- ```luau
	--- const ui = require("@vectarine/ui")
	--- const vec = require("@vectarine/vec")
	---
	--- local pages = ui.tabs({ [""] = ui.spacer(vec.V2(1, 1)), options = ui.spacer(vec.V2(1, 1)) })
	--- pages:setActiveTab("options")
	--- print(pages:getActiveTab()) -- options
	--- ```
	getActiveTab: (self: TabWidget<T>) -> string,
} & Widget<T>

--- Creates a tab widget that shows one of several widgets and lets you switch between them with `TabWidget:setActiveTab`.
---
--- `tabs` maps tab names to widgets. At first, the tab named `""` (the empty string) is shown. If there is no tab with
--- this name, one of the others is picked, in no guaranteed order, so name one tab `""` when it matters. The size of
--- the tab widget is the size of the tab it is showing. Only the active tab is drawn and receives mouse events.
---
--- Switching can play an animation (see `TabAnimation`). The tab widget keeps its own copy of the widgets of the
--- table, as for every widget that takes children.
---
--- @param tabs A table mapping tab names to widgets.
--- @return The new `TabWidget`.
---
--- # Example
---
--- ```luau
--- const ui = require("@vectarine/ui")
--- const vec4 = require("@vectarine/vec4")
---
--- local pages = ui.tabs({
--- 	[""] = ui.text("Main menu", { color = vec4.WHITE }),
--- 	settings = ui.text("Settings", { color = vec4.WHITE }),
--- })
---
--- function Update(deltaTime: number)
--- 	pages:draw()
--- end
--- ```
---
--- # See also
---
--- - `TabWidget:setActiveTab`: switch tab.
function module.tabs<T>(tabs: { [string]: Widget<T> }): TabWidget<T>
	error("Implemented in native code")
end

--- Runs a draw function that shows a debugging outline on the widgets drawn inside.
---
--- A widget that is hovered by the mouse gets a transparent red rectangle over it, which helps to see what is taking
--- up space on the screen and to understand the layout of your widgets. The previous setting is restored when
--- `drawFn` ends, so calls can be nested.
---
--- @param drawFn The function that draws the widgets, which are shown with outlines.
--- @param toggle Pass `false` to disable outlines inside `drawFn` and `true` to enable them. Defaults to `true`.
---
--- # Example
---
--- ```luau
--- const ui = require("@vectarine/ui")
--- const vec = require("@vectarine/vec")
---
--- local menu = ui.spacer(vec.V2(1, 1))
---
--- function Update(deltaTime: number)
--- 	ui.withDebugOutline(function()
--- 		menu:draw()
--- 	end)
--- end
--- ```
function module.withDebugOutline<T>(drawFn: () -> (), toggle: boolean?): ()
	error("Implemented in native code")
end

return module
