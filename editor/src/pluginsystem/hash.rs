use std::{
    fs,
    io::{self, Read},
    path::Path,
};

/// Represents the hash of a plugin
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn from(bytes: blake3::Hash) -> Self {
        Self(*bytes.as_bytes())
    }

    pub fn from_file<T: Read>(file_reader: &mut io::BufReader<T>) -> Option<Self> {
        let mut hasher = blake3::Hasher::new();

        // A buffer to hold chunks of the file
        let mut buffer = [0; 8192];
        loop {
            let count = file_reader.read(&mut buffer).ok()?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        Some(Self::from(hasher.finalize()))
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        let file = fs::File::open(path).ok()?;
        let mut reader = io::BufReader::new(file);
        Self::from_file(&mut reader)
    }
}
