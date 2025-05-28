use clap::Parser;
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

mod hash;
mod display;

use hash::md5::Md5Hasher;
use hash::sha1::Sha1Hasher;
use hash::sha256::Sha256Hasher;
use display::Display;

pub trait Hasher {
    fn name(&self) -> &'static str;
    
    fn hash(&self, input: &str) -> String;
}

#[derive(Debug)]
pub enum CrackError {
    UnsupportedAlgorithm(String),
    InvalidHashFormat { expected_len: usize, actual_len: usize },
    FileNotFound(String),
    IoError(std::io::Error),
    EmptyWordlist,
}

impl std::fmt::Display for CrackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrackError::UnsupportedAlgorithm(algo) => {
                write!(f, "Unsupported algorithm: '{}'. Supported algorithms: md5, sha1, sha256", algo)
            }
            CrackError::InvalidHashFormat { expected_len, actual_len } => {
                write!(f, "Invalid hash format. Expected {} characters, got {}", expected_len, actual_len)
            }
            CrackError::FileNotFound(path) => {
                write!(f, "Wordlist file not found: {}", path)
            }
            CrackError::IoError(err) => {
                write!(f, "I/O error: {}", err)
            }
            CrackError::EmptyWordlist => {
                write!(f, "Wordlist is empty")
            }
        }
    }
}

impl std::error::Error for CrackError {}

impl From<std::io::Error> for CrackError {
    fn from(err: std::io::Error) -> Self {
        CrackError::IoError(err)
    }
}

pub struct HashCracker;

impl HashCracker {
    pub fn create_hasher(algorithm: &str) -> Result<Box<dyn Hasher>, CrackError> {
        match algorithm.to_lowercase().as_str() {
            "md5" => Ok(Box::new(Md5Hasher::new())),
            "sha1" => Ok(Box::new(Sha1Hasher::new())),
            "sha256" => Ok(Box::new(Sha256Hasher::new())),
            _ => Err(CrackError::UnsupportedAlgorithm(algorithm.to_string())),
        }
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

    pub fn crack_hash(
        hasher: &dyn Hasher,
        target_hash: &str,
        wordlist_path: &PathBuf,
    ) -> Result<Option<String>, CrackError> {
        if !wordlist_path.exists() {
            return Err(CrackError::FileNotFound(wordlist_path.display().to_string()));
        }

        let file = File::open(wordlist_path)?;
        let reader = BufReader::new(file);
        
        let mut attempts = 0;
        let start_time = Instant::now();

        Display::print_start_info(hasher.name(), target_hash);

        for (_line_num, line_result) in reader.lines().enumerate() {
            let password = match line_result {
                Ok(line) => line,
                Err(_) => {
                    continue;
                }
            };
            
            attempts += 1;

            let computed_hash = hasher.hash(&password);
            
            if attempts % 10000 == 0 {
                Display::print_progress(attempts);
            }

            if computed_hash.eq_ignore_ascii_case(target_hash) {
                let elapsed = start_time.elapsed();
                Display::print_success(&password, attempts, elapsed);
                
                return Ok(Some(password));
            }
        }

        if attempts == 0 {
            return Err(CrackError::EmptyWordlist);
        }

        let elapsed = start_time.elapsed();
        Display::print_failure(attempts, elapsed);
        
        Ok(None)
    }
}

#[derive(Parser)]
#[command(name = "crack-hash")]
#[command(about = "A hash cracking tool that supports multiple algorithms")]
#[command(version = "0.1.0")]
struct Cli {
    #[arg(short, long, help = "Hash algorithm (supported: md5, sha1, sha256)")]
    algo: String,

    #[arg(short = 'H', long, help = "Target hash to crack")]
    hash: String,

    #[arg(short, long, help = "Path to wordlist file")]
    wordlist: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    Display::print_banner();
    
    // Validate and create hasher
    let hasher = match HashCracker::create_hasher(&cli.algo) {
        Ok(hasher) => hasher,
        Err(e) => {
            Display::print_error(&e.to_string());
            std::process::exit(1);
        }
    };

    // Validate hash format
    if let Err(e) = HashCracker::validate_hash_format(&cli.algo, &cli.hash) {
        Display::print_error(&e.to_string());
        std::process::exit(1);
    }

    // Attempt to crack the hash
    match HashCracker::crack_hash(hasher.as_ref(), &cli.hash, &cli.wordlist) {
        Ok(Some(_password)) => {
            std::process::exit(0);
        }
        Ok(None) => {
            std::process::exit(1);
        }
        Err(e) => {
            Display::print_error(&e.to_string());
            std::process::exit(1);
        }
    }
} 