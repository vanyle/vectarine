#[cfg(target_os = "emscripten")]
mod emscripten {
    use std::os::raw::c_uint;

    unsafe extern "C" {
        pub fn emscripten_get_now() -> f64;
        pub fn emscripten_sleep(ms: c_uint);
    }

    pub fn now_ms() -> f64 {
        unsafe { emscripten_get_now() }
    }

    pub fn sleep(ms: u32) {
        unsafe {
            emscripten_sleep(ms);
        }
    }
}

/// Returns a number which increases by 1 every millisecond. The number is relative to an arbitrary point in time,
/// so only the difference between two calls is meaningful.
pub fn now_ms() -> f64 {
    #[cfg(target_os = "emscripten")]
    {
        emscripten::now_ms()
    }
    #[cfg(not(target_os = "emscripten"))]
    {
        use std::time::Instant;
        lazy_static::lazy_static! {
            static ref START_INSTANT: Instant = Instant::now();
        }
        START_INSTANT.elapsed().as_micros() as f64 / 1000.0
    }
}

pub fn sleep(ms: u32) {
    #[cfg(target_os = "emscripten")]
    {
        emscripten::sleep(ms);
    }
    #[cfg(not(target_os = "emscripten"))]
    {
        std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    }
}
