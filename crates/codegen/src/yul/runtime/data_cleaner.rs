use std::collections::HashSet;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn normalize_text(&self, input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        if self.dedupe_set.contains(&normalized) {
            false
        } else {
            self.dedupe_set.insert(normalized);
            true
        }
    }

    pub fn clean_dataset(&mut self, data: Vec<&str>) -> Vec<String> {
        let mut cleaned = Vec::new();
        for item in data {
            if self.deduplicate(item) {
                cleaned.push(self.normalize_text(item));
            }
        }
        cleaned
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        let data = vec!["Apple", "apple", "APPLE", "Banana", "banana"];
        let cleaned = cleaner.clean_dataset(data);
        
        assert_eq!(cleaned.len(), 2);
        assert_eq!(cleaner.get_unique_count(), 2);
        assert!(cleaned.contains(&"apple".to_string()));
        assert!(cleaned.contains(&"banana".to_string()));
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  HELLO World  "), "hello world");
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct DataCleaner {
    input_path: String,
    output_path: String,
    delimiter: char,
}

impl DataCleaner {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        DataCleaner {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            delimiter: ',',
        }
    }

    pub fn set_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn clean(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;

        let mut cleaned_count = 0;
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line = line?;

            if self.is_valid_record(&line) {
                writeln!(output_file, "{}", line)?;
                cleaned_count += 1;
            } else {
                eprintln!("Warning: Skipping invalid record at line {}", line_number);
            }
        }

        Ok(cleaned_count)
    }

    fn is_valid_record(&self, record: &str) -> bool {
        let fields: Vec<&str> = record.split(self.delimiter).collect();
        
        if fields.len() < 2 {
            return false;
        }

        for field in fields {
            if field.trim().is_empty() {
                return false;
            }
        }

        true
    }
}

pub fn validate_csv_file(path: &str) -> Result<bool, Box<dyn Error>> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err("File does not exist".into());
    }

    let metadata = path_obj.metadata()?;
    if metadata.len() == 0 {
        return Err("File is empty".into());
    }

    let extension = path_obj.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    
    if extension.to_lowercase() != "csv" {
        return Err("File is not a CSV".into());
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_cleaner_valid_record() {
        let cleaner = DataCleaner::new("test_input.csv", "test_output.csv");
        assert!(cleaner.is_valid_record("field1,field2,field3"));
        assert!(!cleaner.is_valid_record("field1,,field3"));
        assert!(!cleaner.is_valid_record("single_field"));
    }

    #[test]
    fn test_file_validation() {
        let temp_file = "temp_test.csv";
        let mut file = File::create(temp_file).unwrap();
        writeln!(file, "test,data").unwrap();

        assert!(validate_csv_file(temp_file).is_ok());
        
        fs::remove_file(temp_file).unwrap();
    }
}