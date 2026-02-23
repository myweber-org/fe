use std::collections::HashSet;

pub struct DataCleaner {
    pub remove_duplicates: bool,
    pub normalize_case: bool,
}

impl DataCleaner {
    pub fn new(remove_duplicates: bool, normalize_case: bool) -> Self {
        DataCleaner {
            remove_duplicates,
            normalize_case,
        }
    }

    pub fn clean(&self, data: Vec<String>) -> Vec<String> {
        let mut processed = data;

        if self.normalize_case {
            processed = processed
                .into_iter()
                .map(|s| s.to_lowercase())
                .collect();
        }

        if self.remove_duplicates {
            let unique_set: HashSet<String> = processed.into_iter().collect();
            processed = unique_set.into_iter().collect();
        }

        processed.sort();
        processed
    }

    pub fn clean_with_callback<F>(&self, data: Vec<String>, mut callback: F) -> Vec<String>
    where
        F: FnMut(&str),
    {
        let cleaned = self.clean(data);
        
        for item in &cleaned {
            callback(item);
        }
        
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_removes_duplicates() {
        let cleaner = DataCleaner::new(true, false);
        let input = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];
        
        let result = cleaner.clean(input);
        assert_eq!(result.len(), 3);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_cleaner_normalizes_case() {
        let cleaner = DataCleaner::new(false, true);
        let input = vec![
            "Apple".to_string(),
            "BANANA".to_string(),
            "Cherry".to_string(),
        ];
        
        let result = cleaner.clean(input);
        assert!(result.iter().all(|s| s.chars().all(|c| c.is_lowercase())));
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataCleaner {
    file_path: String,
    delimiter: char,
}

impl DataCleaner {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        DataCleaner {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn validate_csv(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut errors = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line?;
            
            if line_content.trim().is_empty() {
                continue;
            }

            let fields: Vec<&str> = line_content.split(self.delimiter).collect();
            
            if fields.len() < 2 {
                errors.push(format!("Line {}: Insufficient columns, found {}", line_number, fields.len()));
                continue;
            }

            for (idx, field) in fields.iter().enumerate() {
                if field.trim().is_empty() {
                    errors.push(format!("Line {}: Empty field at column {}", line_number, idx + 1));
                }
            }
        }

        Ok(errors)
    }

    pub fn get_record_count(&self) -> Result<usize, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            let line_content = line?;
            if !line_content.trim().is_empty() {
                count += 1;
            }
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value").unwrap();
        writeln!(temp_file, "1,test,100").unwrap();
        writeln!(temp_file, "2,sample,200").unwrap();

        let cleaner = DataCleaner::new(temp_file.path().to_str().unwrap(), ',');
        let errors = cleaner.validate_csv().unwrap();
        assert!(errors.is_empty());
        
        let count = cleaner.get_record_count().unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_invalid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name").unwrap();
        writeln!(temp_file, "1,").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,data").unwrap();

        let cleaner = DataCleaner::new(temp_file.path().to_str().unwrap(), ',');
        let errors = cleaner.validate_csv().unwrap();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Empty field at column 2"));
        
        let count = cleaner.get_record_count().unwrap();
        assert_eq!(count, 3);
    }
}