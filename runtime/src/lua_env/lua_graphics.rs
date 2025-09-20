use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use mlua::Table;

use crate::{
    game_resource::{ResourceId, font_resource::FontResource},
    graphics::draw_instruction::{self, DrawInstruction},
    lua_env::{add_fn_to_table, lua_vec2::Vec2},
};

pub fn setup_graphics_api(
    lua: &Rc<mlua::Lua>,
    draw_instructions: &Rc<RefCell<VecDeque<DrawInstruction>>>,
    env_state: &Rc<RefCell<crate::io::IoEnvState>>,
    resources: &Rc<crate::game_resource::ResourceManager>,
) -> mlua::Result<mlua::Table> {
    let graphics_module = lua.create_table()?;

    add_fn_to_table(lua, &graphics_module, "drawRect", {
        let draw_instructions = draw_instructions.clone();
        move |_, (pos, size, color): (Vec2, Vec2, Table)| {
            let color = [
                color.get::<f32>("r").unwrap_or(0.0),
                color.get::<f32>("g").unwrap_or(0.0),
                color.get::<f32>("b").unwrap_or(0.0),
                color.get::<f32>("a").unwrap_or(0.0),
            ];
            draw_instructions
                .borrow_mut()
                .push_back(DrawInstruction::Rectangle { pos, size, color });
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawPolygon", {
        let draw_instructions = draw_instructions.clone();
        move |_, (points, color): (Vec<Vec2>, Table)| {
            schedule_draw_polygon(&draw_instructions, points, color)
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawArrow", {
        let draw_instructions = draw_instructions.clone();
        move |lua, (pos, dir, color, size): (Vec2, Vec2, Option<Table>, Option<f32>)| {
            let color = color.unwrap_or(get_default_color(lua).unwrap());
            let dir_len = (dir.x * dir.x + dir.y * dir.y).sqrt();
            if dir_len == 0.0 {
                return Ok(());
            }
            let dir_norm = dir.normalized();
            let perp = dir_norm.rotated(std::f32::consts::FRAC_PI_2);
            let arrow_width = size.unwrap_or(0.01);
            let arrow_head_size = size.unwrap_or(0.01) * 2.0;
            // Draw a line as a grad and a triangle at the end
            let p1 = pos + dir - perp.scale(arrow_head_size / 1.5);
            let p2 = pos + dir + perp.scale(arrow_head_size / 1.5);
            let p3 = pos + dir + dir_norm.scale(arrow_head_size);
            schedule_draw_polygon(&draw_instructions, Vec::from([p1, p2, p3]), color.clone())?;

            let p1 = pos - perp.scale(arrow_width / 2.0);
            let p2 = pos + dir - perp.scale(arrow_width / 2.0);
            let p3 = pos + dir + perp.scale(arrow_width / 2.0);
            let p4 = pos + perp.scale(arrow_width / 2.0);
            schedule_draw_polygon(&draw_instructions, Vec::from([p1, p2, p3, p4]), color)?;
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawCircle", {
        let draw_instructions = draw_instructions.clone();
        move |_, (pos, radius, color): (Vec2, f32, Table)| {
            let color = [
                color.get::<f32>("r").unwrap_or(0.0),
                color.get::<f32>("g").unwrap_or(0.0),
                color.get::<f32>("b").unwrap_or(0.0),
                color.get::<f32>("a").unwrap_or(0.0),
            ];
            draw_instructions
                .borrow_mut()
                .push_back(DrawInstruction::Circle { pos, radius, color });
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawImage", {
        let draw_instructions = draw_instructions.clone();
        move |_, (id, pos, size): (ResourceId, Vec2, Vec2)| {
            let draw_ins = DrawInstruction::Image { pos, size, id };
            draw_instructions.borrow_mut().push_back(draw_ins);
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawImagePart", {
        let draw_instructions = draw_instructions.clone();
        move |_,
              (id, p1, p2, p3, p4, src_pos, src_size): (
            ResourceId,
            Vec2,
            Vec2,
            Vec2,
            Vec2,
            Vec2,
            Vec2,
        )| {
            let draw_ins = DrawInstruction::ImagePart {
                p1,
                p2,
                p3,
                p4,
                uv_pos: src_pos,
                uv_size: src_size,
                id,
            };
            draw_instructions.borrow_mut().push_back(draw_ins);
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawText", {
        let draw_instructions = draw_instructions.clone();
        move |_, (text, font, pos, size, color): (String, ResourceId, Vec2, f32, Table)| {
            let color = [
                color.get::<f32>("r").unwrap_or(0.0),
                color.get::<f32>("g").unwrap_or(0.0),
                color.get::<f32>("b").unwrap_or(0.0),
                color.get::<f32>("a").unwrap_or(0.0),
            ];
            let draw_ins = DrawInstruction::Text {
                pos,
                text,
                color,
                font_size: size,
                font_id: font,
            };
            draw_instructions.borrow_mut().push_back(draw_ins);
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "clear", {
        let draw_instructions = draw_instructions.clone();
        move |_, (color,): (Table,)| {
            let color = [
                color.get::<f32>("r").unwrap_or(0.0),
                color.get::<f32>("g").unwrap_or(0.0),
                color.get::<f32>("b").unwrap_or(0.0),
                color.get::<f32>("a").unwrap_or(0.0),
            ];
            draw_instructions
                .borrow_mut()
                .push_back(DrawInstruction::Clear { color });
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "measureText", {
        let resources = resources.clone();
        let env_state = env_state.clone();
        move |lua, (text, font, font_size): (String, ResourceId, f32)| {
            let font_resource = resources.get_by_id::<FontResource>(font);
            let result = lua.create_table().unwrap();
            let Ok(font_resource) = font_resource else {
                let _ = result.set("width", 0.0);
                let _ = result.set("height", 0.0);
                let _ = result.set("bearingY", 0.0);
                return Ok(result);
            };
            let env_state = env_state.borrow();
            let ratio = env_state.window_width as f32 / env_state.window_height as f32;
            let (width, height, max_ascent) = font_resource.measure_text(&text, font_size, ratio);
            let _ = result.set("width", width);
            let _ = result.set("height", height);
            let _ = result.set("bearingY", max_ascent);
            Ok(result)
        }
    });

    Ok(graphics_module)
}

fn schedule_draw_polygon(
    queue_for_closure: &RefCell<VecDeque<DrawInstruction>>,
    points: Vec<Vec2>,
    color: Table,
) -> Result<(), mlua::Error> {
    let color = [
        color.get::<f32>("r").unwrap_or(0.0),
        color.get::<f32>("g").unwrap_or(0.0),
        color.get::<f32>("b").unwrap_or(0.0),
        color.get::<f32>("a").unwrap_or(0.0),
    ];
    queue_for_closure
        .borrow_mut()
        .push_back(DrawInstruction::Polygon { points, color });
    Ok(())
}

pub fn get_default_color(lua: &mlua::Lua) -> mlua::Result<Table> {
    let default_color = lua.create_table()?;
    default_color.set("r", 0.0)?;
    default_color.set("g", 0.0)?;
    default_color.set("b", 0.0)?;
    default_color.set("a", 1.0)?;
    Ok(default_color)
}
