use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub struct LogParser {
    file_path: String,
}

impl LogParser {
    pub fn new(file_path: &str) -> Self {
        LogParser {
            file_path: file_path.to_string(),
        }
    }

    pub fn extract_errors(&self) -> io::Result<Vec<String>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut errors = Vec::new();
        
        for line in reader.lines() {
            let line = line?;
            if line.contains("ERROR") || line.contains("error") {
                errors.push(line);
            }
        }
        
        Ok(errors)
    }
    
    pub fn count_errors_by_type(&self) -> io::Result<std::collections::HashMap<String, usize>> {
        let errors = self.extract_errors()?;
        let mut error_counts = std::collections::HashMap::new();
        
        for error in errors {
            let parts: Vec<&str> = error.split_whitespace().collect();
            if parts.len() > 2 {
                let error_type = parts[2].to_string();
                *error_counts.entry(error_type).or_insert(0) += 1;
            }
        }
        
        Ok(error_counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_extract_errors() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "INFO: Application started").unwrap();
        writeln!(temp_file, "ERROR: Database connection failed").unwrap();
        writeln!(temp_file, "WARN: High memory usage").unwrap();
        writeln!(temp_file, "error: File not found").unwrap();
        
        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.extract_errors().unwrap();
        
        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("Database connection failed"));
        assert!(errors[1].contains("File not found"));
    }
}