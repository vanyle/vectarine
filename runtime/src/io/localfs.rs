#[cfg(target_os = "emscripten")]
use std::cell::Cell;
#[cfg(target_os = "emscripten")]
use std::cell::RefCell;
#[cfg(target_os = "emscripten")]
use std::collections::HashMap;

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
        if path.is_relative() // Only perform this check for relative paths.
            && let Ok(canonical) = path.canonicalize()
        {
            let canonical_with_slash = canonical.to_string_lossy().replace("\\", "/");
            let ends_with = canonical_with_slash.ends_with(filename);
            if !ends_with {
                // Access might work on MacOS or Windows, but not on the web (path is case-sensitive + you might be accessing a file outside the bundle)
                // We fail on all platforms for consistency and to catch errors early.
                // TODO: It would be nice to also this kind of path issues in the editor instead of the runtime.
                #[cfg(debug_assertions)]
                {
                    println!(
                        "The path provided is not canonicalized correctly: {} instead of {}",
                        filename,
                        canonical.display(),
                    );
                }
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
        let result = fs::write(Path::new(path), data);
        callback(result.is_ok());
        #[cfg(debug_assertions)]
        {
            if let Err(e) = result {
                println!("Failed to write file: {}", e);
            }
        }
    }
}

#[cfg(target_os = "emscripten")]
type CallbackMap = HashMap<u32, Box<dyn FnOnce(Option<Vec<u8>>)>>;

// Safety: Javascript is single-threaded.
#[cfg(target_os = "emscripten")]
thread_local! {

    static JS_CALLBACK_POOL: RefCell<CallbackMap> = RefCell::new(HashMap::new());
    static NEXT_CALLBACK_ID: Cell<u32> = const { Cell::new(0) };
}

/// # Safety
/// Don't call this function, it's meant to be called from Javascript. The returned pointer is owned by Javascript, but allocated using Rust's allocator.
#[unsafe(no_mangle)]
#[cfg(target_os = "emscripten")]
pub extern "C" fn alloc_rust_buffer(size: usize) -> *mut u8 {
    let mut buf = vec![0u8; size];
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf); // Leak it so JS can fill it
    ptr
}

/// # Safety
/// Don't call this function, it's meant to be called from Javascript.
/// This function acquires ownership of the content pointer and is responsible for freeing it.
#[unsafe(no_mangle)]
#[cfg(target_os = "emscripten")]
pub unsafe extern "C" fn read_rust_callback_from_js(
    callback_id: u32,
    content_ptr: *mut u8,
    content_len: usize,
) {
    let content = if content_ptr.is_null() {
        None
    } else {
        // from_raw_parts takes ownership of content_ptr as per the documentation. The Vec will be responsible for freeing the memory when it's dropped.
        Some(unsafe { Vec::from_raw_parts(content_ptr, content_len, content_len) })
    };
    let callback = JS_CALLBACK_POOL.with_borrow_mut(|pool| pool.remove(&callback_id));

    if let Some(callback) = callback {
        callback(content);
    } else {
        println!("No callback found for id: {}", callback_id);
    }
}

#[cfg(target_os = "emscripten")]
impl ReadOnlyFileSystem for LocalFileSystem {
    fn read_file(&self, filename: &str, callback: Box<dyn FnOnce(Option<Vec<u8>>)>) {
        use emscripten_functions::emscripten;

        let callback_id = NEXT_CALLBACK_ID.with(|id_cell| {
            let id = id_cell.get();
            id_cell.set(id.wrapping_add(1));
            id
        });

        JS_CALLBACK_POOL.with_borrow_mut(|pool| {
            pool.insert(callback_id, callback);
        });

        emscripten::run_script_string(format!(
            "vectarine.read_file_for_rust({callback_id}, \"{filename}\")"
        ));
    }
}

#[cfg(target_os = "emscripten")]
impl FileSystem for LocalFileSystem {
    fn write_file(&self, _path: &str, _data: &[u8], callback: Box<dyn FnOnce(bool)>) {
        callback(false);
    }
}
