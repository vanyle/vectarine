use std::{collections::VecDeque, time::Duration};

// For a 60 FPS game, we store metrics for about 6 seconds.
pub const METRICS_STORAGE_DURATION: usize = 60 * 6;

pub trait Measurable: Copy + Ord + Default + std::iter::Sum<Self> {
    fn div_count(self, count: usize) -> Self;
    fn into_f32(self) -> f32;
}
impl Measurable for std::time::Duration {
    fn div_count(self, count: usize) -> Self {
        self / (count as u32)
    }
    fn into_f32(self) -> f32 {
        self.as_secs_f32() * 1000.0
    }
}
impl Measurable for usize {
    fn div_count(self, count: usize) -> Self {
        self / count
    }
    fn into_f32(self) -> f32 {
        self as f32
    }
}

/// A Metric<T> is a value of type T that is recorded for each frame.
pub struct Metric<T: Copy> {
    name: String,
    values: VecDeque<T>,
    frames_since_addition: usize,
}

impl<T: Copy> Metric<T> {
    pub fn new(name: String) -> Self {
        Metric {
            name,
            values: VecDeque::new(),
            frames_since_addition: 0,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn samples(&self) -> usize {
        self.values.len()
    }

    pub fn frames_since_addition(&self) -> usize {
        self.frames_since_addition
    }

    pub fn avg(&self) -> T
    where
        T: Measurable + std::iter::Sum<T>,
    {
        self.values
            .iter()
            .copied()
            .sum::<T>()
            .div_count(self.values.len())
    }

    pub fn recent_avg(&self, recent_frame_samples: usize) -> T
    where
        T: Measurable + std::iter::Sum<T>,
    {
        self.values
            .iter()
            .skip(self.values.len().saturating_sub(recent_frame_samples))
            .copied()
            .sum::<T>()
            .div_count(recent_frame_samples)
    }

    pub fn max(&self) -> T
    where
        T: Ord + Default,
    {
        self.values.iter().copied().max().unwrap_or_default()
    }

    pub fn values(&self) -> impl Iterator<Item = T> + '_ {
        self.values.iter().copied()
    }

    pub fn smoothed_values(&self, smoothing_window: usize) -> impl Iterator<Item = T> + '_
    where
        T: Measurable + std::iter::Sum<T> + Default,
    {
        self.values
            .iter()
            .skip(smoothing_window)
            .enumerate()
            .map(move |(i, _)| {
                self.values
                    .iter()
                    .skip(i.saturating_sub(smoothing_window))
                    .take(smoothing_window)
                    .copied()
                    .sum::<T>()
                    .div_count(smoothing_window)
            })
    }
}

/// Keep elements that satisfy the predicate, remove others in place.
/// This method can change the order of elements.
fn swap_retain<T, F>(elements: &mut Vec<T>, mut predicate: F)
where
    F: FnMut(&T) -> bool,
{
    let mut i = 0;
    while i < elements.len() {
        if !predicate(&elements[i]) {
            elements.swap_remove(i);
        } else {
            i += 1;
        }
    }
}

/// Stores the metrics for all frames and produces stats about them.
pub struct MetricsHolder {
    duration_historical_metrics: Vec<Metric<Duration>>,
    number_historical_metrics: Vec<Metric<usize>>,
}

// Name of some default metrics.
pub const TOTAL_FRAME_TIME_METRIC_NAME: &str = "total_frame_time";
pub const DRAW_CALL_METRIC_NAME: &str = "draw_call";
pub const LUA_HEAP_SIZE_METRIC_NAME: &str = "lua_heap_size";
pub const LUA_SCRIPT_TIME_METRIC_NAME: &str = "total_lua_script_time";
// pub const ENGINE_FRAME_TIME_METRIC_NAME: &str = "engine_frame_time";

impl MetricsHolder {
    pub fn new() -> Self {
        MetricsHolder {
            duration_historical_metrics: Vec::new(),
            number_historical_metrics: Vec::new(),
        }
    }
    pub fn record_number_metric(&mut self, name: &str, value: usize) {
        let metric = self
            .number_historical_metrics
            .iter_mut()
            .find(|m| m.name == name);
        if let Some(metric) = metric {
            if metric.frames_since_addition == 0
                && let Some(last) = metric.values.back_mut()
            {
                *last += value;
            } else {
                metric.values.push_back(value);
                metric.frames_since_addition = 0;
            }
        } else {
            self.number_historical_metrics.push(Metric {
                name: name.to_string(),
                values: VecDeque::from([value]),
                frames_since_addition: 0,
            });
        }
    }
    pub fn record_duration_metric(&mut self, name: &str, value: Duration) {
        let metric = self
            .duration_historical_metrics
            .iter_mut()
            .find(|m| m.name == name);
        if let Some(metric) = metric {
            if metric.frames_since_addition == 0
                && let Some(last) = metric.values.back_mut()
            {
                *last += value;
            } else {
                metric.values.push_back(value);
                metric.frames_since_addition = 0;
            }
        } else {
            self.duration_historical_metrics.push(Metric {
                name: name.to_string(),
                values: VecDeque::from([value]),
                frames_since_addition: 0,
            });
        }
    }
    pub fn flush(&mut self) {
        for metric in &mut self.number_historical_metrics {
            let metric_has_too_many_values = metric.values.len() > METRICS_STORAGE_DURATION;
            let metric_is_outdated = metric.frames_since_addition > 0;
            if metric_has_too_many_values || metric_is_outdated {
                metric.values.pop_front();
            }
            metric.frames_since_addition += 1;
        }
        swap_retain(&mut self.number_historical_metrics, |m| {
            !m.values.is_empty()
        });

        for metric in &mut self.duration_historical_metrics {
            let metric_has_too_many_values = metric.values.len() > METRICS_STORAGE_DURATION;
            let metric_is_outdated = metric.frames_since_addition > 0;
            if metric_has_too_many_values || metric_is_outdated {
                metric.values.pop_front();
            }
            metric.frames_since_addition += 1;
        }
        swap_retain(&mut self.duration_historical_metrics, |m| {
            !m.values.is_empty()
        });
    }
    pub fn get_numeric_metric_by_name(&self, name: &str) -> Option<&Metric<usize>> {
        self.number_historical_metrics
            .iter()
            .find(|m| m.name == name)
    }
    pub fn get_duration_metric_by_name(&self, name: &str) -> Option<&Metric<Duration>> {
        self.duration_historical_metrics
            .iter()
            .find(|m| m.name == name)
    }

    pub fn get_numeric_metrics(&self) -> impl Iterator<Item = &Metric<usize>> {
        self.number_historical_metrics.iter()
    }
    pub fn get_duration_metrics(&self) -> impl Iterator<Item = &Metric<Duration>> {
        self.duration_historical_metrics.iter()
    }
}

impl Default for MetricsHolder {
    fn default() -> Self {
        Self::new()
    }
}
