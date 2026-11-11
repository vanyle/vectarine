// Functions meant to deal with the case where the editor is separated from the game window.

use runtime::{
    anyhow,
    drawing_surface::{DrawingSurface, sdl_drawing_surface::drawable_screen_size},
    egui_wgpu,
    game_resource::{ResourceManager, font_resource},
    graphics::{batchdraw::BatchDraw2d, gfx::Gfx, surface::WindowSurface},
    sdl2::{
        event::{Event, WindowEvent},
        video::Window,
    },
};

use crate::{
    editorinterface::{EditorState, clear_window},
    egui_sdl2_platform,
};

/// What is needed to show the editor in its own window.
pub struct EditorWindowInterface {
    pub platform: egui_sdl2_platform::Platform,
    pub painter: egui_wgpu::Renderer,
    pub surface: WindowSurface,
}

/// Create an SDL2 Window to display the editor without the game.
/// This window is hidden by default and is show when the WindowStyle is set to GameSeparateFromEditor.
pub fn create_specific_editor_window(
    video_subsystem: &runtime::sdl2::VideoSubsystem,
    gfx: &Gfx,
) -> (Window, EditorWindowInterface) {
    let editor_window: Window = video_subsystem
        .window("Vectarine Editor", 700, 500)
        .allow_highdpi() // For Retina displays on macOS
        .resizable()
        .hidden()
        .metal_view()
        .build()
        .expect("Failed to create window");
    let interface =
        EditorWindowInterface::new(&editor_window, gfx).expect("Failed to create editor interface");
    (editor_window, interface)
}

impl EditorWindowInterface {
    pub fn new(window: &Window, gfx: &Gfx) -> anyhow::Result<Self> {
        let (width, height) = drawable_screen_size(window);
        // Safety: the window lives until the editor exits.
        let surface = unsafe { WindowSurface::for_window(gfx.gpu(), window, width, height) }
            .map_err(|e| anyhow::anyhow!(e))?;
        let painter = egui_wgpu::Renderer::new(
            &gfx.gpu().device,
            surface.format(),
            egui_wgpu::RendererOptions::default(),
        );
        let platform =
            egui_sdl2_platform::Platform::new(window.size()).expect("Failed to create platform");

        Ok(Self {
            platform,
            painter,
            surface,
        })
    }
}

pub fn render_editor_in_extra_window(
    sdl: &runtime::sdl2::Sdl,
    video: &runtime::sdl2::VideoSubsystem,
    gfx: &Gfx,
    editor_state: &mut EditorState,
    editor_interface: &mut EditorWindowInterface,
    editor_window_events: &[runtime::sdl2::event::Event],
) -> bool {
    editor_state
        .editor_specific_window_surface
        .borrow_mut()
        .window
        .show();

    let (width, height) =
        drawable_screen_size(&editor_state.editor_specific_window_surface.borrow().window);
    let is_visible = editor_interface.surface.begin_frame(gfx, width, height);
    clear_window(gfx);

    let aspect_ratio = width as f32 / height as f32;
    editor_state
        .editor_batch_draw
        .set_aspect_ratio(aspect_ratio);

    if editor_state.project.borrow().is_some() {
        let text = "This is the editor\nYour game is in another window\n\nUse preferences to merge or split\nthe editor and game windows.";
        let font_size = 0.13;
        draw_centered_text(
            gfx,
            &mut editor_state.editor_batch_draw,
            aspect_ratio,
            text,
            font_size,
        );
    }
    gfx.apply_pending_clear();

    let target = editor_interface.surface.frame_view();
    editor_state.draw_editor_interface(
        &mut editor_interface.platform,
        sdl,
        video,
        editor_window_events,
        &mut editor_interface.painter,
        target.as_ref(),
    );
    editor_interface.surface.end_frame(gfx);
    is_visible
}

pub fn draw_info_in_empty_game_window(
    gfx: &Gfx,
    game_window: &Window,
    batch_draw: &mut BatchDraw2d,
) {
    let (width, height) = game_window.size();
    let aspect_ratio = width as f32 / height as f32;
    batch_draw.set_aspect_ratio(aspect_ratio);

    let text = "No game is loaded\nUse the editor window\nto load or create a game.";
    let font_size = 0.13;
    draw_centered_text(gfx, batch_draw, aspect_ratio, text, font_size);
}

pub fn draw_error_in_game_window(
    gfx: &Gfx,
    game_window: &dyn DrawingSurface,
    batch_draw: &mut BatchDraw2d,
    error: &crate::luau::InfiniteLoopError,
) {
    let (width, height) = game_window.get_drawable_size_in_hardware_px();
    let aspect_ratio = width as f32 / height as f32;
    batch_draw.set_aspect_ratio(aspect_ratio);

    let title = "Abnormally long frame";
    let location = format!("{}:{}", error.file, error.line);
    let hint = "You might have an infinite loop in your code.";

    font_resource::use_default_font(gfx, |font_data| {
        let dummy_manager = ResourceManager::dummy_manager();

        let title_size = 0.20;
        let detail_size = 0.08;

        let (title_width, _, _) = font_data.measure_text(title, title_size, aspect_ratio);
        batch_draw.draw_text(
            -title_width / 2.0,
            0.3,
            title,
            [1.0, 0.2, 0.2, 1.0],
            title_size,
            font_data,
        );

        let (loc_width, _, _) = font_data.measure_text(&location, detail_size, aspect_ratio);
        batch_draw.draw_text(
            -loc_width / 2.0,
            0.05,
            &location,
            [1.0, 1.0, 1.0, 1.0],
            detail_size,
            font_data,
        );

        let (hint_width, _, _) = font_data.measure_text(hint, detail_size, aspect_ratio);
        batch_draw.draw_text(
            -hint_width / 2.0,
            -0.15,
            hint,
            [0.8, 0.8, 0.8, 1.0],
            detail_size,
            font_data,
        );

        batch_draw.draw(&dummy_manager, true);
    });
}

pub fn send_window_resize_sync_event(
    sdl: &runtime::sdl2::Sdl,
    video: &runtime::sdl2::VideoSubsystem,
    window: &Window,
    platform: &mut egui_sdl2_platform::Platform,
    editor_surface: &dyn DrawingSurface,
) {
    let (width, height) = window.size();
    let event: Event = Event::Window {
        timestamp: 0,
        window_id: window.id(),
        win_event: WindowEvent::Resized(width as i32, height as i32),
    };
    platform.handle_events(&[event], sdl, editor_surface, video);
}

pub fn draw_centered_text(
    gfx: &Gfx,
    batch_draw: &mut BatchDraw2d,
    aspect_ratio: f32,
    text: &str,
    font_size: f32,
) {
    // Draw extras for the editor interface
    font_resource::use_default_font(gfx, |font_data| {
        let dummy_manager = ResourceManager::dummy_manager();

        for (i, line) in text.lines().enumerate() {
            let (width, _height, _max_ascent) =
                font_data.measure_text(line, font_size, aspect_ratio);
            batch_draw.draw_text(
                -width / 2.0,
                0.5 - (i as f32 * font_size),
                line,
                [1.0f32, 1.0, 1.0, 1.0],
                font_size,
                font_data,
            );
        }
        batch_draw.draw(&dummy_manager, true);
    });
}
