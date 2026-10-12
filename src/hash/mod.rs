pub mod md5;
pub mod sha1;
pub mod sha256;

use crate::Hasher;
pub use md5::Md5Hasher;
pub use sha1::Sha1Hasher;
pub use sha256::Sha256Hasher;

/// Largest digest produced by a supported algorithm (SHA256).
pub const MAX_DIGEST_LEN: usize = 32;

/// Raw digest bytes held inline on the stack so the hot loop never allocates.
#[derive(Clone, Copy)]
pub struct Digest {
    bytes: [u8; MAX_DIGEST_LEN],
    len: usize,
}

impl Digest {
    pub fn new(raw: &[u8]) -> Self {
        let mut bytes = [0u8; MAX_DIGEST_LEN];
        bytes[..raw.len()].copy_from_slice(raw);
        Self {
            bytes,
            len: raw.len(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    pub fn to_hex(self) -> String {
        to_hex(self.as_bytes())
    }
}

pub fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(out, "{:02x}", byte);
    }
    out
}

/// Decode a hexadecimal string into bytes, accepting either case.
/// Returns `None` for odd length or any non-hex character.
pub fn decode_hex(hex: &str) -> Option<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }

    let (pairs, _) = hex.as_bytes().as_chunks::<2>();
    pairs
        .iter()
        .map(|pair| {
            let hi = (pair[0] as char).to_digit(16)?;
            let lo = (pair[1] as char).to_digit(16)?;
            Some(((hi << 4) | lo) as u8)
        })
        .collect()
}

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

/// Password-hash scheme a target string belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scheme {
    /// Bare hex digest (md5/sha1/sha256); the algorithm must be supplied.
    Raw,
    /// Modular Crypt Format (`$id$...`); `name` is a human-readable label.
    Crypt { name: &'static str },
}

/// Identify a target's scheme from its shape. A leading `$` marks Modular Crypt
/// Format, whose `$id$` selects the algorithm; anything else is a raw hex digest.
/// Returns `None` for an MCF string whose id is not recognized.
pub fn detect_scheme(target: &str) -> Option<Scheme> {
    if !target.starts_with('$') {
        return Some(Scheme::Raw);
    }

    let name = match target.split('$').nth(1)? {
        "1" => "md5crypt",
        "5" => "sha256crypt",
        "6" => "sha512crypt",
        "2a" | "2b" | "2y" => "bcrypt",
        "sha1" => "sha1crypt",
        _ => return None,
    };

    Some(Scheme::Crypt { name })
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
    fn decode_hex_roundtrips_and_is_case_insensitive() {
        let bytes = decode_hex("5D41402ABC4b2a76b9719d911017c592").unwrap();
        assert_eq!(to_hex(&bytes), "5d41402abc4b2a76b9719d911017c592");
    }

    #[test]
    fn decode_hex_rejects_malformed_input() {
        assert!(decode_hex("abc").is_none());
        assert!(decode_hex("zz").is_none());
    }

    #[test]
    fn test_hasher_functionality() {
        let md5_hasher = get_hasher("md5").unwrap();
        let sha1_hasher = get_hasher("sha1").unwrap();
        let sha256_hasher = get_hasher("sha256").unwrap();

        // Test actual hashing functionality
        let input = b"hello";

        assert_eq!(
            md5_hasher.hash(input).to_hex(),
            "5d41402abc4b2a76b9719d911017c592"
        );
        assert_eq!(
            sha1_hasher.hash(input).to_hex(),
            "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"
        );
        assert_eq!(
            sha256_hasher.hash(input).to_hex(),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn detect_scheme_treats_hex_as_raw() {
        assert_eq!(
            detect_scheme("5d41402abc4b2a76b9719d911017c592"),
            Some(Scheme::Raw)
        );
    }

    #[test]
    fn detect_scheme_maps_known_crypt_ids() {
        assert_eq!(
            detect_scheme("$1$p8Y6$MCdRLBzuGlfOs9S.hXOp0"),
            Some(Scheme::Crypt { name: "md5crypt" })
        );
        assert_eq!(
            detect_scheme("$6$salt$digest"),
            Some(Scheme::Crypt { name: "sha512crypt" })
        );
        assert_eq!(
            detect_scheme("$2b$12$abcdefghijklmnopqrstuv"),
            Some(Scheme::Crypt { name: "bcrypt" })
        );
    }

    #[test]
    fn detect_scheme_rejects_unknown_crypt_id() {
        assert_eq!(detect_scheme("$9$salt$digest"), None);
        assert_eq!(detect_scheme("$"), None);
    }
}
