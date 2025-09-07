#[cfg(target_os = "emscripten")]
use emscripten_functions::emscripten;

/// Returns the content of the file at `path`
/// Depending on your platform, this function can query the file system or perform an HTTP request to get the content.
#[cfg(not(target_os = "emscripten"))]
pub fn read_file(filename: &str, callback: Box<dyn FnOnce(String)>) {
    use std::{
        fs::{self},
        path::Path,
    };

    let content = fs::read(Path::new(filename)).ok();
    let content = content
        .map(|c| String::from_utf8_lossy(&c).into_owned())
        .unwrap_or_default();
    callback(content);
}

#[cfg(target_os = "emscripten")]
pub fn read_file(filename: &str, callback: Box<dyn FnOnce(String)>) {
    use crate::maybelater;

    let _promise_id = maybelater::create_promise(callback);
    emscripten::run_script_string(format!("read_file_for_rust({_promise_id}, \"{filename}\")"));
}
