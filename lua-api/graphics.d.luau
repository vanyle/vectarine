export type Color = { r: number; g: number; b: number; a: number }

--- Clear the canvas
declare function clear(color: Color): ()

--- Draws a filled rectangle
declare function drawRect(pos: Vec2, size: Vec2, color: Color): ()

--- Draws a convex filled polygon
declare function drawPolygon(points: {Vec2}, color: Color): ()

--- Draws an arrow starting at `pos`, and towards `direction`
declare function drawArrow(pos: Vec2, direction: Vec2, color: Color?, size: number?): ()

--- Draws a filled circle
declare function drawCircle(center: Vec2, radius: number, color: Color): ()

--- Draws an image
declare function drawImage(image: ImageResource, pos: Vec2, size: Vec2): ()

--- Draws a rectangular part of an image delimited by a position and a size
--- This section is deformed to match the quadrilateral delimited by the 4 destination points
--- You can swap the destination points to rotate, flip the image.
--- Use `src_pos = V2(0, 0)` and `src_size = V2(1, 1)` to draw the full image.
declare function drawImage(image: ImageResource, dest_p1: Vec2, dest_p2: Vec2, dest_p3: Vec2, dest_p4: Vec2, src_pos: Vec2, src_size: Vec2): ()

--- Draws text at (x,y) with given font, size and color
--- size is the maximum height that a text of that font will take on the screen
--- This is sometimes called the line height.
declare function drawText(text: string, font: FontResource, pos: Vec2, size: number, color: Color): ()

--- Measures how much space the text will take when drawn
--- height will always be less than size.
--- bearingY will always be less than height. This is the distance from the top of the text to the baseline.
declare function measureText(text: string, font: FontResource, size: number): {width: number, height: number, bearingY: number}
