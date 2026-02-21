/// Represents the hash of a plugin
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn from(bytes: blake3::Hash) -> Self {
        Self(*bytes.as_bytes())
    }
}
