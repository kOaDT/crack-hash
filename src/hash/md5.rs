use crate::Hasher;

/// MD5 hasher implementation
pub struct Md5Hasher;

impl Hasher for Md5Hasher {
    fn name(&self) -> &'static str {
        "MD5"
    }

    fn hash(&self, input: &str) -> String {
        let digest = md5::compute(input.as_bytes());
        format!("{:x}", digest)
    }
}

impl Md5Hasher {
    /// Create a new MD5 hasher instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for Md5Hasher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5_name() {
        let hasher = Md5Hasher::new();
        assert_eq!(hasher.name(), "MD5");
    }

    #[test]
    fn test_md5_hash() {
        let hasher = Md5Hasher::new();
        let result = hasher.hash("hello");
        assert_eq!(result, "5d41402abc4b2a76b9719d911017c592");
    }

    #[test]
    fn test_md5_empty_string() {
        let hasher = Md5Hasher::new();
        let result = hasher.hash("");
        assert_eq!(result, "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn test_md5_password_example() {
        let hasher = Md5Hasher::new();
        let result = hasher.hash("password123");
        assert_eq!(result, "482c811da5d5b4bc6d497ffa98491e38");
    }
} 