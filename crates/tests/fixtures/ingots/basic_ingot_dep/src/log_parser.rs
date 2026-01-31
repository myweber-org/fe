use std::fs::File;
use std::io::{BufRead, BufReader};
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

    pub fn extract_errors(&self) -> Result<Vec<String>, String> {
        let path = Path::new(&self.file_path);
        if !path.exists() {
            return Err(format!("File not found: {}", self.file_path));
        }

        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let mut errors = Vec::new();

        for (line_number, line_result) in reader.lines().enumerate() {
            let line = line_result.map_err(|e| e.to_string())?;
            if line.to_lowercase().contains("error") {
                errors.push(format!("Line {}: {}", line_number + 1, line));
            }
        }

        Ok(errors)
    }

    pub fn count_errors(&self) -> Result<usize, String> {
        let errors = self.extract_errors()?;
        Ok(errors.len())
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
        writeln!(temp_file, "ERROR: File not found").unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let errors = parser.extract_errors().unwrap();

        assert_eq!(errors.len(), 2);
        assert!(errors[0].contains("Database connection failed"));
        assert!(errors[1].contains("File not found"));
    }

    #[test]
    fn test_count_errors() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "ERROR: Test error 1").unwrap();
        writeln!(temp_file, "ERROR: Test error 2").unwrap();
        writeln!(temp_file, "INFO: Normal log").unwrap();

        let parser = LogParser::new(temp_file.path().to_str().unwrap());
        let count = parser.count_errors().unwrap();

        assert_eq!(count, 2);
    }

    #[test]
    fn test_file_not_found() {
        let parser = LogParser::new("non_existent_file.log");
        let result = parser.extract_errors();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("File not found"));
    }
}