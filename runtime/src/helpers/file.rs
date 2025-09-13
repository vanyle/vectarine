/// Returns the content of the file at `path`
/// Depending on your platform, this function can query the file system or perform an HTTP request to get the content.
#[cfg(not(target_os = "emscripten"))]
pub fn read_file(filename: &str, callback: Box<dyn FnOnce(Vec<u8>)>) {
    use std::{
        fs::{self},
        path::Path,
    };

    let content = fs::read(Path::new(filename)).ok();
    callback(content.unwrap_or_default());
}

#[cfg(target_os = "emscripten")]
pub fn read_file(filename: &str, callback: Box<dyn FnOnce(Vec<u8>)>) {
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
                let s = content.as_bytes();
                callback(s);
            };
            ().into()
        }),
    );

    Val::call(&read_file_handle, "read_file_for_rust", &[&o]);

    //emscripten::run_script_string(format!("read_file_for_rust({_promise_id}, \"{filename}\")"));
}
