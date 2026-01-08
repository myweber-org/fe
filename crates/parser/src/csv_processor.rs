use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub enum CsvError {
    IoError(std::io::Error),
    ParseError(String, usize),
    InvalidHeader(String),
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsvError::IoError(e) => write!(f, "IO error: {}", e),
            CsvError::ParseError(msg, line) => write!(f, "Parse error at line {}: {}", line, msg),
            CsvError::InvalidHeader(msg) => write!(f, "Invalid header: {}", msg),
        }
    }
}

impl Error for CsvError {}

impl From<std::io::Error> for CsvError {
    fn from(error: std::io::Error) -> Self {
        CsvError::IoError(error)
    }
}

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file(&self, file_path: &str) -> Result<Vec<Vec<String>>, CsvError> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line?;
            
            if line_content.trim().is_empty() {
                continue;
            }

            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.is_empty() {
                return Err(CsvError::ParseError(
                    "Empty line with content".to_string(),
                    line_number,
                ));
            }

            if self.has_header && line_number == 1 {
                if fields.iter().any(|f| f.is_empty()) {
                    return Err(CsvError::InvalidHeader(
                        "Header contains empty fields".to_string(),
                    ));
                }
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err(CsvError::ParseError(
                "File contains no valid data".to_string(),
                0,
            ));
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), CsvError> {
        if records.is_empty() {
            return Ok(());
        }

        let expected_len = records[0].len();
        
        for (idx, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(CsvError::ParseError(
                    format!("Record has {} fields, expected {}", record.len(), expected_len),
                    idx + 1,
                ));
            }
        }

        Ok(())
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
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Jane,25,London").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_invalid_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,,city").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path().to_str().unwrap());
        
        assert!(matches!(result, Err(CsvError::InvalidHeader(_))));
    }
}