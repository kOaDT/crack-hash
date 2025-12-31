use std::path::PathBuf;

use colored::*;

use crate::batch::BatchResult;

pub struct Display;

impl Display {
    pub fn print_banner() {
        println!("{}", "🔓 Crack Hash v1.1.0 🔓".bright_yellow().bold());
        println!("{}", "═════════════════════════".bright_yellow());
        println!();
    }

    pub fn print_start_info(algorithm: &str, target_hash: &str) {
        println!("{}", "STARTING HASH CRACKING...".bright_white().bold());
        println!();
        
        println!("{}", format!("Algorithm: {}", algorithm.bright_white().bold()));
        println!("{}", format!("Target: {}", target_hash.bright_white().bold()));
        println!();
    }

    pub fn print_success(plaintext: &str, attempts: u64, elapsed: std::time::Duration) {
        println!();
        println!("{}", "PLAINTEXT FOUND!".bright_green().bold().on_black());

        let plaintext_len = plaintext.len() + 15;
        let eq_count = (plaintext_len + 10) / 2;
        
        println!("{}", "=".repeat(eq_count).bright_white());
        println!("{}", format!("PLAINTEXT: {}    ", plaintext.bright_yellow().bold()));
        println!("{}", "=".repeat(eq_count).bright_white());
        println!();
        
        println!("{}", "--------------------------------".bright_white().bold());
        println!("{}", format!("Attempts: {}", attempts.to_string().bright_white().bold()));
        println!("{}", format!("Time: {:.2?}", elapsed).bright_white());
        println!("{}", format!("Rate: {:.0} h/s", Self::calculate_rate(attempts, elapsed)).bright_white());
        println!("{}", "--------------------------------".bright_white().bold());
    }

    pub fn print_failure(attempts: u64, elapsed: std::time::Duration) {
        println!();
        
        println!("{}", "❌ PLAINTEXT NOT FOUND".bright_red().bold());
        println!();
                
        println!("{}", format!("Total attempts: {}", attempts.to_string().bright_white().bold()));
        println!("{}", format!("Time elapsed: {:.2?}", elapsed).bright_white());
        println!("{}", format!("Hash rate: {:.0} h/s", Self::calculate_rate(attempts, elapsed)).bright_white());
        
        println!();
        println!("{}", "💡 Try a different wordlist or check if the hash is correct.".bright_yellow());
    }

    fn calculate_rate(attempts: u64, elapsed: std::time::Duration) -> f64 {
        let secs = elapsed.as_secs_f64();
        if secs > 0.0 {
            attempts as f64 / secs
        } else {
            attempts as f64
        }
    }

    pub fn print_error(message: &str) {
        println!();
        println!("{}", "ERROR".bright_red().bold());
        println!("{}", format!("{}", message).bright_red());
        println!();
    }

    pub fn print_batch_start_optimized(mode: &str, total: usize, unique: usize, skipped: usize, algorithm: &str) {
        println!(
            "{}",
            format!("BATCH MODE ({})", mode).bright_white().bold()
        );
        println!(
            "{}",
            format!("Algorithm: {}", algorithm.to_uppercase().bright_cyan().bold())
        );
        if skipped > 0 {
            println!(
                "{}",
                format!(
                    "Hashes: {} total, {} valid unique, {} skipped (invalid format)",
                    total.to_string().bright_white().bold(),
                    unique.to_string().bright_green().bold(),
                    skipped.to_string().bright_yellow()
                )
            );
        } else {
            println!(
                "{}",
                format!(
                    "Hashes: {} total, {} unique",
                    total.to_string().bright_white().bold(),
                    unique.to_string().bright_green().bold()
                )
            );
        }
        println!();
    }

    pub fn print_batch_match(hash: &str, plaintext: &str) {
        println!(
            "{}",
            format!("✓ {} → {}", hash.to_string(), plaintext.bright_green().bold())
        );
    }

    pub fn print_batch_complete_optimized(result: &BatchResult, output_path: &PathBuf) {
        println!();
        println!("{}", "BATCH PROCESSING COMPLETE".bright_green().bold());
        println!("{}", "═════════════════════════════════".bright_green());
        println!();

        println!(
            "{}",
            format!("Total hashes: {}", result.total)
                .bright_white()
                .bold()
        );
        println!(
            "{}",
            format!("Unique hashes: {}", result.unique)
                .bright_white()
        );
        if result.skipped > 0 {
            println!(
                "{}",
                format!("Skipped (invalid): {}", result.skipped)
                    .bright_yellow()
            );
        }
        println!(
            "{}",
            format!("Cracked: {}", result.cracked).bright_green().bold()
        );
        println!(
            "{}",
            format!("Not found: {}", result.failed).bright_red().bold()
        );
        println!(
            "{}",
            format!(
                "Success rate: {:.1}%",
                (result.cracked as f64 / result.total as f64) * 100.0
            )
            .bright_white()
        );
        println!(
            "{}",
            format!("Time elapsed: {:.2?}", result.elapsed).bright_white()
        );
        println!();
        println!(
            "{}",
            format!("Results saved to: {}", output_path.display()).bright_cyan()
        );
    }
} 