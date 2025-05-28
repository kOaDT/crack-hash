use crate::Hasher;
use sha1::{Sha1, Digest};

pub struct Sha1Hasher;

impl Hasher for Sha1Hasher {
    fn name(&self) -> &'static str {
        "SHA1"
    }

    fn hash(&self, input: &str) -> String {
        let mut hasher = Sha1::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}

impl Sha1Hasher {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Sha1Hasher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1_name() {
        let hasher = Sha1Hasher::new();
        assert_eq!(hasher.name(), "SHA1");
    }

    #[test]
    fn test_sha1_hash() {
        let hasher = Sha1Hasher::new();
        let result = hasher.hash("hello");
        assert_eq!(result, "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d");
    }

    #[test]
    fn test_sha1_empty_string() {
        let hasher = Sha1Hasher::new();
        let result = hasher.hash("");
        assert_eq!(result, "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    }

    #[test]
    fn test_sha1_password_example() {
        let hasher = Sha1Hasher::new();
        let result = hasher.hash("password123");
        assert_eq!(result, "cbfdac6008f9cab4083784cbd1874f76618d2a97");
    }
} 