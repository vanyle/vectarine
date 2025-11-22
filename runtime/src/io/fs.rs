pub trait ReadOnlyFileSystem {
    fn read_file(&self, path: &str, callback: Box<dyn FnOnce(Option<Vec<u8>>)>);
}

pub trait FileSystem: ReadOnlyFileSystem {
    fn write_file(&self, path: &str, data: &[u8], callback: Box<dyn FnOnce(bool)>);
}

pub fn init_fs() {
    // Initialize IDBFS for persistent storage on Emscripten
    #[cfg(target_os = "emscripten")]
    {
        use emscripten_functions::emscripten::run_script;
        run_script(
            r#"
            try {
                Module.FS.mkdir("/data");
            } catch(e) {
                console.log("/data directory may already exist:", e);
            }
            Module.FS.mount(Module.FS.filesystems.IDBFS, {}, "/data");
            
            // Load persisted data from IndexedDB
            Module.FS.syncfs(true, function(err) {
                if (err) {
                    console.error("Failed to load persisted data from IndexedDB:", err);
                }
            });
        "#,
        );
    }
}
