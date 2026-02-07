use std::fs::File;
use std::io::{self, BufRead, BufReader};
use regex::Regex;

pub fn extract_errors(log_path: &str) -> io::Result<Vec<String>> {
    let file = File::open(log_path)?;
    let reader = BufReader::new(file);
    let error_pattern = Regex::new(r"ERROR.*").unwrap();
    
    let mut errors = Vec::new();
    
    for line in reader.lines() {
        let line = line?;
        if error_pattern.is_match(&line) {
            errors.push(line);
        }
    }
    
    Ok(errors)
}

pub fn count_errors_by_source(log_path: &str) -> io::Result<std::collections::HashMap<String, usize>> {
    let file = File::open(log_path)?;
    let reader = BufReader::new(file);
    let source_pattern = Regex::new(r"ERROR.*\[(.*?)\]").unwrap();
    
    let mut error_counts = std::collections::HashMap::new();
    
    for line in reader.lines() {
        let line = line?;
        if let Some(captures) = source_pattern.captures(&line) {
            if let Some(source) = captures.get(1) {
                *error_counts.entry(source.as_str().to_string()).or_insert(0) += 1;
            }
        }
    }
    
    Ok(error_counts)
}