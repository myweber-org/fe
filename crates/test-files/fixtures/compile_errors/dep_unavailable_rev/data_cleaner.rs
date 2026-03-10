use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(Path::new(output_path))?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in rdr.deserialize() {
        let record: Record = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Skipping invalid record: {}", e);
                continue;
            }
        };

        let cleaned_record = Record {
            id: record.id,
            name: record.name.trim().to_string(),
            value: if record.value.is_nan() || record.value.is_infinite() {
                0.0
            } else {
                record.value
            },
            category: if record.category.is_empty() {
                "uncategorized".to_string()
            } else {
                record.category
            },
        };

        wtr.serialize(&cleaned_record)?;
    }

    wtr.flush()?;
    println!("Data cleaning completed successfully");
    Ok(())
}

fn main() {
    let input_file = "raw_data.csv";
    let output_file = "cleaned_data.csv";

    if let Err(e) = clean_csv_data(input_file, output_file) {
        eprintln!("Error occurred during data cleaning: {}", e);
        std::process::exit(1);
    }
}
use regex::Regex;
use std::collections::HashSet;

pub fn clean_and_normalize(input: &str, stop_words: &HashSet<&str>) -> String {
    let re = Regex::new(r"[^\w\s]").unwrap();
    let cleaned = re.replace_all(input, "").to_lowercase();
    
    cleaned
        .split_whitespace()
        .filter(|word| !stop_words.contains(word))
        .collect::<Vec<&str>>()
        .join(" ")
}

pub fn create_default_stopwords() -> HashSet<&'static str> {
    let words = vec!["the", "a", "an", "and", "or", "but", "in", "on", "at"];
    words.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_and_normalize() {
        let stopwords = create_default_stopwords();
        let input = "The quick brown fox jumps over the lazy dog!";
        let result = clean_and_normalize(input, &stopwords);
        assert_eq!(result, "quick brown fox jumps over lazy dog");
    }

    #[test]
    fn test_with_punctuation() {
        let stopwords = create_default_stopwords();
        let input = "Hello, World! This is a test.";
        let result = clean_and_normalize(input, &stopwords);
        assert_eq!(result, "hello world this is test");
    }
}use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub fn remove_duplicates(input_path: &str, output_path: &str) -> io::Result<()> {
    let path = Path::new(input_path);
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut unique_lines = HashSet::new();
    let mut output_lines = Vec::new();

    for line_result in reader.lines() {
        let line = line_result?;
        if unique_lines.insert(line.clone()) {
            output_lines.push(line);
        }
    }

    let mut output_file = File::create(output_path)?;
    for line in output_lines {
        writeln!(output_file, "{}", line)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_remove_duplicates() {
        let input = "test_input.txt";
        let output = "test_output.txt";

        let test_data = "apple\nbanana\napple\ncherry\nbanana\ndate\n";
        fs::write(input, test_data).unwrap();

        remove_duplicates(input, output).unwrap();

        let result = fs::read_to_string(output).unwrap();
        let expected = "apple\nbanana\ncherry\ndate\n";
        assert_eq!(result, expected);

        fs::remove_file(input).unwrap();
        fs::remove_file(output).unwrap();
    }
}
use std::collections::HashSet;
use std::error::Error;

pub struct DataCleaner {
    unique_ids: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            unique_ids: HashSet::new(),
        }
    }

    pub fn deduplicate(&mut self, id: &str) -> bool {
        self.unique_ids.insert(id.to_string())
    }

    pub fn validate_email(email: &str) -> Result<(), Box<dyn Error>> {
        if email.is_empty() {
            return Err("Email cannot be empty".into());
        }
        
        if !email.contains('@') {
            return Err("Email must contain @ symbol".into());
        }
        
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err("Invalid email format".into());
        }
        
        if !parts[1].contains('.') {
            return Err("Email domain must contain a dot".into());
        }
        
        Ok(())
    }

    pub fn normalize_phone_number(phone: &str) -> String {
        phone
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect()
    }

    pub fn sanitize_input(input: &str) -> String {
        input
            .trim()
            .replace('\n', " ")
            .replace('\t', " ")
            .replace(|c: char| c.is_control(), "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("user123"));
        assert!(!cleaner.deduplicate("user123"));
        assert!(cleaner.deduplicate("user456"));
    }

    #[test]
    fn test_validate_email() {
        assert!(DataCleaner::validate_email("test@example.com").is_ok());
        assert!(DataCleaner::validate_email("invalid").is_err());
        assert!(DataCleaner::validate_email("test@").is_err());
        assert!(DataCleaner::validate_email("@example.com").is_err());
    }

    #[test]
    fn test_normalize_phone_number() {
        assert_eq!(
            DataCleaner::normalize_phone_number("+1 (555) 123-4567"),
            "15551234567"
        );
        assert_eq!(
            DataCleaner::normalize_phone_number("555.123.4567"),
            "5551234567"
        );
    }

    #[test]
    fn test_sanitize_input() {
        assert_eq!(
            DataCleaner::sanitize_input("  Hello\tWorld\n"),
            "Hello World"
        );
        assert_eq!(
            DataCleaner::sanitize_input("Text\x00with\x07control"),
            "Textwithcontrol"
        );
    }
}