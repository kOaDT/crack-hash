use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use csv::{ReaderBuilder, WriterBuilder};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use crate::cracker::HashCracker;
use crate::display::Display;
use crate::hash::get_hasher;
use crate::CrackError;

pub struct BatchResult {
    pub total: usize,
    pub unique: usize,
    pub skipped: usize,
    pub cracked: usize,
    pub failed: usize,
    pub elapsed: std::time::Duration,
}

pub struct TxtBatchProcessor {
    algorithm: String,
    input_path: PathBuf,
    output_path: PathBuf,
    wordlist_path: PathBuf,
}

impl TxtBatchProcessor {
    pub fn new(
        algorithm: String,
        input_path: PathBuf,
        output_path: PathBuf,
        wordlist_path: PathBuf,
    ) -> Self {
        Self {
            algorithm,
            input_path,
            output_path,
            wordlist_path,
        }
    }

    pub fn process(&self) -> Result<BatchResult, CrackError> {
        let hashes = self.load_hashes()?;
        let total = hashes.len();

        if total == 0 {
            return Err(CrackError::EmptyInputFile);
        }

        let hasher = get_hasher(&self.algorithm)
            .ok_or_else(|| CrackError::UnsupportedAlgorithm(self.algorithm.clone()))?;

        let mut target_map: HashMap<String, Vec<usize>> = HashMap::new();
        let mut invalid_indices: Vec<usize> = Vec::new();

        for (idx, hash) in hashes.iter().enumerate() {
            if HashCracker::validate_hash_format(&self.algorithm, hash).is_err() {
                invalid_indices.push(idx);
                continue;
            }
            let normalized = hash.to_lowercase();
            target_map.entry(normalized).or_default().push(idx);
        }

        let unique_count = target_map.len();
        let skipped_count = invalid_indices.len();
        Display::print_batch_start_optimized("TXT", total, unique_count, skipped_count, &self.algorithm);

        let start_time = Instant::now();

        let words = Self::load_wordlist(&self.wordlist_path)?;
        let word_count = words.len() as u64;
        let progress_bar = Self::create_wordlist_progress_bar(word_count);
        let progress_bar = Arc::new(progress_bar);

        let found: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
        let remaining = Arc::new(Mutex::new(target_map.len()));

        let chunk_size = (words.len() / rayon::current_num_threads()).max(1000);

        words.par_chunks(chunk_size).for_each(|chunk| {
            let mut local_found: Vec<(String, String)> = Vec::new();

            for word in chunk {
                if *remaining.lock().unwrap() == 0 {
                    break;
                }

                let computed = hasher.hash(word);
                let computed_lower = computed.to_lowercase();

                if target_map.contains_key(&computed_lower) {
                    local_found.push((computed_lower, word.clone()));
                }
            }

            if !local_found.is_empty() {
                let mut found_guard = found.lock().unwrap();
                let mut remaining_guard = remaining.lock().unwrap();

                for (hash, plaintext) in local_found {
                    if !found_guard.contains_key(&hash) {
                        found_guard.insert(hash.clone(), plaintext.clone());
                        *remaining_guard = remaining_guard.saturating_sub(1);
                        Display::print_batch_match(&hash, &plaintext);
                    }
                }
            }

            progress_bar.inc(chunk.len() as u64);
        });

        progress_bar.finish_with_message("Wordlist processed");

        let found_map = Arc::try_unwrap(found).unwrap().into_inner().unwrap();

        let mut results: Vec<(String, Option<String>)> = vec![(String::new(), None); total];

        for (hash_lower, indices) in target_map.iter() {
            let plaintext = found_map.get(hash_lower).cloned();
            for &idx in indices {
                results[idx] = (hashes[idx].clone(), plaintext.clone());
            }
        }

        for idx in invalid_indices {
            results[idx] = (hashes[idx].clone(), None);
        }

        let cracked_total = results.iter().filter(|(_, pt)| pt.is_some()).count();

        self.write_results(&results)?;

        let elapsed = start_time.elapsed();
        let result = BatchResult {
            total,
            unique: unique_count,
            skipped: skipped_count,
            cracked: cracked_total,
            failed: total - cracked_total - skipped_count,
            elapsed,
        };

        Display::print_batch_complete_optimized(&result, &self.output_path);

        Ok(result)
    }

    fn load_hashes(&self) -> Result<Vec<String>, CrackError> {
        if !self.input_path.exists() {
            return Err(CrackError::FileNotFound(
                self.input_path.display().to_string(),
            ));
        }

        let file = File::open(&self.input_path)?;
        let reader = BufReader::new(file);

        let hashes: Vec<String> = reader
            .lines()
            .filter_map(|line| line.ok())
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        Ok(hashes)
    }

    pub fn load_wordlist(path: &PathBuf) -> Result<Vec<String>, CrackError> {
        if !path.exists() {
            return Err(CrackError::FileNotFound(path.display().to_string()));
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let words: Vec<String> = reader
            .lines()
            .filter_map(|line| line.ok())
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        if words.is_empty() {
            return Err(CrackError::EmptyWordlist);
        }

        Ok(words)
    }

    fn write_results(&self, results: &[(String, Option<String>)]) -> Result<(), CrackError> {
        let file = File::create(&self.output_path)?;
        let mut writer = BufWriter::new(file);

        for (hash, plaintext) in results {
            let line = match plaintext {
                Some(pt) => format!("{}:{}\n", hash, pt),
                None => format!("{}:NOT_FOUND\n", hash),
            };
            writer.write_all(line.as_bytes())?;
        }

        writer.flush()?;
        Ok(())
    }

    fn create_wordlist_progress_bar(total: u64) -> ProgressBar {
        let pb = ProgressBar::new(total);
        let style = ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} words ({per_sec})")
            .unwrap()
            .progress_chars("#>-");
        pb.set_style(style);
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        pb
    }
}

pub struct CsvBatchProcessor {
    algorithm: String,
    input_path: PathBuf,
    output_path: PathBuf,
    wordlist_path: PathBuf,
    hash_column: usize,
    delimiter: u8,
    has_header: bool,
}

impl CsvBatchProcessor {
    pub fn new(
        algorithm: String,
        input_path: PathBuf,
        output_path: PathBuf,
        wordlist_path: PathBuf,
        hash_column: usize,
        delimiter: char,
        has_header: bool,
    ) -> Self {
        Self {
            algorithm,
            input_path,
            output_path,
            wordlist_path,
            hash_column,
            delimiter: delimiter as u8,
            has_header,
        }
    }

    pub fn process(&self) -> Result<BatchResult, CrackError> {
        let (header, records) = self.load_csv()?;
        let total = records.len();

        if total == 0 {
            return Err(CrackError::EmptyInputFile);
        }

        let hasher = get_hasher(&self.algorithm)
            .ok_or_else(|| CrackError::UnsupportedAlgorithm(self.algorithm.clone()))?;

        let mut target_map: HashMap<String, Vec<usize>> = HashMap::new();
        let mut invalid_indices: Vec<usize> = Vec::new();
        let mut empty_indices: Vec<usize> = Vec::new();

        for (idx, record) in records.iter().enumerate() {
            let hash = record
                .get(self.hash_column)
                .map(|s| s.trim())
                .unwrap_or("");

            if hash.is_empty() {
                empty_indices.push(idx);
                continue;
            }

            if HashCracker::validate_hash_format(&self.algorithm, hash).is_err() {
                invalid_indices.push(idx);
                continue;
            }

            let normalized = hash.to_lowercase();
            target_map.entry(normalized).or_default().push(idx);
        }

        let unique_count = target_map.len();
        let skipped_count = invalid_indices.len() + empty_indices.len();
        Display::print_batch_start_optimized("CSV", total, unique_count, skipped_count, &self.algorithm);

        let start_time = Instant::now();

        let words = TxtBatchProcessor::load_wordlist(&self.wordlist_path)?;
        let word_count = words.len() as u64;
        let progress_bar = TxtBatchProcessor::create_wordlist_progress_bar(word_count);
        let progress_bar = Arc::new(progress_bar);

        let found: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
        let remaining = Arc::new(Mutex::new(target_map.len()));

        let chunk_size = (words.len() / rayon::current_num_threads()).max(1000);

        words.par_chunks(chunk_size).for_each(|chunk| {
            let mut local_found: Vec<(String, String)> = Vec::new();

            for word in chunk {
                if *remaining.lock().unwrap() == 0 {
                    break;
                }

                let computed = hasher.hash(word);
                let computed_lower = computed.to_lowercase();

                if target_map.contains_key(&computed_lower) {
                    local_found.push((computed_lower, word.clone()));
                }
            }

            if !local_found.is_empty() {
                let mut found_guard = found.lock().unwrap();
                let mut remaining_guard = remaining.lock().unwrap();

                for (hash, plaintext) in local_found {
                    if !found_guard.contains_key(&hash) {
                        found_guard.insert(hash.clone(), plaintext.clone());
                        *remaining_guard = remaining_guard.saturating_sub(1);
                        Display::print_batch_match(&hash, &plaintext);
                    }
                }
            }

            progress_bar.inc(chunk.len() as u64);
        });

        progress_bar.finish_with_message("Wordlist processed");

        let found_map = Arc::try_unwrap(found).unwrap().into_inner().unwrap();

        let mut results: Vec<(Vec<String>, Option<String>)> =
            records.iter().map(|r| (r.clone(), None)).collect();

        for (hash_lower, indices) in target_map.iter() {
            let plaintext = found_map.get(hash_lower).cloned();
            for &idx in indices {
                results[idx].1 = plaintext.clone();
            }
        }

        let cracked_total = results.iter().filter(|(_, pt)| pt.is_some()).count();

        self.write_csv(&header, &results)?;

        let elapsed = start_time.elapsed();
        let result = BatchResult {
            total,
            unique: unique_count,
            skipped: skipped_count,
            cracked: cracked_total,
            failed: total - cracked_total - skipped_count,
            elapsed,
        };

        Display::print_batch_complete_optimized(&result, &self.output_path);

        Ok(result)
    }

    fn load_csv(&self) -> Result<(Option<Vec<String>>, Vec<Vec<String>>), CrackError> {
        if !self.input_path.exists() {
            return Err(CrackError::FileNotFound(
                self.input_path.display().to_string(),
            ));
        }

        let file = File::open(&self.input_path)?;
        let mut reader = ReaderBuilder::new()
            .delimiter(self.delimiter)
            .has_headers(self.has_header)
            .flexible(true)
            .from_reader(file);

        let header = if self.has_header {
            Some(
                reader
                    .headers()
                    .map_err(|e| CrackError::CsvError(e.to_string()))?
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            )
        } else {
            None
        };

        let mut records = Vec::new();
        for result in reader.records() {
            let record = result.map_err(|e| CrackError::CsvError(e.to_string()))?;

            if self.hash_column >= record.len() {
                return Err(CrackError::InvalidColumnIndex {
                    index: self.hash_column,
                    max: record.len().saturating_sub(1),
                });
            }

            records.push(record.iter().map(|s| s.trim().to_string()).collect());
        }

        Ok((header, records))
    }

    fn write_csv(
        &self,
        header: &Option<Vec<String>>,
        results: &[(Vec<String>, Option<String>)],
    ) -> Result<(), CrackError> {
        let file = File::create(&self.output_path)?;
        let mut writer = WriterBuilder::new()
            .delimiter(self.delimiter)
            .from_writer(file);

        if let Some(h) = header {
            let mut new_header = h.clone();
            new_header.push("plaintext".to_string());
            writer
                .write_record(&new_header)
                .map_err(|e| CrackError::CsvError(e.to_string()))?;
        }

        for (record, plaintext) in results {
            let mut new_record = record.clone();
            new_record.push(plaintext.clone().unwrap_or_else(|| "NOT_FOUND".to_string()));
            writer
                .write_record(&new_record)
                .map_err(|e| CrackError::CsvError(e.to_string()))?;
        }

        writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    fn create_test_wordlist(dir: &std::path::Path) -> PathBuf {
        let wordlist_path = dir.join("wordlist.txt");
        let mut file = File::create(&wordlist_path).unwrap();
        writeln!(file, "password").unwrap();
        writeln!(file, "123456").unwrap();
        writeln!(file, "hello").unwrap();
        writeln!(file, "world").unwrap();
        wordlist_path
    }

    #[test]
    fn test_txt_batch_processor() {
        let dir = tempdir().unwrap();
        let wordlist_path = create_test_wordlist(dir.path());

        let input_path = dir.path().join("hashes.txt");
        let output_path = dir.path().join("results.txt");

        let mut input_file = File::create(&input_path).unwrap();
        writeln!(input_file, "5d41402abc4b2a76b9719d911017c592").unwrap(); // hello
        writeln!(input_file, "e10adc3949ba59abbe56e057f20f883e").unwrap(); // 123456

        let processor = TxtBatchProcessor::new(
            "md5".to_string(),
            input_path,
            output_path.clone(),
            wordlist_path,
        );

        let result = processor.process().unwrap();
        assert_eq!(result.total, 2);
        assert_eq!(result.cracked, 2);

        let output_content = std::fs::read_to_string(&output_path).unwrap();
        assert!(output_content.contains("hello"));
        assert!(output_content.contains("123456"));
    }

    #[test]
    fn test_txt_batch_with_duplicates() {
        let dir = tempdir().unwrap();
        let wordlist_path = create_test_wordlist(dir.path());

        let input_path = dir.path().join("hashes.txt");
        let output_path = dir.path().join("results.txt");

        let mut input_file = File::create(&input_path).unwrap();
        writeln!(input_file, "5d41402abc4b2a76b9719d911017c592").unwrap(); // hello
        writeln!(input_file, "5d41402abc4b2a76b9719d911017c592").unwrap(); // hello (duplicate)
        writeln!(input_file, "e10adc3949ba59abbe56e057f20f883e").unwrap(); // 123456

        let processor = TxtBatchProcessor::new(
            "md5".to_string(),
            input_path,
            output_path.clone(),
            wordlist_path,
        );

        let result = processor.process().unwrap();
        assert_eq!(result.total, 3);
        assert_eq!(result.unique, 2);
        assert_eq!(result.cracked, 3);

        let output_content = std::fs::read_to_string(&output_path).unwrap();
        let hello_count = output_content.matches("hello").count();
        assert_eq!(hello_count, 2);
    }

    #[test]
    fn test_csv_batch_processor() {
        let dir = tempdir().unwrap();
        let wordlist_path = create_test_wordlist(dir.path());

        let input_path = dir.path().join("hashes.csv");
        let output_path = dir.path().join("results.csv");

        let mut input_file = File::create(&input_path).unwrap();
        writeln!(input_file, "id,hash,email").unwrap();
        writeln!(
            input_file,
            "1,5d41402abc4b2a76b9719d911017c592,user1@test.com"
        )
        .unwrap(); // hello
        writeln!(
            input_file,
            "2,e10adc3949ba59abbe56e057f20f883e,user2@test.com"
        )
        .unwrap(); // 123456

        let processor = CsvBatchProcessor::new(
            "md5".to_string(),
            input_path,
            output_path.clone(),
            wordlist_path,
            1,
            ',',
            true,
        );

        let result = processor.process().unwrap();
        assert_eq!(result.total, 2);
        assert_eq!(result.cracked, 2);

        let output_content = std::fs::read_to_string(&output_path).unwrap();
        assert!(output_content.contains("plaintext"));
        assert!(output_content.contains("hello"));
        assert!(output_content.contains("123456"));
    }

    #[test]
    fn test_txt_batch_with_invalid_hash() {
        let dir = tempdir().unwrap();
        let wordlist_path = create_test_wordlist(dir.path());

        let input_path = dir.path().join("hashes.txt");
        let output_path = dir.path().join("results.txt");

        let mut input_file = File::create(&input_path).unwrap();
        writeln!(input_file, "5d41402abc4b2a76b9719d911017c592").unwrap(); // hello
        writeln!(input_file, "invalid_hash").unwrap();

        let processor = TxtBatchProcessor::new(
            "md5".to_string(),
            input_path,
            output_path.clone(),
            wordlist_path,
        );

        let result = processor.process().unwrap();
        assert_eq!(result.total, 2);
        assert_eq!(result.cracked, 1);
        assert_eq!(result.skipped, 1);
        assert_eq!(result.failed, 0);
    }

    #[test]
    fn test_csv_batch_no_header() {
        let dir = tempdir().unwrap();
        let wordlist_path = create_test_wordlist(dir.path());

        let input_path = dir.path().join("hashes.csv");
        let output_path = dir.path().join("results.csv");

        let mut input_file = File::create(&input_path).unwrap();
        writeln!(input_file, "5d41402abc4b2a76b9719d911017c592,extra_data").unwrap();

        let processor = CsvBatchProcessor::new(
            "md5".to_string(),
            input_path,
            output_path.clone(),
            wordlist_path,
            0,
            ',',
            false,
        );

        let result = processor.process().unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.cracked, 1);
    }
}
