use crate::io::fs::ReadOnlyFileSystem;

/// An empty file system with no files
pub struct DummyFileSystem;
impl ReadOnlyFileSystem for DummyFileSystem {
    fn read_file(&self, _path: &str, callback: Box<dyn FnOnce(Option<Vec<u8>>)>) {
        callback(None);
    }
}
