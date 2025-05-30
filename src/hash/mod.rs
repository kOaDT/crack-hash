pub mod md5;
pub mod sha1;
pub mod sha256;

use crate::Hasher;
pub use md5::Md5Hasher;
pub use sha1::Sha1Hasher;
pub use sha256::Sha256Hasher;

/// Factory function to create a hasher based on the algorithm name
/// 
/// # Arguments
/// * `algo` - Algorithm name (case-insensitive): "md5", "sha1", "sha256"
/// 
/// # Returns
/// * `Some(Box<dyn Hasher>)` - The corresponding hasher implementation
/// * `None` - If the algorithm is not supported
/// 
/// # Examples
/// ```
/// use hash::get_hasher;
/// 
/// let hasher = get_hasher("md5").unwrap();
/// assert_eq!(hasher.name(), "MD5");
/// 
/// let hasher = get_hasher("SHA256").unwrap();
/// assert_eq!(hasher.name(), "SHA256");
/// 
/// assert!(get_hasher("unsupported").is_none());
/// ```
pub fn get_hasher(algo: &str) -> Option<Box<dyn Hasher>> {
    match algo.to_lowercase().as_str() {
        "md5" => Some(Box::new(Md5Hasher::new())),
        "sha1" => Some(Box::new(Sha1Hasher::new())),
        "sha256" => Some(Box::new(Sha256Hasher::new())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_hasher_md5() {
        let hasher = get_hasher("md5").unwrap();
        assert_eq!(hasher.name(), "MD5");
    }

    #[test]
    fn test_get_hasher_sha1() {
        let hasher = get_hasher("sha1").unwrap();
        assert_eq!(hasher.name(), "SHA1");
    }

    #[test]
    fn test_get_hasher_sha256() {
        let hasher = get_hasher("sha256").unwrap();
        assert_eq!(hasher.name(), "SHA256");
    }

    #[test]
    fn test_get_hasher_case_insensitive() {
        let hasher_upper = get_hasher("MD5").unwrap();
        let hasher_lower = get_hasher("md5").unwrap();
        let hasher_mixed = get_hasher("Md5").unwrap();
        
        assert_eq!(hasher_upper.name(), "MD5");
        assert_eq!(hasher_lower.name(), "MD5");
        assert_eq!(hasher_mixed.name(), "MD5");
    }

    #[test]
    fn test_get_hasher_unsupported() {
        assert!(get_hasher("unsupported").is_none());
        assert!(get_hasher("sha512").is_none());
        assert!(get_hasher("").is_none());
    }

    #[test]
    fn test_hasher_functionality() {
        let md5_hasher = get_hasher("md5").unwrap();
        let sha1_hasher = get_hasher("sha1").unwrap();
        let sha256_hasher = get_hasher("sha256").unwrap();

        // Test actual hashing functionality
        let input = "hello";
        
        assert_eq!(md5_hasher.hash(input), "5d41402abc4b2a76b9719d911017c592");
        assert_eq!(sha1_hasher.hash(input), "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d");
        assert_eq!(sha256_hasher.hash(input), "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
    }
} 