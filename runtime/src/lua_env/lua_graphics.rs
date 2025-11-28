use std::{cell::RefCell, rc::Rc};

use mlua::{AnyUserData, Table};

use crate::{
    game_resource,
    graphics::{batchdraw, glstencil::draw_with_mask},
    io,
    lua_env::{
        add_fn_to_table,
        lua_coord::{get_pos_as_vec2, get_size_as_vec2},
    },
};

pub fn setup_graphics_api(
    lua: &Rc<mlua::Lua>,
    batch: &Rc<RefCell<batchdraw::BatchDraw2d>>,
    _env_state: &Rc<RefCell<io::IoEnvState>>,
    resources: &Rc<game_resource::ResourceManager>,
) -> mlua::Result<mlua::Table> {
    let graphics_module = lua.create_table()?;

    add_fn_to_table(lua, &graphics_module, "drawRect", {
        let batch = batch.clone();
        move |_, (mpos, msize, color): (AnyUserData, AnyUserData, Table)| {
            let pos = get_pos_as_vec2(mpos)?;
            let size = get_size_as_vec2(msize)?;
            let color = [
                color.get::<f32>("r").unwrap_or(0.0),
                color.get::<f32>("g").unwrap_or(0.0),
                color.get::<f32>("b").unwrap_or(0.0),
                color.get::<f32>("a").unwrap_or(0.0),
            ];
            batch
                .borrow_mut()
                .draw_rect(pos.x(), pos.y(), size.x(), size.y(), color);
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawPolygon", {
        let batch = batch.clone();
        move |_, (points, color): (Vec<AnyUserData>, Table)| {
            let points = points
                .into_iter()
                .map(|p| get_pos_as_vec2(p).unwrap_or_default());
            batch
                .borrow_mut()
                .draw_polygon(points, table_to_color(color));
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawArrow", {
        let batch = batch.clone();
        move |lua, (mpos, mdir, color, size): (AnyUserData, AnyUserData, Option<Table>, Option<f32>)| {
            let pos = get_pos_as_vec2(mpos)?;
            let dir = get_size_as_vec2(mdir)?;
            let color = table_to_color(color.unwrap_or(get_default_color(lua)?));
            let dir_len = dir.length();
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
            batch.draw_polygon([p1, p2, p3].into_iter(), color);

            let p1 = pos - perp.scale(arrow_width / 2.0);
            let p2 = pos + dir - perp.scale(arrow_width / 2.0);
            let p3 = pos + dir + perp.scale(arrow_width / 2.0);
            let p4 = pos + perp.scale(arrow_width / 2.0);
            batch.draw_polygon([p1, p2, p3, p4].into_iter(), color);
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawCircle", {
        let batch = batch.clone();
        move |_, (mpos, radius, color): (AnyUserData, f32, Table)| {
            let pos = get_pos_as_vec2(mpos)?;
            batch
                .borrow_mut()
                .draw_circle(pos.x(), pos.y(), radius, table_to_color(color));
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawWithMask", {
        let batch = batch.clone();
        let resources = resources.clone();
        let gl = batch.borrow().drawing_target.gl().clone();
        move |_lua, (draw_fn, mask_fn): (mlua::Function, mlua::Function)| {
            draw_with_mask(
                &gl,
                || {
                    let _ = mask_fn.call::<()>(());
                    batch.borrow_mut().draw(&resources, true);
                },
                || {
                    let _ = draw_fn.call::<()>(());
                    batch.borrow_mut().draw(&resources, true);
                },
            );
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
