use crate::Hasher;
use sha2::{Sha256, Digest};

pub struct Sha256Hasher;

impl Hasher for Sha256Hasher {
    fn name(&self) -> &'static str {
        "SHA256"
    }

    fn hash(&self, input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}

impl Sha256Hasher {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Sha256Hasher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_name() {
        let hasher = Sha256Hasher::new();
        assert_eq!(hasher.name(), "SHA256");
    }

    #[test]
    fn test_sha256_hash() {
        let hasher = Sha256Hasher::new();
        let result = hasher.hash("hello");
        assert_eq!(result, "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
    }

    #[test]
    fn test_sha256_empty_string() {
        let hasher = Sha256Hasher::new();
        let result = hasher.hash("");
        assert_eq!(result, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    #[test]
    fn test_sha256_password_example() {
        let hasher = Sha256Hasher::new();
        let result = hasher.hash("password123");
        assert_eq!(result, "ef92b778bafe771e89245b89ecbc08a44a4e166c06659911881f383d4473e94f");
    }
} 