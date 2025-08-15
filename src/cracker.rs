use std::path::PathBuf;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;

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
        
        let words = self.load_wordlist()?;
        
        if words.is_empty() {
            return Err(CrackError::EmptyWordlist);
        }

        let stats = Arc::new(Mutex::new(CrackingStats::new()));
        let hasher = Arc::new(self.hasher.as_ref());
        let target_hash = Arc::new(self.target_hash.clone());

        let result = self.attempt_crack_parallel(&words, &hasher, &target_hash, &stats);

        let stats = stats.lock().unwrap();
        match result {
            Some(password) => {
                Display::print_success(&password, stats.attempts, stats.elapsed());
                Ok(Some(password))
            }
            None => {
                Display::print_failure(stats.attempts, stats.elapsed());
                Ok(None)
            }
        }
    }

    fn validate_wordlist(&self) -> Result<(), CrackError> {
        if !self.wordlist_path.exists() {
            return Err(CrackError::FileNotFound(self.wordlist_path.display().to_string()));
        }
        Ok(())
    }

    fn load_wordlist(&self) -> Result<Vec<String>, CrackError> {
        let file = File::open(&self.wordlist_path)?;
        let reader = BufReader::new(file);
        
        let mut words = Vec::new();
        for line_result in reader.lines() {
            let password = match line_result {
                Ok(line) => line.trim().to_string(),
                Err(_) => continue,
            };
            
            if !password.is_empty() {
                words.push(password);
            }
        }
        
        Ok(words)
    }

    fn attempt_crack_parallel(
        &self,
        words: &[String],
        hasher: &Arc<&dyn Hasher>,
        target_hash: &Arc<String>,
        stats: &Arc<Mutex<CrackingStats>>,
    ) -> Option<String> {
        let chunk_size = (words.len() / rayon::current_num_threads()).max(1);
        
        words
            .par_chunks(chunk_size)
            .find_map_any(|chunk| {
                for password in chunk {
                    {
                        let mut stats = stats.lock().unwrap();
                        stats.increment();
                        
                        if stats.should_show_progress() {
                            Display::print_progress(stats.attempts);
                        }
                    }

                    if self.check_password_match_parallel(password, hasher, target_hash) {
                        return Some(password.clone());
                    }
                }
                None
            })
    }

    fn check_password_match_parallel(
        &self,
        password: &str,
        hasher: &Arc<&dyn Hasher>,
        target_hash: &Arc<String>,
    ) -> bool {
        let computed_hash = hasher.hash(password);
        computed_hash.eq_ignore_ascii_case(target_hash)
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