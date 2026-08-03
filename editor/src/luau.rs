use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::Rc,
    time::{Duration, Instant},
};

use runtime::mlua;

#[derive(Clone, Debug)]
pub struct InfiniteLoopError {
    pub file: String,
    pub line: usize,
}

const MAX_FRAMES_TO_TRACK: usize = 10;
#[derive(Debug)]
pub struct CrossFrameTimer {
    start_of_current_frame: Instant,
    durations_of_last_frames: VecDeque<Duration>,
}

impl CrossFrameTimer {
    // Needs to be called every frame to reset the timer.
    pub fn signal_frame_end(&mut self) {
        let last_frame_duration = self.start_of_current_frame.elapsed();
        self.durations_of_last_frames.push_back(last_frame_duration);
        if self.durations_of_last_frames.len() > MAX_FRAMES_TO_TRACK {
            self.durations_of_last_frames.pop_front();
        }
        self.start_of_current_frame = Instant::now();
    }
    pub fn is_abnormally_long_frames(&self) -> bool {
        let current_frame_duration = self.start_of_current_frame.elapsed();
        let recent_frames_duration =
            current_frame_duration + self.durations_of_last_frames.iter().sum::<Duration>();
        // If there is a 2 sec lag-spike, that's fine, but if the average over 10 frames is above 3 sec, that is super strange.
        recent_frames_duration.as_millis() > 3000 || current_frame_duration.as_millis() > 2000
    }
}

pub type HookTiming = Rc<RefCell<Option<CrossFrameTimer>>>;
pub type HookError = Rc<RefCell<Option<InfiniteLoopError>>>;

pub fn update_hook_timing(hook_timing: &HookTiming) {
    if let Some(timer) = hook_timing.borrow_mut().as_mut() {
        timer.signal_frame_end();
    } else {
        *hook_timing.borrow_mut() = Some(CrossFrameTimer {
            start_of_current_frame: Instant::now(),
            durations_of_last_frames: VecDeque::new(),
        });
    }
}

pub fn setup_luau_hooks(lua: &mlua::Lua) -> (HookTiming, HookError) {
    let timing_info: HookTiming = Rc::new(RefCell::new(None));
    let hook_error: HookError = Rc::new(RefCell::new(None));

    let timing_info_for_hook = timing_info.clone();
    let hook_error_for_hook = hook_error.clone();

    lua.set_interrupt(move |lua| {
        if timing_info_for_hook
            .borrow()
            .as_ref()
            .filter(|frame_timer| frame_timer.is_abnormally_long_frames())
            .is_some()
        {
            let mut file = "unknown".to_string();
            let mut line = 0usize;

            for level in 0..10 {
                let mut found = false;
                lua.inspect_stack(level, |debug| {
                    let source = debug.source();
                    if let Some(src) = source.short_src.or(source.source)
                        && !src.is_empty()
                        && src != "=[C]"
                    {
                        file = src.to_string();
                        line = debug.current_line().unwrap_or(0);
                        found = true;
                    }
                });
                if found {
                    break;
                }
            }

            *hook_error_for_hook.borrow_mut() = Some(InfiniteLoopError { file, line });

            return Err(mlua::Error::RuntimeError(
                "Abnormally long frame (more than 3 seconds). Stopping execution.".into(),
            ));
        }
        Ok(mlua::VmState::Continue)
    });

    (timing_info, hook_error)
}
