use crate::{
    console::Verbosity,
    game_resource::{ResourceId, font_resource::FontResource, image_resource::ImageResource},
    graphics::shape::Quad,
    lua_env::{self, lua_vec2::Vec2},
};

#[derive(Debug, Clone)]
pub enum DrawInstruction {
    Rectangle {
        pos: Vec2,
        size: Vec2,
        color: [f32; 4],
    },
    Polygon {
        points: Vec<Vec2>,
        color: [f32; 4],
    },
    Circle {
        pos: Vec2,
        radius: f32,
        color: [f32; 4],
    },
    Image {
        pos: Vec2,
        size: Vec2,
        id: ResourceId,
    },
    ImagePart {
        p1: Vec2,
        p2: Vec2,
        p3: Vec2,
        p4: Vec2,
        id: ResourceId,
        uv_pos: Vec2,
        uv_size: Vec2,
    },
    Text {
        pos: Vec2,
        text: String,
        color: [f32; 4],
        font_size: f32,
        font_id: ResourceId,
    },
    Clear {
        color: [f32; 4],
    },
}

pub fn render_instruction(
    instruction: DrawInstruction,
    lua_env: &mut lua_env::LuaEnvironment, // for printing errors to the console
) {
    let batch = &mut lua_env.batch;
    let resource_manager = &lua_env.resources;

    match instruction {
        DrawInstruction::Rectangle { pos, size, color } => {
            batch.draw_rect(pos.x, pos.y, size.x, size.y, color);
        }
        DrawInstruction::Polygon { points, color } => {
            batch.draw_polygon(points, color);
        }
        DrawInstruction::Circle { pos, radius, color } => {
            batch.draw_circle(pos.x, pos.y, radius, color);
        }
        DrawInstruction::Image { pos, size, id } => {
            let resource = resource_manager.get_by_id::<ImageResource>(id);
            let Ok(image_resource) = resource else {
                return; // not loaded or wrong type
            };
            let texture = image_resource.texture.borrow();
            let texture = texture.as_ref();
            let Some(texture) = texture else {
                debug_assert!(false, "Resource said it was loaded but the texture is None");
                unreachable!(); // texture is not loaded. This probably breaks an invariant.
            };
            batch.draw_image(pos.x, pos.y, size.x, size.y, texture);
        }
        DrawInstruction::ImagePart {
            p1,
            p2,
            p3,
            p4,
            id,
            uv_pos,
            uv_size,
        } => {
            let resource = resource_manager.get_by_id::<ImageResource>(id);
            let Ok(image_resource) = resource else {
                return;
            };
            let texture = image_resource.texture.borrow();
            let texture = texture.as_ref();
            let Some(texture) = texture else {
                debug_assert!(false, "Resource said it was loaded but the texture is None");
                unreachable!();
            };
            batch.draw_image_part(Quad { p1, p2, p3, p4 }, texture, uv_pos, uv_size);
        }
        DrawInstruction::Text {
            pos,
            text,
            color,
            font_size,
            font_id,
        } => {
            let resource = resource_manager.get_by_id::<FontResource>(font_id);
            let res = match resource {
                Ok(res) => res,
                Err(cause) => {
                    lua_env.print(
                        &format!("Warning: Failed to draw text with '{font_id}': {cause}"),
                        Verbosity::Warn,
                    );
                    return;
                }
            };

            let font_rendering_data = res.font_rendering.borrow();
            let font_rendering_data = font_rendering_data.as_ref();
            let Some(font_rendering_data) = font_rendering_data else {
                debug_assert!(false, "Resource said it was loaded but the texture is None");
                return; // texture is not loaded. This probably breaks an invariant.
            };
            let (x, y) = (pos.x, pos.y);
            batch.draw_text(x, y, &text, color, font_size, font_rendering_data);
        }
        DrawInstruction::Clear { color } => {
            batch.clear(color[0], color[1], color[2], color[3]);
        }
    }
}
