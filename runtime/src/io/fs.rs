pub trait ReadOnlyFileSystem {
    fn read_file(&self, path: &str, callback: Box<dyn FnOnce(Option<Vec<u8>>)>);
}

pub trait FileSystem: ReadOnlyFileSystem {
    fn write_file(&self, path: &str, data: &[u8], callback: Box<dyn FnOnce(bool)>);
}
