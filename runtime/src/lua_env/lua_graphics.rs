use std::{cell::RefCell, rc::Rc};

use mlua::Table;

use crate::{
    game_resource::{self, ResourceId, font_resource::FontResource, image_resource::ImageResource},
    graphics::{batchdraw, shape::Quad},
    io,
    lua_env::{add_fn_to_table, lua_canvas, lua_vec2::Vec2},
};

pub fn setup_graphics_api(
    lua: &Rc<mlua::Lua>,
    batch: &Rc<RefCell<batchdraw::BatchDraw2d>>,
    env_state: &Rc<RefCell<io::IoEnvState>>,
    resources: &Rc<game_resource::ResourceManager>,
) -> mlua::Result<mlua::Table> {
    let graphics_module = lua.create_table()?;

    add_fn_to_table(lua, &graphics_module, "drawRect", {
        let batch = batch.clone();
        move |_, (pos, size, color): (Vec2, Vec2, Table)| {
            let color = [
                color.get::<f32>("r").unwrap_or(0.0),
                color.get::<f32>("g").unwrap_or(0.0),
                color.get::<f32>("b").unwrap_or(0.0),
                color.get::<f32>("a").unwrap_or(0.0),
            ];
            batch
                .borrow_mut()
                .draw_rect(pos.x, pos.y, size.x, size.y, color);
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawPolygon", {
        let batch = batch.clone();
        move |_, (points, color): (Vec<Vec2>, Table)| {
            batch
                .borrow_mut()
                .draw_polygon(points, table_to_color(color));
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawArrow", {
        let batch = batch.clone();
        move |lua, (pos, dir, color, size): (Vec2, Vec2, Option<Table>, Option<f32>)| {
            let color = table_to_color(color.unwrap_or(get_default_color(lua).unwrap()));
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
            let mut batch = batch.borrow_mut();
            batch.draw_polygon(Vec::from([p1, p2, p3]), color);

            let p1 = pos - perp.scale(arrow_width / 2.0);
            let p2 = pos + dir - perp.scale(arrow_width / 2.0);
            let p3 = pos + dir + perp.scale(arrow_width / 2.0);
            let p4 = pos + perp.scale(arrow_width / 2.0);
            batch.draw_polygon(Vec::from([p1, p2, p3, p4]), color);
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawCircle", {
        let batch = batch.clone();
        move |_, (pos, radius, color): (Vec2, f32, Table)| {
            batch
                .borrow_mut()
                .draw_circle(pos.x, pos.y, radius, table_to_color(color));
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawImage", {
        let batch = batch.clone();
        let resources = resources.clone();
        move |_, (id, pos, size): (ResourceId, Vec2, Vec2)| {
            let tex = resources.get_by_id::<ImageResource>(id);
            let Ok(tex) = tex else {
                return Ok(());
            };
            let tex = tex.texture.borrow();
            let Some(tex) = tex.as_ref() else {
                return Ok(());
            };
            batch
                .borrow_mut()
                .draw_image(pos.x, pos.y, size.x, size.y, tex);
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawImagePart", {
        let batch = batch.clone();
        let resources = resources.clone();
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
            let tex = resources.get_by_id::<ImageResource>(id);
            let Ok(tex) = tex else {
                return Ok(());
            };
            let tex = tex.texture.borrow();
            let Some(tex) = tex.as_ref() else {
                return Ok(());
            };
            let quad = Quad { p1, p2, p3, p4 };
            batch
                .borrow_mut()
                .draw_image_part(quad, tex, src_pos, src_size);
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawCanvas", {
        let batch = batch.clone();
        move |_, (canvas, pos, size): (lua_canvas::RcFramebuffer, Vec2, Vec2)| {
            let canvas = canvas.gl();
            batch
                .borrow_mut()
                .draw_canvas(pos.x, pos.y, size.x, size.y, canvas);
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawText", {
        let batch = batch.clone();
        let resources = resources.clone();
        move |_, (text, font, pos, size, color): (String, ResourceId, Vec2, f32, Table)| {
            let color = table_to_color(color);
            let font_resource = resources.get_by_id::<FontResource>(font);
            let Ok(font_resource) = font_resource else {
                return Ok(());
            };
            let font_resource = font_resource.font_rendering.borrow();
            let Some(font_resource) = font_resource.as_ref() else {
                return Ok(());
            };
            batch
                .borrow_mut()
                .draw_text(pos.x, pos.y, &text, color, size, font_resource);
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "clear", {
        let batch = batch.clone();
        move |_, (color,): (Table,)| {
            let color = table_to_color(color);
            batch
                .borrow_mut()
                .clear(color[0], color[1], color[2], color[3]);
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

pub fn table_to_color(color: Table) -> [f32; 4] {
    [
        color.get::<f32>("r").unwrap_or(0.0),
        color.get::<f32>("g").unwrap_or(0.0),
        color.get::<f32>("b").unwrap_or(0.0),
        color.get::<f32>("a").unwrap_or(0.0),
    ]
}

pub fn get_default_color(lua: &mlua::Lua) -> mlua::Result<Table> {
    let default_color = lua.create_table()?;
    default_color.set("r", 0.0)?;
    default_color.set("g", 0.0)?;
    default_color.set("b", 0.0)?;
    default_color.set("a", 1.0)?;
    Ok(default_color)
}
