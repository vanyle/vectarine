use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PromiseId(u32);

impl Display for PromiseId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

struct AsyncRuntime {
    next_id: u32,
    promises: HashMap<PromiseId, Box<dyn FnOnce(String)>>,
}

impl AsyncRuntime {
    fn new() -> Self {
        Self {
            next_id: 1,
            promises: HashMap::new(),
        }
    }
}

// IMO I should probably put a mutex there if emscripten allows it.
static mut RUNTIME: Option<AsyncRuntime> = None;

pub fn init_runtime() {
    unsafe {
        RUNTIME = Some(AsyncRuntime::new());
    }
}

pub fn create_promise(on_fulfilled: Box<dyn FnOnce(String)>) -> PromiseId {
    unsafe {
        let maybe_runtime = &raw mut RUNTIME;
        let Some(Some(async_runtime)) = maybe_runtime.as_mut() else {
            println!("Runtime is not initialized.");
            return PromiseId(0);
        };
        let random_id = PromiseId(async_runtime.next_id);
        async_runtime.next_id += 1;
        async_runtime.promises.insert(random_id, on_fulfilled);
        random_id
    }
}

/// ## Safety
/// Calling this function from Rust is always unsafe.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fulfill_promise(id: u32, result: *const u8) {
    unsafe {
        let Some(Some(async_runtime)) = (&raw mut RUNTIME).as_mut() else {
            println!("Runtime is not initialized.");
            return;
        };
        let maybe_callback = async_runtime.promises.remove(&PromiseId(id));
        let Some(maybe_callback) = maybe_callback else {
            println!("Promise with ID {id} not found. Maybe it was already resolved? ");
            return;
        };
        let result_as_string = std::ffi::CStr::from_ptr(result as *const i8)
            .to_string_lossy()
            .into_owned();
        maybe_callback(result_as_string);
    }
}
