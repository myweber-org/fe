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

            let record: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if record.is_empty() {
                return Err(CsvError::ParseError(
                    "Empty record found".to_string(),
                    line_number,
                ));
            }

            records.push(record);
        }

        if self.has_header && !records.is_empty() {
            let header = &records[0];
            if header.iter().any(|field| field.is_empty()) {
                return Err(CsvError::InvalidHeader(
                    "Header contains empty fields".to_string(),
                ));
            }
        }

        Ok(records)
    }

    pub fn validate_numeric_column(&self, records: &[Vec<String>], column_index: usize) -> Result<(), CsvError> {
        if records.is_empty() {
            return Ok(());
        }

        let start_index = if self.has_header { 1 } else { 0 };
        
        for (i, record) in records.iter().enumerate().skip(start_index) {
            if column_index >= record.len() {
                return Err(CsvError::ParseError(
                    format!("Column index {} out of bounds", column_index),
                    i + 1,
                ));
            }

            let value = &record[column_index];
            if value.parse::<f64>().is_err() {
                return Err(CsvError::ParseError(
                    format!("Non-numeric value '{}' in column {}", value, column_index),
                    i + 1,
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
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["name", "age", "city"]);
    }

    #[test]
    fn test_numeric_validation() {
        let records = vec![
            vec!["name".to_string(), "age".to_string()],
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "twenty-five".to_string()],
        ];

        let processor = CsvProcessor::new(',', true);
        let result = processor.validate_numeric_column(&records, 1);
        
        assert!(result.is_err());
    }
}