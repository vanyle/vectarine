use crate::io::fs::FileSystem;
use crate::io::fs::ReadOnlyFileSystem;

pub struct LocalFileSystem;

#[cfg(not(target_os = "emscripten"))]
impl ReadOnlyFileSystem for LocalFileSystem {
    /// Returns the content of the file at `path`
    /// Depending on your platform, this function can query the file system or perform an HTTP request to get the content.
    fn read_file(&self, filename: &str, callback: Box<dyn FnOnce(Option<Vec<u8>>)>) {
        use std::{
            fs::{self},
            path::Path,
        };

        let path = Path::new(filename);
        if let Ok(canonical) = path.canonicalize() {
            let ends_with = canonical.ends_with(path);
            if !ends_with {
                // Access might work on MacOS or Windows, but not on the web (path is case-sensitive + you might be accessing a file outside the bundle)
                // We fail on all platforms for consistency and to catch errors early.
                // TODO: log a warning
                callback(None);
                return;
            }
        }

        let content = fs::read(Path::new(filename)).ok();
        callback(content);
    }
}

#[cfg(not(target_os = "emscripten"))]
impl FileSystem for LocalFileSystem {
    fn write_file(&self, path: &str, data: &[u8], callback: Box<dyn FnOnce(bool)>) {
        use std::fs;
        use std::path::Path;
        let result = fs::write(Path::new(path), data).is_ok();
        callback(result);
    }
}

#[cfg(target_os = "emscripten")]
impl ReadOnlyFileSystem for LocalFileSystem {
    fn read_file(&self, filename: &str, callback: Box<dyn FnOnce(Option<Vec<u8>>)>) {
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
}

#[cfg(target_os = "emscripten")]
impl FileSystem for LocalFileSystem {
    fn write_file(&self, _path: &str, _data: &[u8], callback: Box<dyn FnOnce(bool)>) {
        callback(false);
    }
}
