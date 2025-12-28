// Functions meant to deal with the case where the editor is separated from the game window.

use std::sync::Arc;

use glow::HasContext;
use runtime::{
    anyhow, egui_glow,
    game::drawable_screen_size,
    game_resource::{ResourceManager, font_resource},
    sdl2::video::{GLContext, Window},
};

use crate::{editorinterface::EditorState, egui_sdl2_platform};

pub struct EditorInterfaceWithGl {
    pub platform: egui_sdl2_platform::Platform,
    pub painter: egui_glow::Painter,
    pub gl: Arc<glow::Context>,
    pub dummy_manager: runtime::game_resource::ResourceManager,
}

/// Create an SDL2 Window to display the editor without the game.
/// This window is hidden by default and is show when the WindowStyle is set to GameSeparateFromEditor.
pub fn create_specific_editor_window(
    video_subsystem: &runtime::sdl2::VideoSubsystem,
    gl: &Arc<glow::Context>,
) -> (Window, EditorInterfaceWithGl) {
    let editor_window: Window = video_subsystem
        .window("Vectarine Editor", 700, 500)
        .opengl()
        .allow_highdpi() // For Retina displays on macOS
        .resizable()
        .hidden()
        .build()
        .expect("Failed to create window");
    let interface =
        EditorInterfaceWithGl::new(&editor_window, gl).expect("Failed to create editor interface");
    (editor_window, interface)
}

impl EditorInterfaceWithGl {
    pub fn new(window: &Window, gl: &Arc<glow::Context>) -> anyhow::Result<Self> {
        let painter =
            egui_glow::Painter::new(gl.clone(), "", None, true).expect("Failed to create painter");
        let platform = egui_sdl2_platform::Platform::new(drawable_screen_size(window))
            .expect("Failed to create platform");

        Ok(Self {
            platform,
            painter,
            dummy_manager: ResourceManager::dummy_manager(),
            gl: gl.clone(),
        })
    }
}

pub fn render_editor_in_extra_window(
    sdl: &runtime::sdl2::Sdl,
    gl: &Arc<glow::Context>,
    gl_context: &GLContext,
    editor_state: &mut EditorState,
    editor_interface: &mut EditorInterfaceWithGl,
    editor_window_events: &[runtime::sdl2::event::Event],
) {
    editor_state.editor_specific_window.show();

    editor_state
        .editor_specific_window
        .gl_make_current(gl_context)
        .expect("Failed to make context current");
    let (width, height) = drawable_screen_size(&editor_state.editor_specific_window);
    let aspect_ratio = width as f32 / height as f32;
    editor_state
        .editor_batch_draw
        .set_aspect_ratio(aspect_ratio);

    unsafe {
        gl.viewport(0, 0, width as i32, height as i32);
    }

    // Draw extras for the editor interface
    font_resource::use_default_font(gl, |font_data| {
        let text = "This is the editor\nYour game is in another window\n\nUse preferences to merge the editor\n and the game if you prefer.";
        let font_size = 0.13;

        for (i, line) in text.lines().enumerate() {
            let (width, _height, _max_ascent) =
                font_data.measure_text(line, font_size, aspect_ratio);
            editor_state.editor_batch_draw.draw_text(
                -width / 2.0,
                0.5 - (i as f32 * font_size),
                line,
                [1.0f32, 1.0, 1.0, 1.0],
                font_size,
                font_data,
            );
        }
    });
    editor_state
        .editor_batch_draw
        .draw(&editor_interface.dummy_manager, true);

    let platform = &mut editor_interface.platform;
    let painter = &mut editor_interface.painter;
    editor_state.draw_editor_interface(platform, sdl, editor_window_events, painter);
}
