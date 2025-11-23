use crate::editorinterface::EditorState;

pub fn draw_editor_profiler(editor: &mut EditorState, ctx: &egui::Context) {
    let mut is_shown = editor.config.borrow().is_profiler_window_shown;
    const AVERAGE_SMOOTHING_WINDOW_SIZE: usize = 5;

    egui::Window::new("Profiler")
        .default_width(400.0)
        .default_height(200.0)
        .open(&mut is_shown)
        .show(ctx, |ui| {
            let fps = 1.0 / ctx.input(|i| i.unstable_dt);

            thread_local! {
                static FPS_HISTORY: std::cell::RefCell<Vec<f32>> = const { std::cell::RefCell::new(Vec::new()) };
            }

            FPS_HISTORY.with(|history| {
                let mut history = history.borrow_mut();
                history.push(fps);
                if history.len() > 500 {
                    history.remove(0);
                }

                let smoothed_history: Vec<f32> = history
                    .iter()
                    .enumerate()
                    .skip(AVERAGE_SMOOTHING_WINDOW_SIZE - 1)
                    .map(|(i, _)| {
                        let start = i - (AVERAGE_SMOOTHING_WINDOW_SIZE - 1);
                        let slice = &history[start..=i];
                        slice.iter().sum::<f32>() / slice.len() as f32
                    })
                    .collect();

                if let Some(avg_fps) = smoothed_history.last() {
                    ui.label(format!("FPS: {:.0}", avg_fps));
                }

                let available_width = ui.available_width();
                let height = 100.0;
                let (response, painter) =
                    ui.allocate_painter(egui::vec2(available_width, height), egui::Sense::hover());

                let rect = response.rect;

                // Draw background
                painter.rect_filled(rect, 0.0, egui::Color32::from_black_alpha(20));

                if smoothed_history.len() < 2 {
                    return;
                }

                let max_fps = smoothed_history.iter().cloned().fold(60.0, f32::max);

                let points: Vec<egui::Pos2> = smoothed_history
                    .iter()
                    .enumerate()
                    .map(|(i, &val)| {
                        let x = rect.min.x + (i as f32 / 500.0) * rect.width();
                        let y = rect.max.y - (val / max_fps) * rect.height();
                        egui::pos2(x, y)
                    })
                    .collect();

                painter.add(egui::Shape::line(
                    points,
                    egui::Stroke::new(1.0, egui::Color32::GREEN),
                ));
                
                ui.label(format!("Max: {:.0}", max_fps));
            });
        });
    editor.config.borrow_mut().is_profiler_window_shown = is_shown;
}
