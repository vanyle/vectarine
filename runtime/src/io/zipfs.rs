use std::cell::RefCell;
use std::io::{Cursor, Read};
use vectarine_plugin_sdk::anyhow::Result;
use zip::ZipArchive;

use crate::io::fs::ReadOnlyFileSystem;

pub struct ZipFileSystem {
    archive: RefCell<ZipArchive<Cursor<Vec<u8>>>>,
}

impl ZipFileSystem {
    /// Create a fake file system from a zip.
    pub fn new(zip_content: Vec<u8>) -> Result<Self> {
        let reader = Cursor::new(zip_content);
        let archive = zip::ZipArchive::new(reader)?;
        Ok(Self {
            archive: RefCell::new(archive),
        })
    }

    pub fn read_file_sync(&self, filename: &str) -> Option<Vec<u8>> {
        let mut archive = self.archive.borrow_mut();
        let mut file = archive.by_name(filename).ok()?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).ok()?;
        Some(contents)
    }
}

impl ReadOnlyFileSystem for ZipFileSystem {
    /// Returns the content of the file at `path`
    /// Reads the file from the zip archive and calls the callback with the file contents.
    fn read_file(&self, filename: &str, callback: Box<dyn FnOnce(Option<Vec<u8>>)>) {
        let mut archive = self.archive.borrow_mut();

        // Try to find and read the file from the zip archive
        let result = archive.by_name(filename).ok().and_then(|mut file| {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).ok()?;
            Some(contents)
        });
        callback(result);
    }
}
