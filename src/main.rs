use clap::Parser;
use std::path::PathBuf;

mod hash;
mod display;
mod cracker;

use hash::get_hasher;
use display::Display;
use cracker::HashCracker;

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
    
    let hasher = match get_hasher(&cli.algo) {
        Some(hasher) => hasher,
        None => {
            let error = CrackError::UnsupportedAlgorithm(cli.algo.clone());
            Display::print_error(&error.to_string());
            std::process::exit(1);
        }
    };

    if let Err(e) = HashCracker::validate_hash_format(&cli.algo, &cli.hash) {
        Display::print_error(&e.to_string());
        std::process::exit(1);
    }

    let cracker = HashCracker::new(hasher, cli.hash, cli.wordlist);
    
    match cracker.crack() {
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