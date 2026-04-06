use std::{cell::RefCell, rc::Rc};

use crate::game_resource::ResourceManager;
use crate::game_resource::image_resource::ImageResource;
use crate::graphics::batchdraw::{self, make_rect};
use crate::io::IoEnvState;
use crate::lua_env::lua_image::ImageResourceId;
use crate::lua_env::lua_vec2::Vec2;
use crate::lua_env::lua_vec4::Vec4;
use vectarine_plugin_sdk::mlua;

use super::{EventState, VectarineWidget};

pub struct ImageWidget {
    pub size: Vec2,
    pub image_id: ImageResourceId,
    pub resources: Rc<ResourceManager>,
    pub preserve_aspect_ratio: bool,
    pub tint_fn: Option<mlua::Function>,
    pub nine_slicing: Option<f32>,
    pub event_state: EventState,
}

impl ImageWidget {
    fn get_tint(
        &self,
        lua: &mlua::Lua,
        current_state: &EventState,
        extra: &mlua::Value,
    ) -> mlua::Result<[f32; 4]> {
        if let Some(ref tint_fn) = self.tint_fn {
            let event_table = current_state
                .to_lua(lua)
                .expect("Conversion to table should never fail");
            let color = tint_fn.call::<Vec4>((event_table, extra.clone()))?;
            return Ok(color.0);
        }
        Ok([1.0, 1.0, 1.0, 1.0])
    }

    fn draw_nine_slice(
        &self,
        batch: &RefCell<batchdraw::BatchDraw2d>,
        io_env: &RefCell<IoEnvState>,
        slice_ratio: f32,
        color: [f32; 4],
        img_width: f32,
        img_height: f32,
    ) {
        let tex_resource = self.resources.get_by_id::<ImageResource>(self.image_id.0);
        let Ok(tex_resource) = tex_resource else {
            return;
        };
        let tex_borrow = tex_resource.texture.borrow();
        let Some(tex) = tex_borrow.as_ref() else {
            return;
        };

        let widget_w = self.size.x();
        let widget_h = self.size.y();

        let io = io_env.borrow();
        let window_ratio = io.window_width as f32 / io.window_height as f32;
        drop(io);

        let image_ratio = img_width / img_height;
        let sx = slice_ratio;
        let sy = slice_ratio;

        // Compute border sizes in widget-local coordinates, matching the Luau drawContainer logic
        let border_h = (sy * widget_h).min(sx * widget_w / image_ratio);
        let border_w = border_h * image_ratio / window_ratio;
        let bx = border_w / widget_w;
        let by = border_h / widget_h;

        // Widget-local corner positions (origin at -1,-1)
        let x0 = -1.0_f32;
        let x1 = -1.0 + widget_w * bx;
        let x2 = -1.0 + widget_w * (1.0 - bx);
        let x3 = -1.0 + widget_w;

        let y0 = -1.0_f32;
        let y1 = -1.0 + widget_h * by;
        let y2 = -1.0 + widget_h * (1.0 - by);
        let y3 = -1.0 + widget_h;

        // UV coordinates
        let su0: f32 = 0.0;
        let su1 = sx;
        let su2 = 1.0 - sx;

        let sv0: f32 = 0.0;
        let sv1 = sy;
        let sv2 = 1.0 - sy;

        let sw_left = sx;
        let sw_mid = su2 - su1;
        let sw_right = sx;
        let sh_top = sy;
        let sh_mid = sv2 - sv1;
        let sh_bottom = sy;

        let mut b = batch.borrow_mut();

        // Row 0 (screen bottom): y0 -> y1, source bottom (sv2)
        b.draw_image_part(
            make_rect(x0, y0, x1 - x0, y1 - y0),
            tex,
            Vec2::new(su0, sv2),
            Vec2::new(sw_left, sh_bottom),
            color,
        );
        b.draw_image_part(
            make_rect(x1, y0, x2 - x1, y1 - y0),
            tex,
            Vec2::new(su1, sv2),
            Vec2::new(sw_mid, sh_bottom),
            color,
        );
        b.draw_image_part(
            make_rect(x2, y0, x3 - x2, y1 - y0),
            tex,
            Vec2::new(su2, sv2),
            Vec2::new(sw_right, sh_bottom),
            color,
        );

        // Row 1 (screen middle): y1 -> y2, source middle (sv1)
        b.draw_image_part(
            make_rect(x0, y1, x1 - x0, y2 - y1),
            tex,
            Vec2::new(su0, sv1),
            Vec2::new(sw_left, sh_mid),
            color,
        );
        b.draw_image_part(
            make_rect(x1, y1, x2 - x1, y2 - y1),
            tex,
            Vec2::new(su1, sv1),
            Vec2::new(sw_mid, sh_mid),
            color,
        );
        b.draw_image_part(
            make_rect(x2, y1, x3 - x2, y2 - y1),
            tex,
            Vec2::new(su2, sv1),
            Vec2::new(sw_right, sh_mid),
            color,
        );

        // Row 2 (screen top): y2 -> y3, source top (sv0)
        b.draw_image_part(
            make_rect(x0, y2, x1 - x0, y3 - y2),
            tex,
            Vec2::new(su0, sv0),
            Vec2::new(sw_left, sh_top),
            color,
        );
        b.draw_image_part(
            make_rect(x1, y2, x2 - x1, y3 - y2),
            tex,
            Vec2::new(su1, sv0),
            Vec2::new(sw_mid, sh_top),
            color,
        );
        b.draw_image_part(
            make_rect(x2, y2, x3 - x2, y3 - y2),
            tex,
            Vec2::new(su2, sv0),
            Vec2::new(sw_right, sh_top),
            color,
        );
    }
}

impl VectarineWidget for ImageWidget {
    fn size(&self) -> Vec2 {
        self.size
    }

    fn draw(
        &mut self,
        lua: &mlua::Lua,
        batch: &RefCell<batchdraw::BatchDraw2d>,
        io_env: &RefCell<IoEnvState>,
        current_state: EventState,
        _process_child_events: bool,
        _draw_debug_outline: bool,
        extra: mlua::Value,
    ) -> mlua::Result<()> {
        let color = self.get_tint(lua, &current_state, &extra)?;

        let tex_resource = self.resources.get_by_id::<ImageResource>(self.image_id.0);
        let Ok(tex_resource) = tex_resource else {
            return Ok(());
        };
        let (img_w, img_h) = {
            let tex_borrow = tex_resource.texture.borrow();
            let Some(tex) = tex_borrow.as_ref() else {
                return Ok(());
            };
            (tex.width() as f32, tex.height() as f32)
        };

        // Nine-slice mode (only when not preserving aspect ratio)
        if let Some(slice_ratio) = self.nine_slicing
            && !self.preserve_aspect_ratio
        {
            self.draw_nine_slice(batch, io_env, slice_ratio, color, img_w, img_h);
            return Ok(());
        }

        let widget_w = self.size.x();
        let widget_h = self.size.y();

        let (draw_w, draw_h, draw_x, draw_y) = if self.preserve_aspect_ratio {
            let img_ratio = img_w / img_h;
            let io = io_env.borrow();
            let window_ratio = io.window_width as f32 / io.window_height as f32;
            drop(io);

            // The widget size is in screen-ratio coords, so we need to account for window ratio
            // when computing the image's aspect-correct size
            let widget_ratio = (widget_w * window_ratio) / widget_h;

            let (dw, dh) = if img_ratio > widget_ratio {
                // Image is wider relative to widget — fit to width
                let dw = widget_w;
                let dh = widget_w * window_ratio / img_ratio;
                (dw, dh)
            } else {
                // Image is taller relative to widget — fit to height
                let dh = widget_h;
                let dw = widget_h * img_ratio / window_ratio;
                (dw, dh)
            };

            let dx = -1.0 + (widget_w - dw) / 2.0;
            let dy = -1.0 + (widget_h - dh) / 2.0;
            (dw, dh, dx, dy)
        } else {
            (widget_w, widget_h, -1.0, -1.0)
        };

        let tex_borrow = tex_resource.texture.borrow();
        let Some(tex) = tex_borrow.as_ref() else {
            return Ok(());
        };
        batch
            .borrow_mut()
            .draw_image(draw_x, draw_y, draw_w, draw_h, tex, color);
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn VectarineWidget> {
        Box::new(ImageWidget {
            size: self.size,
            image_id: self.image_id,
            resources: self.resources.clone(),
            preserve_aspect_ratio: self.preserve_aspect_ratio,
            tint_fn: self.tint_fn.clone(),
            nine_slicing: self.nine_slicing,
            event_state: self.event_state.clone(),
        })
    }

    fn event_state_mut(&mut self) -> &mut EventState {
        &mut self.event_state
    }

    fn event_state(&self) -> &EventState {
        &self.event_state
    }

    fn debug_label(&self) -> String {
        "Image".to_string()
    }
}
