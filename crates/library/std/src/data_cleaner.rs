
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

    pub fn deduplicate(&mut self, input: &str) -> Option<String> {
        if self.dedupe_set.contains(input) {
            None
        } else {
            self.dedupe_set.insert(input.to_string());
            Some(input.to_string())
        }
    }

    pub fn normalize_whitespace(text: &str) -> String {
        text.split_whitespace().collect::<Vec<&str>>().join(" ")
    }

    pub fn trim_and_lowercase(text: &str) -> String {
        text.trim().to_lowercase()
    }

    pub fn clean_pipeline(&mut self, text: &str) -> Option<String> {
        let normalized = Self::normalize_whitespace(text);
        let processed = Self::trim_and_lowercase(&normalized);
        self.deduplicate(&processed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert_eq!(cleaner.deduplicate("test"), Some("test".to_string()));
        assert_eq!(cleaner.deduplicate("test"), None);
    }

    #[test]
    fn test_normalization() {
        assert_eq!(
            DataCleaner::normalize_whitespace("  hello   world  "),
            "hello world"
        );
    }

    #[test]
    fn test_pipeline() {
        let mut cleaner = DataCleaner::new();
        let result = cleaner.clean_pipeline("  Hello   World  ");
        assert_eq!(result, Some("hello world".to_string()));
        
        let duplicate = cleaner.clean_pipeline("  hello   world  ");
        assert_eq!(duplicate, None);
    }
}
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn remove_duplicates(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let reader = BufReader::new(input_file);
    let mut lines = reader.lines();
    
    let header = match lines.next() {
        Some(Ok(h)) => h,
        _ => return Err("Empty or invalid input file".into()),
    };
    
    let mut seen = HashSet::new();
    let mut unique_lines = Vec::new();
    
    for line_result in lines {
        let line = line_result?;
        if !seen.contains(&line) {
            seen.insert(line.clone());
            unique_lines.push(line);
        }
    }
    
    let mut output_file = File::create(Path::new(output_path))?;
    writeln!(output_file, "{}", header)?;
    
    for line in unique_lines {
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
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        
        let test_data = "id,name,value\n1,Alice,100\n2,Bob,200\n1,Alice,100\n3,Charlie,300\n2,Bob,200";
        fs::write(test_input, test_data).unwrap();
        
        remove_duplicates(test_input, test_output).unwrap();
        
        let result = fs::read_to_string(test_output).unwrap();
        let expected = "id,name,value\n1,Alice,100\n2,Bob,200\n3,Charlie,300\n";
        
        assert_eq!(result, expected);
        
        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}