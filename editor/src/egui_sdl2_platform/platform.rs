use egui::{Modifiers, Pos2};
use runtime::anyhow;
use runtime::sdl2;
use runtime::sdl2::{
    event::{Event, WindowEvent},
    mouse::{Cursor, MouseButton, SystemCursor},
};

use crate::egui_sdl2_platform::conversions::ToEguiKey;

/// The sdl2 platform for egui
pub struct Platform {
    // The cursors for the platform
    cursor: Option<Cursor>,
    system_cursor: SystemCursor,
    // The position of the mouse pointer
    pointer_pos: Pos2,
    // The egui modifiers
    modifiers: Modifiers,
    // The raw input
    raw_input: egui::RawInput,

    // The egui context
    egui_ctx: egui::Context,
}

impl Platform {
    /// Construct a new [`Platform`]
    pub fn new(screen_size: (u32, u32)) -> anyhow::Result<Self> {
        Ok(Self {
            cursor: Cursor::from_system(SystemCursor::Arrow).ok(),
            system_cursor: SystemCursor::Arrow,
            pointer_pos: Pos2::ZERO,
            raw_input: egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::Vec2 {
                        x: screen_size.0 as f32,
                        y: screen_size.1 as f32,
                    },
                )),
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            egui_ctx: egui::Context::default(),
        })
    }

    /// Handle a sdl2 event
    pub fn handle_event(&mut self, event: &Event, sdl: &sdl2::Sdl, video: &sdl2::VideoSubsystem) {
        #[allow(clippy::collapsible_match)]
        match event {
            // Handle reizing
            Event::Window { win_event, .. } => match win_event {
                WindowEvent::Resized(w, h) | WindowEvent::SizeChanged(w, h) => {
                    self.raw_input.screen_rect = Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::Vec2 {
                            x: *w as f32,
                            y: *h as f32,
                        },
                    ));
                }
                _ => {}
            },

            // Handle the mouse button being held down
            Event::MouseButtonDown { mouse_btn, .. } => {
                let btn = match mouse_btn {
                    MouseButton::Left => Some(egui::PointerButton::Primary),
                    MouseButton::Middle => Some(egui::PointerButton::Middle),
                    MouseButton::Right => Some(egui::PointerButton::Secondary),
                    _ => None,
                };
                if let Some(btn) = btn {
                    self.raw_input.events.push(egui::Event::PointerButton {
                        pos: self.pointer_pos,
                        button: btn,
                        pressed: true,
                        modifiers: self.modifiers,
                    });
                }
                self.egui_ctx.wants_pointer_input();
            }
            // Handle the mouse button being released
            Event::MouseButtonUp { mouse_btn, .. } => {
                let btn = match mouse_btn {
                    MouseButton::Left => Some(egui::PointerButton::Primary),
                    MouseButton::Middle => Some(egui::PointerButton::Middle),
                    MouseButton::Right => Some(egui::PointerButton::Secondary),
                    _ => None,
                };
                if let Some(btn) = btn {
                    self.raw_input.events.push(egui::Event::PointerButton {
                        pos: self.pointer_pos,
                        button: btn,
                        pressed: false,
                        modifiers: self.modifiers,
                    });
                }
                self.egui_ctx.wants_pointer_input();
            }
            // Handle mouse motion
            Event::MouseMotion { x, y, .. } => {
                // Update the pointer position
                self.pointer_pos = egui::Pos2::new(*x as f32, *y as f32);
                self.raw_input
                    .events
                    .push(egui::Event::PointerMoved(self.pointer_pos));
                self.egui_ctx.wants_pointer_input();
            }
            // Handle the mouse scrolling
            Event::MouseWheel { x, y, .. } => {
                // Calculate the delta
                let delta = egui::Vec2::new(*x as f32, *y as f32);
                // Check the mod state
                use sdl2::keyboard::Mod;
                let left_ctrl = sdl.keyboard().mod_state() & Mod::LCTRLMOD == Mod::LCTRLMOD;
                let right_ctrl = sdl.keyboard().mod_state() & Mod::RCTRLMOD == Mod::RCTRLMOD;

                // Push the egui event
                if left_ctrl || right_ctrl {
                    self.raw_input
                        .events
                        .push(egui::Event::Zoom((delta.y * 8.0 / 125.0).exp()));
                } else {
                    self.raw_input.events.push(egui::Event::MouseWheel {
                        delta,
                        modifiers: self.modifiers,
                        unit: egui::MouseWheelUnit::Line,
                    });
                }
                self.egui_ctx.wants_pointer_input();
            }

            // Handle a key being pressed
            Event::KeyDown {
                keycode, keymod, ..
            } => {
                // Make sure there is a keycode
                if let Some(keycode) = keycode {
                    // Convert the keycode to an egui key
                    if let Some(key) = keycode.to_egui_key() {
                        // Check the modifiers
                        use sdl2::keyboard::Mod;
                        let alt = (*keymod & Mod::LALTMOD == Mod::LALTMOD)
                            || (*keymod & Mod::RALTMOD == Mod::RALTMOD);
                        let ctrl = (*keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                            || (*keymod & Mod::RCTRLMOD == Mod::RCTRLMOD);
                        let shift = (*keymod & Mod::LSHIFTMOD == Mod::LSHIFTMOD)
                            || (*keymod & Mod::RSHIFTMOD == Mod::RSHIFTMOD);
                        let mac_cmd = *keymod & Mod::LGUIMOD == Mod::LGUIMOD;
                        let command = (*keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                            || (*keymod & Mod::LGUIMOD == Mod::LGUIMOD);

                        // Handle Cut Copy and paste
                        match key {
                            egui::Key::C => self.raw_input.events.push(egui::Event::Copy),
                            egui::Key::X => self.raw_input.events.push(egui::Event::Cut),
                            egui::Key::V => {
                                let clipboard = video.clipboard();
                                if clipboard.has_clipboard_text() {
                                    self.raw_input.events.push(egui::Event::Text(
                                        clipboard.clipboard_text().unwrap(),
                                    ));
                                }
                            }
                            _ => {}
                        }

                        // Update the modifiers
                        self.modifiers = Modifiers {
                            alt,
                            ctrl,
                            shift,
                            mac_cmd,
                            command,
                        };
                        self.raw_input.modifiers = self.modifiers;
                        // Push the event
                        self.raw_input.events.push(egui::Event::Key {
                            key,
                            physical_key: Some(key),
                            pressed: true,
                            repeat: false,
                            modifiers: self.modifiers,
                        });
                    }
                }
                self.egui_ctx.wants_keyboard_input();
            }
            // Handle a key being released
            Event::KeyUp {
                keycode, keymod, ..
            } => {
                // Make sure there is a keycode
                if let Some(keycode) = keycode {
                    // Convert the keycode to an egui key
                    if let Some(key) = keycode.to_egui_key() {
                        // Check the modifiers
                        use sdl2::keyboard::Mod;
                        let alt = (*keymod & Mod::LALTMOD == Mod::LALTMOD)
                            || (*keymod & Mod::RALTMOD == Mod::RALTMOD);
                        let ctrl = (*keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                            || (*keymod & Mod::RCTRLMOD == Mod::RCTRLMOD);
                        let shift = (*keymod & Mod::LSHIFTMOD == Mod::LSHIFTMOD)
                            || (*keymod & Mod::RSHIFTMOD == Mod::RSHIFTMOD);
                        let mac_cmd = *keymod & Mod::LGUIMOD == Mod::LGUIMOD;
                        let command = (*keymod & Mod::LCTRLMOD == Mod::LCTRLMOD)
                            || (*keymod & Mod::LGUIMOD == Mod::LGUIMOD);

                        // Update the modifiers
                        self.modifiers = Modifiers {
                            alt,
                            ctrl,
                            shift,
                            mac_cmd,
                            command,
                        };
                        self.raw_input.modifiers = self.modifiers;
                        // Push the event
                        self.raw_input.events.push(egui::Event::Key {
                            key,
                            physical_key: Some(key),
                            pressed: false,
                            repeat: false,
                            modifiers: self.modifiers,
                        });
                    }
                }
                self.egui_ctx.wants_keyboard_input();
            }
            // Handle text input
            Event::TextInput { text, .. } => {
                self.raw_input.events.push(egui::Event::Text(text.clone()));
                self.egui_ctx.wants_keyboard_input();
            }

            _ => {}
        }
    }

    /// Set the pixels per point
    pub fn set_pixels_per_point(&mut self, pixels_per_point: f32) {
        self.context().set_pixels_per_point(pixels_per_point);
    }

    /// Update the time
    pub fn update_time(&mut self, duration: f64) {
        self.raw_input.time = Some(duration);
    }

    /// Return the processed context
    pub fn context(&mut self) -> egui::Context {
        // Begin the frame
        self.egui_ctx.begin_pass(self.raw_input.take());
        // Return the ctx
        self.egui_ctx.clone()
    }

    /// Stop drawing the egui frame and return the full output
    pub fn end_frame(
        &mut self,
        video: &mut sdl2::VideoSubsystem,
    ) -> anyhow::Result<egui::FullOutput> {
        // Get the egui output
        let output = self.egui_ctx.end_pass();
        // Update the clipboard
        for cmd in &output.platform_output.commands {
            match cmd {
                egui::OutputCommand::CopyText(text) => {
                    video.clipboard().set_clipboard_text(text).map_err(|e| {
                        anyhow::anyhow!("Failed to assign text to clipboard: {}", e)
                    })?;
                }
                egui::OutputCommand::CopyImage(_) | egui::OutputCommand::OpenUrl(_) => {
                    // TODO: Handle CopyImage and OpenUrl commands
                }
            }
        }

        if let Some(cursor) = &mut self.cursor {
            // Update the cursor icon
            let new_cursor = match output.platform_output.cursor_icon {
                egui::CursorIcon::Crosshair => SystemCursor::Crosshair,
                egui::CursorIcon::Default => SystemCursor::Arrow,
                egui::CursorIcon::Grab => SystemCursor::Hand,
                egui::CursorIcon::Grabbing => SystemCursor::SizeAll,
                egui::CursorIcon::Move => SystemCursor::SizeAll,
                egui::CursorIcon::PointingHand => SystemCursor::Hand,
                egui::CursorIcon::ResizeHorizontal => SystemCursor::SizeWE,
                egui::CursorIcon::ResizeNeSw => SystemCursor::SizeNESW,
                egui::CursorIcon::ResizeNwSe => SystemCursor::SizeNWSE,
                egui::CursorIcon::ResizeVertical => SystemCursor::SizeNS,
                egui::CursorIcon::Text => SystemCursor::IBeam,
                egui::CursorIcon::NotAllowed | egui::CursorIcon::NoDrop => SystemCursor::No,
                egui::CursorIcon::Wait => SystemCursor::Wait,
                _ => SystemCursor::Arrow,
            };

            if self.system_cursor != new_cursor {
                self.system_cursor = new_cursor;
                *cursor = Cursor::from_system(new_cursor).map_err(|e| {
                    anyhow::anyhow!("Failed to get cursor from systems cursor: {}", e)
                })?;
                cursor.set();
            }
        }

        Ok(output)
    }

    /// Tessellate the egui frame
    pub fn tessellate(&self, full_output: &egui::FullOutput) -> Vec<egui::ClippedPrimitive> {
        self.egui_ctx
            .tessellate(full_output.shapes.clone(), self.egui_ctx.pixels_per_point())
    }
}
