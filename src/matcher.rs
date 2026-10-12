use crate::cracker::HashCracker;
use crate::hash::{decode_hex, detect_scheme, get_hasher, Scheme};
use crate::{CrackError, Hasher};

/// Tests a candidate password against a single prepared target.
pub trait Matcher: Send + Sync {
    fn matches(&self, candidate: &[u8]) -> bool;
}

/// Unsalted hex digest: hash the candidate once and compare bytes.
struct RawMatcher {
    hasher: Box<dyn Hasher>,
    digest: Vec<u8>,
}

impl Matcher for RawMatcher {
    fn matches(&self, candidate: &[u8]) -> bool {
        self.hasher.hash(candidate).as_bytes() == self.digest.as_slice()
    }
}

/// Modular Crypt Format: pwhash re-derives the salt and iteration count from the
/// stored string and recognizes the scheme itself, so we hand it the raw hash.
struct CryptMatcher {
    mcf: String,
}

impl Matcher for CryptMatcher {
    fn matches(&self, candidate: &[u8]) -> bool {
        pwhash::unix::verify(candidate, self.mcf.as_str())
    }
}

fn is_crypt_b64(b: u8) -> bool {
    matches!(b, b'.' | b'/' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z')
}

/// Reject a crypt hash whose structure cannot be valid, so cracking fails in
/// microseconds instead of scanning a whole wordlist against an impossible
/// target. Checksum lengths are exact for the `$id$...$checksum` schemes; bcrypt
/// is a single fixed-width 60-char string; other grammars are only alphabet- and
/// presence-checked.
fn validate_crypt(target: &str, scheme: &str) -> Result<(), CrackError> {
    let malformed = |reason: String| CrackError::MalformedCryptHash {
        scheme: scheme.to_string(),
        reason,
    };

    if scheme == "bcrypt" {
        if target.len() != 60 {
            return Err(malformed(format!(
                "expected 60 characters, got {}",
                target.len()
            )));
        }
        return Ok(());
    }

    let fields: Vec<&str> = target.split('$').collect();
    let checksum = fields.last().copied().unwrap_or("");

    if fields.len() < 4 || fields.iter().skip(2).any(|f| f.is_empty()) {
        return Err(malformed("missing salt or checksum".to_string()));
    }

    let expected_len = match scheme {
        "md5crypt" => Some(22),
        "sha256crypt" => Some(43),
        "sha512crypt" => Some(86),
        _ => None,
    };

    if let Some(n) = expected_len {
        if checksum.len() != n {
            return Err(malformed(format!(
                "expected {}-character checksum, got {}",
                n,
                checksum.len()
            )));
        }
    }

    if !checksum.bytes().all(is_crypt_b64) {
        return Err(malformed(
            "checksum contains non-base64 characters".to_string(),
        ));
    }

    Ok(())
}

/// Build the matcher for `target` plus a human-readable scheme label for display.
/// `algo` is required for a raw hex digest and ignored for crypt hashes, whose
/// scheme is read from the `$id$` prefix.
pub fn build_matcher(
    target: &str,
    algo: Option<&str>,
) -> Result<(Box<dyn Matcher>, String), CrackError> {
    match detect_scheme(target) {
        Some(Scheme::Raw) => {
            let algo = algo.ok_or(CrackError::AlgorithmRequiredForRawHash)?;
            HashCracker::validate_hash_format(algo, target)?;
            let hasher = get_hasher(algo)
                .ok_or_else(|| CrackError::UnsupportedAlgorithm(algo.to_string()))?;
            let digest = decode_hex(target).ok_or(CrackError::InvalidHashCharacters)?;
            let label = hasher.name().to_string();
            Ok((Box::new(RawMatcher { hasher, digest }), label))
        }
        Some(Scheme::Crypt { name }) => {
            validate_crypt(target, name)?;
            Ok((
                Box::new(CryptMatcher {
                    mcf: target.to_string(),
                }),
                name.to_string(),
            ))
        }
        None => Err(CrackError::UnknownCryptScheme(
            target.split('$').nth(1).unwrap_or("").to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Hashes below are produced by `openssl passwd` for the password "hunter2",
    // so a match proves this crate cracks a real OS-generated hash.
    const MD5CRYPT_HUNTER2: &str = "$1$abcd$3XZbaqgzuThI2m9mmOxyV1";
    const SHA256CRYPT_HUNTER2: &str = "$5$abcd$U7Q4TqFwz7GS8FYxREjkgcX5lLiJnOxD9ftlB6qX695";
    const SHA512CRYPT_HUNTER2: &str =
        "$6$abcd$Qn1Z9Z5NNxb3xow2Atb3oXUiEVBwVmV4aBStoqZvjxTwRkBHzNb45iNwNBn.BoutZuVhuRGjmmU.dHMTEtO990";

    #[test]
    fn raw_matcher_compares_digest() {
        let (matcher, label) =
            build_matcher("5d41402abc4b2a76b9719d911017c592", Some("md5")).unwrap();
        assert_eq!(label, "MD5");
        assert!(matcher.matches(b"hello"));
        assert!(!matcher.matches(b"world"));
    }

    #[test]
    fn raw_hash_without_algo_is_rejected() {
        let err = build_matcher("5d41402abc4b2a76b9719d911017c592", None).err();
        assert!(matches!(err, Some(CrackError::AlgorithmRequiredForRawHash)));
    }

    #[test]
    fn crypt_matcher_cracks_md5crypt_without_algo() {
        let (matcher, label) = build_matcher(MD5CRYPT_HUNTER2, None).unwrap();
        assert_eq!(label, "md5crypt");
        assert!(matcher.matches(b"hunter2"));
        assert!(!matcher.matches(b"hunter3"));
    }

    #[test]
    fn crypt_matcher_cracks_sha512crypt() {
        let (matcher, label) = build_matcher(SHA512CRYPT_HUNTER2, None).unwrap();
        assert_eq!(label, "sha512crypt");
        assert!(matcher.matches(b"hunter2"));
        assert!(!matcher.matches(b"wrong"));
    }

    #[test]
    fn valid_sha256crypt_is_accepted_and_cracks() {
        let (matcher, label) = build_matcher(SHA256CRYPT_HUNTER2, None).unwrap();
        assert_eq!(label, "sha256crypt");
        assert!(matcher.matches(b"hunter2"));
    }

    // A one-character-short md5crypt checksum (21 instead of 22): the exact bug
    // that let a truncated Cisco type-5 hash scan a whole wordlist for nothing.
    #[test]
    fn malformed_md5crypt_checksum_is_rejected() {
        let err = build_matcher("$1$p8Y6$MCdRLBzuGlfOs9S.hXOp0", None).err();
        assert!(matches!(
            err,
            Some(CrackError::MalformedCryptHash { scheme, .. }) if scheme == "md5crypt"
        ));
    }

    #[test]
    fn crypt_hash_with_empty_salt_is_rejected() {
        let err = build_matcher("$6$$checksum", None).err();
        assert!(matches!(err, Some(CrackError::MalformedCryptHash { .. })));
    }

    #[test]
    fn bcrypt_wrong_length_is_rejected() {
        let err = build_matcher("$2b$12$tooshort", None).err();
        assert!(matches!(
            err,
            Some(CrackError::MalformedCryptHash { scheme, .. }) if scheme == "bcrypt"
        ));
    }

    #[test]
    fn unknown_crypt_scheme_is_rejected() {
        let err = build_matcher("$9$abcd$whatever", None).err();
        assert!(matches!(err, Some(CrackError::UnknownCryptScheme(id)) if id == "9"));
    }
}
