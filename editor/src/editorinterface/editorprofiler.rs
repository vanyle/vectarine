use crate::editorinterface::EditorState;
use egui::RichText;
use runtime::metrics::{METRICS_STORAGE_DURATION, Measurable, Metric};
use std::cell::{Cell, RefCell};

const AVERAGE_SMOOTHING_WINDOW_SIZE: usize = 5;
const Y_SCALE_SMOOTHING_FACTOR: f32 = 0.05;

pub fn draw_editor_profiler(editor: &mut EditorState, ctx: &egui::Context) {
    let mut is_shown = editor.config.borrow().is_profiler_window_shown;

    egui::Window::new("Profiler")
        .default_width(400.0)
        .default_height(200.0)
        .open(&mut is_shown)
        .show(ctx, |ui| {
            let mut project = editor.project.borrow_mut();
            let project = project.as_mut();
            let Some(project) = project else {
                ui.label("No project opened to profile");
                return;
            };
            let metrics = &project.game.metrics_holder;
            let metrics_ref = metrics.borrow();

            egui::ScrollArea::vertical().show(ui, |ui| {
                let colors = [
                    egui::Color32::from_rgb(255, 100, 100),
                    egui::Color32::from_rgb(100, 255, 100),
                    egui::Color32::from_rgb(100, 100, 255),
                    egui::Color32::from_rgb(255, 255, 100),
                    egui::Color32::from_rgb(100, 255, 255),
                    egui::Color32::from_rgb(255, 100, 255),
                    egui::Color32::WHITE,
                ];

                ui.heading("Timed")
                    .on_hover_text("
Show the times taken by various operations during a frame. By default, the total frame time and the time
spent executing Lua are shown, but you can add your own metrics using Debug.timed.
".trim());

                thread_local! {
                    static DURATION_SEARCH: RefCell<String> = const { RefCell::new(String::new()) };
                }

                let search_filter = DURATION_SEARCH.with_borrow(|s| s.to_lowercase());

                DURATION_SEARCH.with_borrow_mut(|search_content| {
                    ui.horizontal(|ui| {
                        egui::TextEdit::singleline(search_content)
                            .hint_text("Filter times by name")
                            .desired_width(200.0)
                            .show(ui);
                    });
                });

                thread_local! {
                    static DURATION_GRAPH_Y_SCALE: Cell<f32> = const { Cell::new(0.0) };
                }

                let filtered_metrics: Vec<_> = metrics_ref
                    .get_duration_metrics()
                    .filter(|m| {
                        search_filter.is_empty() || m.name().to_lowercase().contains(&search_filter)
                    })
                    .collect();

                ui.horizontal_wrapped(|ui| {
                    for (i, metric) in filtered_metrics.iter().enumerate() {
                        let color = colors[i % colors.len()];
                        ui.label(
                            RichText::new(format!(
                                "{}: {:.2}ms",
                                metric.name(),
                                metric.recent_avg(AVERAGE_SMOOTHING_WINDOW_SIZE).into_f32()
                            ))
                            .color(color),
                        );
                    }
                });

                let (response, painter) = setup_drawing_area(ui, 100.0);
                let current_max = filtered_metrics
                    .iter()
                    .map(|m| m.max().into_f32())
                    .reduce(f32::max)
                    .unwrap_or(0.0);

                let max_val = DURATION_GRAPH_Y_SCALE.with(|scale| {
                    let prev_scale = scale.get();
                    let new_scale =
                        prev_scale + (current_max - prev_scale) * Y_SCALE_SMOOTHING_FACTOR;
                    scale.set(new_scale);
                    new_scale.max(0.1) // Ensure we never have zero scale
                });

                for (i, metric) in filtered_metrics.iter().enumerate() {
                    let color = colors[i % colors.len()];
                    draw_graph_impl(
                        &response,
                        &painter,
                        metric,
                        color,
                        max_val,
                        metric.frames_since_addition(),
                    );
                }

                ui.separator();

                ui.heading("Metrics");
                for metric in metrics_ref.get_numeric_metrics() {
                    draw_metric_graph(ui, metric, "");
                    ui.separator();
                }
            });
        });
    editor.config.borrow_mut().is_profiler_window_shown = is_shown;
}

fn draw_metric_graph<T: Measurable>(ui: &mut egui::Ui, metric: &Metric<T>, unit: &str) {
    ui.label(format!(
        "{}: {:.2}{}",
        metric.name(),
        metric.recent_avg(AVERAGE_SMOOTHING_WINDOW_SIZE).into_f32(),
        unit
    ));
    let max_val = metric.max().into_f32();
    let (response, painter) = setup_drawing_area(ui, 100.0);
    draw_graph_impl(
        &response,
        &painter,
        metric,
        egui::Color32::WHITE,
        max_val,
        metric.frames_since_addition(),
    );
}

fn setup_drawing_area(ui: &mut egui::Ui, height: f32) -> (egui::Response, egui::Painter) {
    let available_width = ui.available_width();
    let (response, painter) =
        ui.allocate_painter(egui::vec2(available_width, height), egui::Sense::hover());
    painter.rect_filled(response.rect, 0.0, egui::Color32::from_black_alpha(20));
    (response, painter)
}

fn draw_graph_impl<T: Measurable>(
    response: &egui::Response,
    painter: &egui::Painter,
    metric: &Metric<T>,
    color: egui::Color32,
    max_val: f32,
    frames_since_addition: usize,
) {
    let rect = response.rect;

    let points: Vec<egui::Pos2> = metric
        .smoothed_values(AVERAGE_SMOOTHING_WINDOW_SIZE)
        .enumerate()
        .map(|(i, val)| {
            let len = metric
                .samples()
                .saturating_sub(AVERAGE_SMOOTHING_WINDOW_SIZE);
            if len == 0 {
                return egui::pos2(rect.min.x, rect.max.y);
            }

            let frames_ago = frames_since_addition + (len - 1 - i);
            let x_fraction = 1.0 - (frames_ago as f32 / METRICS_STORAGE_DURATION as f32);

            let x = rect.min.x + x_fraction.clamp(0.0, 1.0) * rect.width();
            let y = rect.max.y - (val.into_f32() / max_val) * rect.height();
            egui::pos2(x, y)
        })
        .collect();

    painter.add(egui::Shape::line(points, egui::Stroke::new(1.0, color)));
}
