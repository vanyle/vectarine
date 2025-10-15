/// Returns the content of the file at `path`
/// Depending on your platform, this function can query the file system or perform an HTTP request to get the content.
#[cfg(not(target_os = "emscripten"))]
pub fn read_file(filename: &str, callback: Box<dyn FnOnce(Option<Vec<u8>>)>) {
    use std::{
        fs::{self},
        path::Path,
    };

    let content = fs::read(Path::new(filename)).ok();
    callback(content);
}

/// Writes to the filename provided the data provided.
/// Returns true on success, false otherwise.
#[cfg(not(target_os = "emscripten"))]
pub fn write_file(filename: &str, data: &[u8]) -> bool {
    use std::{fs::File, io::Write};

    let file = File::create(filename);
    let Ok(mut file) = file else {
        return false;
    };
    file.write_all(data).is_ok()
}

#[cfg(target_os = "emscripten")]
pub fn write_file(_filename: &str, _data: &[u8]) -> bool {
    // Writing files is not supported on Emscripten.
    // In the future, we could probably make it work using a fake file-system.
    false
}

#[cfg(target_os = "emscripten")]
pub fn read_file(filename: &str, callback: Box<dyn FnOnce(Option<Vec<u8>>)>) {
    use base64::prelude::*;
    use emscripten_val::Val;

    let read_file_handle = Val::global("vectarine");
    let mut callback_option = Some(callback);
    let o = Val::object();
    o.set(&Val::from_str("filename"), &Val::from_str(filename));
    o.set(
        &Val::from_str("callback"),
        &Val::from_fn1(move |content: &Val| {
            // callback is FnOnce, we turn it into FnMut using this Option.
            if let Some(callback) = callback_option.take() {
                if content.is_false() {
                    callback(None);
                    return ().into();
                }
                let s = content.as_string();
                let decoded = BASE64_STANDARD.decode(&s).unwrap_or_default();
                callback(Some(decoded));
            };
            ().into()
        }),
    );

    Val::call(&read_file_handle, "read_file_for_rust", &[&o]);

    //emscripten::run_script_string(format!("read_file_for_rust({_promise_id}, \"{filename}\")"));
}
