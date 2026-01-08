use std::{cell::RefCell, rc::Rc, time::Instant};

use runtime::mlua;

#[derive(Clone, Debug)]
pub struct InfiniteLoopError {
    pub file: String,
    pub line: usize,
}

type HookTiming = Rc<RefCell<Option<Instant>>>;
type HookError = Rc<RefCell<Option<InfiniteLoopError>>>;

pub fn setup_luau_hooks(lua: &mlua::Lua) -> (HookTiming, HookError) {
    let frame_start_time: HookTiming = Rc::new(RefCell::new(None));
    let hook_error: HookError = Rc::new(RefCell::new(None));

    let frame_start_time_for_hook = frame_start_time.clone();
    let hook_error_for_hook = hook_error.clone();

    lua.set_interrupt(move |lua| {
        if frame_start_time_for_hook
            .borrow()
            .filter(|s| s.elapsed().as_millis() > 500)
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
                "Abnormally long frame (more than 500ms). Stopping execution.".into(),
            ));
        }
        Ok(mlua::VmState::Continue)
    });

    (frame_start_time, hook_error)
}
