use std::collections::HashSet;
use std::hash::Hash;

pub fn deduplicate<T: Eq + Hash + Clone>(items: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in items {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    result
}

pub fn normalize_strings(strings: Vec<String>) -> Vec<String> {
    strings
        .into_iter()
        .map(|s| s.trim().to_lowercase())
        .collect()
}

pub fn filter_empty(strings: Vec<String>) -> Vec<String> {
    strings
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn clean_data(strings: Vec<String>) -> Vec<String> {
    let normalized = normalize_strings(strings);
    let filtered = filter_empty(normalized);
    deduplicate(filtered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate() {
        let input = vec![1, 2, 2, 3, 4, 4, 5];
        let result = deduplicate(input);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_normalize_strings() {
        let input = vec!["  HELLO  ".to_string(), "World".to_string()];
        let result = normalize_strings(input);
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_filter_empty() {
        let input = vec!["hello".to_string(), "".to_string(), "world".to_string()];
        let result = filter_empty(input);
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_clean_data() {
        let input = vec![
            "  Apple  ".to_string(),
            "apple".to_string(),
            "".to_string(),
            "Banana  ".to_string(),
            "banana".to_string(),
        ];
        let result = clean_data(input);
        assert_eq!(result, vec!["apple", "banana"]);
    }
}use std::collections::HashSet;
use std::error::Error;

pub struct DataCleaner {
    records: Vec<String>,
    seen: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
            seen: HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: &str) -> Result<(), Box<dyn Error>> {
        let trimmed = record.trim();
        
        if trimmed.is_empty() {
            return Err("Empty record not allowed".into());
        }

        if trimmed.len() > 1000 {
            return Err("Record exceeds maximum length".into());
        }

        if self.seen.contains(trimmed) {
            return Err("Duplicate record detected".into());
        }

        self.seen.insert(trimmed.to_string());
        self.records.push(trimmed.to_string());
        Ok(())
    }

    pub fn get_clean_records(&self) -> &Vec<String> {
        &self.records
    }

    pub fn validate_all(&self) -> bool {
        !self.records.is_empty() && self.records.len() == self.seen.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test").unwrap();
        assert!(cleaner.add_record("test").is_err());
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid").unwrap();
        assert!(cleaner.validate_all());
    }
}
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn remove_duplicates(input_path: &str, output_path: &str) -> Result<usize, Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let reader = BufReader::new(input_file);
    
    let mut seen_lines = HashSet::new();
    let mut unique_lines = Vec::new();
    let mut duplicate_count = 0;
    
    for line_result in reader.lines() {
        let line = line_result?;
        
        if seen_lines.contains(&line) {
            duplicate_count += 1;
        } else {
            seen_lines.insert(line.clone());
            unique_lines.push(line);
        }
    }
    
    let mut output_file = File::create(Path::new(output_path))?;
    
    for line in unique_lines {
        writeln!(output_file, "{}", line)?;
    }
    
    Ok(duplicate_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_remove_duplicates() {
        let input_content = "id,name,value\n1,test,100\n2,example,200\n1,test,100\n3,sample,300\n2,example,200";
        
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), input_content).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let duplicates = remove_duplicates(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        ).unwrap();
        
        assert_eq!(duplicates, 2);
        
        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let expected = "id,name,value\n1,test,100\n2,example,200\n3,sample,300\n";
        
        assert_eq!(output_content, expected);
    }
}