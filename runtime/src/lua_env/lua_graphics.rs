use std::{cell::RefCell, rc::Rc};

use mlua::AnyUserData;

use crate::{
    game_resource,
    graphics::{batchdraw, glstencil::draw_with_mask},
    io,
    lua_env::{
        add_fn_to_table,
        lua_coord::{get_pos_as_vec2, get_size_as_vec2},
        lua_vec4::{BLACK, Vec4, WHITE},
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
        move |_, (mpos, msize, color): (AnyUserData, AnyUserData, Option<Vec4>)| {
            let pos = get_pos_as_vec2(mpos)?;
            let size = get_size_as_vec2(msize)?;
            batch.borrow_mut().draw_rect(
                pos.x(),
                pos.y(),
                size.x(),
                size.y(),
                color.unwrap_or(BLACK).0,
            );
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawPolygon", {
        let batch = batch.clone();
        move |_, (points, color): (Vec<AnyUserData>, Option<Vec4>)| {
            let points = points
                .into_iter()
                .map(|p| get_pos_as_vec2(p).unwrap_or_default());
            batch
                .borrow_mut()
                .draw_polygon(points, color.unwrap_or(BLACK).0);
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawArrow", {
        let batch = batch.clone();
        move |_lua, (mpos, mdir, color, size): (AnyUserData, AnyUserData, Option<Vec4>, Option<f32>)| {
            let pos = get_pos_as_vec2(mpos)?;
            let dir = get_size_as_vec2(mdir)?;
            let color = color.unwrap_or(BLACK);
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
            batch.draw_polygon([p1, p2, p3].into_iter(), color.0);

            let p1 = pos - perp.scale(arrow_width / 2.0);
            let p2 = pos + dir - perp.scale(arrow_width / 2.0);
            let p3 = pos + dir + perp.scale(arrow_width / 2.0);
            let p4 = pos + perp.scale(arrow_width / 2.0);
            batch.draw_polygon([p1, p2, p3, p4].into_iter(), color.0);
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawCircle", {
        let batch = batch.clone();
        move |_, (mpos, radius, color): (AnyUserData, f32, Option<Vec4>)| {
            let pos = get_pos_as_vec2(mpos)?;
            batch
                .borrow_mut()
                .draw_circle(pos.x(), pos.y(), radius, color.unwrap_or(BLACK).0);
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
        move |_, (color,): (Option<Vec4>,)| {
            batch.borrow_mut().clear(color.unwrap_or(WHITE).0);
            Ok(())
        }
    });

    Ok(graphics_module)
}
