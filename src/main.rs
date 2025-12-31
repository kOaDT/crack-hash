use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod batch;
mod cracker;
mod display;
mod hash;

use batch::{CsvBatchProcessor, TxtBatchProcessor};
use cracker::HashCracker;
use display::Display;
use hash::get_hasher;

pub trait Hasher: Send + Sync {
    fn name(&self) -> &'static str;

    fn hash(&self, input: &str) -> String;
}

#[derive(Debug)]
pub enum CrackError {
    UnsupportedAlgorithm(String),
    InvalidHashLength { expected_len: usize, actual_len: usize },
    InvalidHashCharacters,
    FileNotFound(String),
    IoError(std::io::Error),
    EmptyWordlist,
    EmptyInputFile,
    CsvError(String),
    InvalidColumnIndex { index: usize, max: usize },
}

impl std::fmt::Display for CrackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrackError::UnsupportedAlgorithm(algo) => {
                write!(
                    f,
                    "Unsupported algorithm: '{}'. Supported algorithms: md5, sha1, sha256",
                    algo
                )
            }
            CrackError::InvalidHashLength {
                expected_len,
                actual_len,
            } => {
                write!(
                    f,
                    "Invalid hash length. Expected {} characters, got {}",
                    expected_len, actual_len
                )
            }
            CrackError::InvalidHashCharacters => {
                write!(
                    f,
                    "Invalid hash format. Hash must contain only hexadecimal characters (0-9, a-f)"
                )
            }
            CrackError::FileNotFound(path) => {
                write!(f, "File not found: {}", path)
            }
            CrackError::IoError(err) => {
                write!(f, "I/O error: {}", err)
            }
            CrackError::EmptyWordlist => {
                write!(f, "Wordlist is empty")
            }
            CrackError::EmptyInputFile => {
                write!(f, "Input file is empty or contains no valid entries")
            }
            CrackError::CsvError(msg) => {
                write!(f, "CSV error: {}", msg)
            }
            CrackError::InvalidColumnIndex { index, max } => {
                write!(
                    f,
                    "Invalid column index: {}. Maximum valid index is {}",
                    index, max
                )
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
#[command(version = "1.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Crack a single hash")]
    Single {
        #[arg(short, long, help = "Hash algorithm (md5, sha1, sha256)")]
        algo: String,

        #[arg(short = 'H', long, help = "Target hash to crack")]
        hash: String,

        #[arg(short, long, help = "Path to wordlist file")]
        wordlist: PathBuf,
    },

    #[command(about = "Crack multiple hashes from a TXT file (one hash per line)")]
    BatchTxt {
        #[arg(short, long, help = "Hash algorithm (md5, sha1, sha256)")]
        algo: String,

        #[arg(short, long, help = "Input file containing hashes (one per line)")]
        input: PathBuf,

        #[arg(short, long, help = "Output file for results")]
        output: PathBuf,

        #[arg(short, long, help = "Path to wordlist file")]
        wordlist: PathBuf,
    },

    #[command(about = "Crack multiple hashes from a CSV file")]
    BatchCsv {
        #[arg(short, long, help = "Hash algorithm (md5, sha1, sha256)")]
        algo: String,

        #[arg(short, long, help = "Input CSV file")]
        input: PathBuf,

        #[arg(short, long, help = "Output CSV file")]
        output: PathBuf,

        #[arg(short, long, help = "Path to wordlist file")]
        wordlist: PathBuf,

        #[arg(
            short = 'c',
            long,
            help = "Column index containing the hash (0-based)",
            default_value = "0"
        )]
        hash_column: usize,

        #[arg(
            short = 'd',
            long,
            help = "CSV delimiter character",
            default_value = ","
        )]
        delimiter: char,

        #[arg(long, help = "CSV file has no header row (headers enabled by default)")]
        no_header: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    Display::print_banner();

    let exit_code = match cli.command {
        Commands::Single {
            algo,
            hash,
            wordlist,
        } => run_single_mode(algo, hash, wordlist),

        Commands::BatchTxt {
            algo,
            input,
            output,
            wordlist,
        } => run_batch_txt_mode(algo, input, output, wordlist),

        Commands::BatchCsv {
            algo,
            input,
            output,
            wordlist,
            hash_column,
            delimiter,
            no_header,
        } => run_batch_csv_mode(algo, input, output, wordlist, hash_column, delimiter, !no_header),
    };

    std::process::exit(exit_code);
}

fn run_single_mode(algo: String, hash: String, wordlist: PathBuf) -> i32 {
    let hasher = match get_hasher(&algo) {
        Some(hasher) => hasher,
        None => {
            let error = CrackError::UnsupportedAlgorithm(algo);
            Display::print_error(&error.to_string());
            return 1;
        }
    };

    if let Err(e) = HashCracker::validate_hash_format(&algo, &hash) {
        Display::print_error(&e.to_string());
        return 1;
    }

    let cracker = HashCracker::new(hasher, hash, wordlist);

    match cracker.crack() {
        Ok(Some(_)) => 0,
        Ok(None) => 1,
        Err(e) => {
            Display::print_error(&e.to_string());
            1
        }
    }
}

fn run_batch_txt_mode(algo: String, input: PathBuf, output: PathBuf, wordlist: PathBuf) -> i32 {
    let processor = TxtBatchProcessor::new(algo, input, output, wordlist);

    match processor.process() {
        Ok(result) => {
            if result.cracked > 0 {
                0
            } else {
                1
            }
        }
        Err(e) => {
            Display::print_error(&e.to_string());
            1
        }
    }
}

fn run_batch_csv_mode(
    algo: String,
    input: PathBuf,
    output: PathBuf,
    wordlist: PathBuf,
    hash_column: usize,
    delimiter: char,
    header: bool,
) -> i32 {
    let processor =
        CsvBatchProcessor::new(algo, input, output, wordlist, hash_column, delimiter, header);

    match processor.process() {
        Ok(result) => {
            if result.cracked > 0 {
                0
            } else {
                1
            }
        }
        Err(e) => {
            Display::print_error(&e.to_string());
            1
        }
    }
}
