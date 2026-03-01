use std::{
    fs,
    io::{self, Read},
};

/// Represents the hash of a plugin
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn from(bytes: blake3::Hash) -> Self {
        Self(*bytes.as_bytes())
    }

    pub fn from_file(file_reader: &mut io::BufReader<fs::File>) -> Option<Self> {
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
}
