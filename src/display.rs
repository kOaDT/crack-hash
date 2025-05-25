use colored::*;

pub struct Display;

impl Display {
    pub fn print_banner() {
        println!("{}", "🔓 Crack Hash v0.1.0 🔓".bright_yellow().bold());
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

    pub fn print_progress(attempts: u64) {
        let progress_msg = format!("🔍 Tried {} passwords...", attempts.to_string().bright_white().bold());
        println!("{}", progress_msg.bright_cyan());
    }

    pub fn print_success(password: &str, attempts: u64, elapsed: std::time::Duration) {
        println!();
        println!("{}", "PASSWORD FOUND!".bright_green().bold().on_black());

        let password_len = password.len() + 15;
        let eq_count = (password_len + 10) / 2;
        
        println!("{}", "=".repeat(eq_count).bright_white());
        println!("{}", format!("PASSWORD: {}    ", password.bright_yellow().bold()));
        println!("{}", "=".repeat(eq_count).bright_white());
        println!();
        
        println!("{}", "--------------------------------".bright_white().bold());
        println!("{}", format!("Attempts: {}", attempts.to_string().bright_white().bold()));
        println!("{}", format!("Time: {:.2?}", elapsed).bright_white());
        println!("{}", format!("Rate: {:.0} h/s", attempts as f64 / elapsed.as_secs_f64()).bright_white());
        println!("{}", "--------------------------------".bright_white().bold());
    }

    pub fn print_failure(attempts: u64, elapsed: std::time::Duration) {
        println!();
        
        println!("{}", "❌ PASSWORD NOT FOUND".bright_red().bold());
        println!();
                
        println!("{}", format!("Total attempts: {}", attempts.to_string().bright_white().bold()));
        println!("{}", format!("Time elapsed: {:.2?}", elapsed).bright_white());
        println!("{}", format!("Hash rate: {:.0} h/s", attempts as f64 / elapsed.as_secs_f64()).bright_white());
        
        println!();
        println!("{}", "💡 Try a different wordlist or check if the hash is correct.".bright_yellow());
    }

    pub fn print_error(message: &str) {
        println!();
        println!("{}", "ERROR".bright_red().bold());
        println!("{}", format!("{}", message).bright_red());
        println!();
    }
} 