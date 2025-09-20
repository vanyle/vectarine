use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use mlua::Table;

use crate::{
    game_resource::ResourceId,
    graphics::draw_instruction::DrawInstruction,
    lua_env::{add_global_fn, lua_vec2::Vec2},
};

pub fn setup_graphics_api(
    lua: &Rc<mlua::Lua>,
    draw_instructions: &Rc<RefCell<VecDeque<DrawInstruction>>>,
) -> mlua::Result<()> {
    let queue_for_closure = draw_instructions.clone();
    add_global_fn(
        lua,
        "drawRect",
        move |_, (pos, size, color): (Vec2, Vec2, Table)| {
            let color = [
                color.get::<f32>("r").unwrap_or(0.0),
                color.get::<f32>("g").unwrap_or(0.0),
                color.get::<f32>("b").unwrap_or(0.0),
                color.get::<f32>("a").unwrap_or(0.0),
            ];
            queue_for_closure
                .borrow_mut()
                .push_back(DrawInstruction::Rectangle { pos, size, color });
            Ok(())
        },
    );

    let queue_for_closure = draw_instructions.clone();
    add_global_fn(
        lua,
        "drawPolygon",
        move |_, (points, color): (Vec<Vec2>, Table)| {
            schedule_draw_polygon(&queue_for_closure, points, color)
        },
    );

    let queue_for_closure = draw_instructions.clone();
    add_global_fn(
        lua,
        "drawArrow",
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
            schedule_draw_polygon(&queue_for_closure, Vec::from([p1, p2, p3]), color.clone())?;

            let p1 = pos - perp.scale(arrow_width / 2.0);
            let p2 = pos + dir - perp.scale(arrow_width / 2.0);
            let p3 = pos + dir + perp.scale(arrow_width / 2.0);
            let p4 = pos + perp.scale(arrow_width / 2.0);
            schedule_draw_polygon(&queue_for_closure, Vec::from([p1, p2, p3, p4]), color)?;
            Ok(())
        },
    );

    let queue_for_closure = draw_instructions.clone();
    add_global_fn(
        lua,
        "drawCircle",
        move |_, (pos, radius, color): (Vec2, f32, Table)| {
            let color = [
                color.get::<f32>("r").unwrap_or(0.0),
                color.get::<f32>("g").unwrap_or(0.0),
                color.get::<f32>("b").unwrap_or(0.0),
                color.get::<f32>("a").unwrap_or(0.0),
            ];
            queue_for_closure
                .borrow_mut()
                .push_back(DrawInstruction::Circle { pos, radius, color });
            Ok(())
        },
    );

    let queue_for_closure = draw_instructions.clone();
    add_global_fn(
        lua,
        "drawImage",
        move |_, (id, pos, size): (ResourceId, Vec2, Vec2)| {
            let draw_ins = DrawInstruction::Image { pos, size, id };
            queue_for_closure.borrow_mut().push_back(draw_ins);
            Ok(())
        },
    );

    let queue_for_closure = draw_instructions.clone();
    add_global_fn(
        lua,
        "drawImage",
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
            queue_for_closure.borrow_mut().push_back(draw_ins);
            Ok(())
        },
    );

    let queue_for_closure = draw_instructions.clone();
    add_global_fn(
        lua,
        "drawText",
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
            queue_for_closure.borrow_mut().push_back(draw_ins);
            Ok(())
        },
    );

    let queue_for_closure = draw_instructions.clone();
    add_global_fn(lua, "clear", move |_, (color,): (Table,)| {
        let color = [
            color.get::<f32>("r").unwrap_or(0.0),
            color.get::<f32>("g").unwrap_or(0.0),
            color.get::<f32>("b").unwrap_or(0.0),
            color.get::<f32>("a").unwrap_or(0.0),
        ];
        queue_for_closure
            .borrow_mut()
            .push_back(DrawInstruction::Clear { color });
        Ok(())
    });

    Ok(())
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
