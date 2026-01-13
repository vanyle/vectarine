use std::{cell::RefCell, rc::Rc, sync::Arc};

use vectarine_plugin_sdk::mlua::{AnyUserData, ObjectLike};

use crate::{
    game_resource::{self, font_resource::use_default_font},
    graphics::{
        batchdraw,
        glstencil::draw_with_mask,
        gltexture::{ImageAntialiasing, Texture},
    },
    io,
    lua_env::{
        add_fn_to_table,
        lua_coord::{get_pos_as_vec2, get_size_as_vec2},
        lua_vec2::Vec2,
        lua_vec4::{BLACK, Vec4, WHITE},
    },
};

pub fn setup_graphics_api(
    lua: &Rc<vectarine_plugin_sdk::mlua::Lua>,
    batch: &Rc<RefCell<batchdraw::BatchDraw2d>>,
    env_state: &Rc<RefCell<io::IoEnvState>>,
    resources: &Rc<game_resource::ResourceManager>,
) -> vectarine_plugin_sdk::mlua::Result<vectarine_plugin_sdk::mlua::Table> {
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

    add_fn_to_table(lua, &graphics_module, "drawLine", {
        let batch = batch.clone();
        move |_,
              (pos1, pos2, color, thickness): (
            AnyUserData,
            AnyUserData,
            Option<Vec4>,
            Option<f32>,
        )| {
            let pos1 = get_pos_as_vec2(pos1)?;
            let pos2 = get_pos_as_vec2(pos2)?;
            let one_to_two = pos2 - pos1;
            let ortho = one_to_two
                .cmul(Vec2::new(0.0, 1.0))
                .normalized()
                .scale(thickness.unwrap_or(0.005));

            let p1 = pos1 + ortho;
            let p2 = pos2 + ortho;
            let p3 = pos2 - ortho;
            let p4 = pos1 - ortho;

            batch
                .borrow_mut()
                .draw_polygon([p1, p2, p3, p4].into_iter(), color.unwrap_or(BLACK).0);
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

    add_fn_to_table(lua, &graphics_module, "drawEllipse", {
        let batch = batch.clone();
        move |_, (mpos, size, color): (AnyUserData, AnyUserData, Option<Vec4>)| {
            let pos = get_pos_as_vec2(mpos)?;
            let size = get_size_as_vec2(size)?;
            batch.borrow_mut().draw_ellipse(
                pos.x(),
                pos.y(),
                size.x(),
                size.y(),
                color.unwrap_or(BLACK).0,
            );
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawWithMask", {
        let batch = batch.clone();
        let resources = resources.clone();
        let gl = batch.borrow().drawing_target.gl().clone();
        move |_lua, (draw_fn, mask_fn): (vectarine_plugin_sdk::mlua::Function, vectarine_plugin_sdk::mlua::Function)| {
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

    // MARK: Splash screen

    let logo_bytes = include_bytes!("../../../assets/logo.png");
    let logo_data = image::load_from_memory_with_format(logo_bytes, image::ImageFormat::Png)
        .expect("The logo image should be valid");
    let logo = Arc::new(Texture::new_rgba(
        batch.borrow().drawing_target.gl(),
        Some(logo_data.to_rgba8().as_raw().as_slice()),
        logo_data.width(),
        logo_data.height(),
        ImageAntialiasing::Linear,
    ));

    let get_draw_splash_screen_fn = || {
        let batch = batch.clone();
        let env_state = env_state.clone();
        let logo = logo.clone();
        let scale = 0.3;
        let progress_bar_length = 0.8;
        let progress_bar_height = 0.05;
        let distance_between_bar_and_logo = 0.05;
        let progress_bar_padding = 0.01;
        move |loading_text: Option<String>, progress: Option<f32>| {
            batch.borrow_mut().clear(BLACK.0);
            let env = env_state.borrow();
            let aspect = env.window_width as f32 / env.window_height as f32;
            let pos = Vec2::new(-scale, -scale * aspect);
            let size = Vec2::new(scale * 2.0, scale * 2.0 * aspect);
            {
                let mut batch = batch.borrow_mut();
                batch.draw_image(pos.x(), pos.y(), size.x(), size.y(), &logo, WHITE.0);

                let progress_bar_pos = Vec2::new(
                    -progress_bar_length / 2.0,
                    -scale * aspect - distance_between_bar_and_logo - progress_bar_height,
                );
                let progress_bar_size = Vec2::new(progress_bar_length, progress_bar_height);
                batch.draw_rect(
                    progress_bar_pos.x(),
                    progress_bar_pos.y(),
                    progress_bar_size.x(),
                    progress_bar_size.y(),
                    WHITE.0,
                );
                if let Some(progress) = progress {
                    batch.draw_rect(
                        progress_bar_pos.x() + progress_bar_padding,
                        progress_bar_pos.y() + progress_bar_padding,
                        (progress_bar_size.x() - progress_bar_padding * 2.0)
                            * f32::clamp(progress, 0.0, 1.0),
                        progress_bar_size.y() - progress_bar_padding * 2.0,
                        BLACK.0,
                    );
                } else {
                    // We draw a windows style progress bar with a moving white bit
                    let elapsed = env.start_time.elapsed();
                    let progress = elapsed.as_secs_f32() * 1.5;
                    let progress = (progress % 1.4) - 0.4;
                    let true_length = progress_bar_length - progress_bar_padding * 2.0;
                    let base_length = true_length * 0.3;
                    let base_start_x = f32::clamp(true_length * progress, 0.0, true_length);
                    let base_end_x =
                        f32::clamp(true_length * progress + base_length, 0.0, true_length);
                    let moving_bit_width = base_end_x - base_start_x;

                    batch.draw_rect(
                        progress_bar_pos.x() + progress_bar_padding + base_start_x,
                        progress_bar_pos.y() + progress_bar_padding,
                        moving_bit_width,
                        progress_bar_size.y() - progress_bar_padding * 2.0,
                        BLACK.0,
                    );
                }
            }

            if let Some(loading_text) = loading_text {
                let text_size = 0.16;
                let gl = batch.borrow().drawing_target.gl().clone();
                use_default_font(&gl, |font_renderer| {
                    let mut batch = batch.borrow_mut();
                    let (width, _height, _max_ascent) =
                        font_renderer.measure_text(&loading_text, text_size, aspect);
                    batch.draw_text(
                        -width / 2.0,
                        -1.8 * scale * aspect,
                        &loading_text,
                        WHITE.0,
                        text_size,
                        font_renderer,
                    );
                });
            }
        }
    };

    add_fn_to_table(lua, &graphics_module, "drawSplashScreen", {
        let draw_splash_screen = get_draw_splash_screen_fn();
        move |_, (loading_text, progress): (Option<String>, Option<f32>)| {
            draw_splash_screen(loading_text, progress);
            Ok(())
        }
    });

    add_fn_to_table(lua, &graphics_module, "drawSplashScreenIfNeeded", {
        let draw_splash_screen = get_draw_splash_screen_fn();
        move |_, (resources_to_wait_for, loading_text): (vectarine_plugin_sdk::mlua::Table, Option<String>)| {
            let ready_count = resources_to_wait_for
                .pairs::<vectarine_plugin_sdk::mlua::Value, vectarine_plugin_sdk::mlua::Value>()
                .map(|res| {
                    let Ok(res) = res else {
                        return false;
                    };
                    let Some(res) = res.1.as_userdata() else {
                        return false;
                    };
                    let is_ready_fn = res.get::<vectarine_plugin_sdk::mlua::Function>("isReady");
                    let Ok(is_ready_fn) = is_ready_fn else {
                        return false;
                    };
                    let Ok(is_ready) = is_ready_fn.call::<bool>(res) else {
                        return false;
                    };
                    is_ready
                })
                .fold(0i64, |acc, is_ready| acc + (is_ready as i64));

            // Depending on the platform, len can be a i32 or i64
            #[allow(clippy::useless_conversion)]
            let len = i64::from(resources_to_wait_for.len()?);
            let not_ready = len > ready_count;
            if not_ready {
                draw_splash_screen(loading_text, Some(ready_count as f32 / len as f32));
            }
            Ok(not_ready)
        }
    });

    Ok(graphics_module)
}
