use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hash-cracker")]
#[command(about = "A hash cracking tool that supports multiple algorithms")]
#[command(version = "0.1.0")]
struct Cli {
    #[arg(short, long)]
    algo: String,

    #[arg(short = 'H', long)]
    hash: String,

    #[arg(short, long)]
    wordlist: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    println!("Hash Cracker v0.1.0");
    println!("==================");
    println!("Algorithm: {}", cli.algo);
    println!("Target hash: {}", cli.hash);
    println!("Wordlist: {}", cli.wordlist.display());

    // TODO: Implement hash cracking logic
    println!("\nHash cracking functionality will be implemented in the next phase.");
} 