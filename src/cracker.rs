use std::path::PathBuf;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

use crate::{Hasher, CrackError};
use crate::display::Display;

pub struct HashCracker {
    hasher: Box<dyn Hasher>,
    target_hash: String,
    wordlist_path: PathBuf,
}

pub struct CrackingStats {
    pub attempts: u64,
    pub start_time: Instant,
}

impl CrackingStats {
    fn new() -> Self {
        Self {
            attempts: 0,
            start_time: Instant::now(),
        }
    }

    fn increment(&mut self) {
        self.attempts += 1;
    }

    fn should_show_progress(&self) -> bool {
        self.attempts % 10000 == 0
    }

    fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

impl HashCracker {
    pub fn new(hasher: Box<dyn Hasher>, target_hash: String, wordlist_path: PathBuf) -> Self {
        Self {
            hasher,
            target_hash,
            wordlist_path,
        }
    }

    pub fn crack(&self) -> Result<Option<String>, CrackError> {
        self.validate_wordlist()?;
        
        Display::print_start_info(self.hasher.name(), &self.target_hash);
        
        let mut stats = CrackingStats::new();
        let reader = self.create_wordlist_reader()?;

        match self.attempt_crack(reader, &mut stats) {
            Some(password) => {
                Display::print_success(&password, stats.attempts, stats.elapsed());
                Ok(Some(password))
            }
            None => {
                if stats.attempts == 0 {
                    Err(CrackError::EmptyWordlist)
                } else {
                    Display::print_failure(stats.attempts, stats.elapsed());
                    Ok(None)
                }
            }
        }
    }

    fn validate_wordlist(&self) -> Result<(), CrackError> {
        if !self.wordlist_path.exists() {
            return Err(CrackError::FileNotFound(self.wordlist_path.display().to_string()));
        }
        Ok(())
    }

    fn create_wordlist_reader(&self) -> Result<BufReader<File>, CrackError> {
        let file = File::open(&self.wordlist_path)?;
        Ok(BufReader::new(file))
    }

    fn attempt_crack(&self, reader: BufReader<File>, stats: &mut CrackingStats) -> Option<String> {
        for line_result in reader.lines() {
            let password = match line_result {
                Ok(line) => line.trim().to_string(),
                Err(_) => continue,
            };

            stats.increment();

            if stats.should_show_progress() {
                Display::print_progress(stats.attempts);
            }

            if self.check_password_match(&password) {
                return Some(password);
            }
        }
        None
    }

    fn check_password_match(&self, password: &str) -> bool {
        let computed_hash = self.hasher.hash(password);
        computed_hash.eq_ignore_ascii_case(&self.target_hash)
    }

    pub fn validate_hash_format(algorithm: &str, hash: &str) -> Result<(), CrackError> {
        let expected_len = match algorithm.to_lowercase().as_str() {
            "md5" => 32,
            "sha1" => 40,
            "sha256" => 64,
            _ => return Err(CrackError::UnsupportedAlgorithm(algorithm.to_string())),
        };

        if hash.len() != expected_len {
            return Err(CrackError::InvalidHashFormat {
                expected_len,
                actual_len: hash.len(),
            });
        }

        if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(CrackError::InvalidHashFormat {
                expected_len,
                actual_len: hash.len(),
            });
        }

        Ok(())
    }
} 