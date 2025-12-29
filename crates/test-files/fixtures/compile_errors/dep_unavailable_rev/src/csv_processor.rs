use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum CsvError {
    IoError(std::io::Error),
    ParseError(String, usize),
    InvalidHeader(String),
    EmptyFile,
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsvError::IoError(e) => write!(f, "IO error: {}", e),
            CsvError::ParseError(msg, line) => write!(f, "Parse error at line {}: {}", line, msg),
            CsvError::InvalidHeader(msg) => write!(f, "Invalid header: {}", msg),
            CsvError::EmptyFile => write!(f, "File is empty"),
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

impl Default for CsvProcessor {
    fn default() -> Self {
        CsvProcessor {
            delimiter: ',',
            has_header: true,
        }
    }
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let mut records = Vec::new();
        let mut line_number = 0;

        if let Some(first_line) = lines.next() {
            let first_line = first_line?;
            line_number += 1;
            
            if first_line.trim().is_empty() {
                return Err(CsvError::EmptyFile);
            }

            let headers: Vec<String> = first_line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if headers.iter().any(|h| h.is_empty()) {
                return Err(CsvError::InvalidHeader("Empty column name found".to_string()));
            }

            if self.has_header {
                records.push(headers);
            } else {
                records.push(vec!["".to_string(); headers.len()]);
                records.push(headers);
            }
        } else {
            return Err(CsvError::EmptyFile);
        }

        for line_result in lines {
            let line = line_result?;
            line_number += 1;
            
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.len() != records[0].len() {
                return Err(CsvError::ParseError(
                    format!("Expected {} fields, found {}", records[0].len(), fields.len()),
                    line_number,
                ));
            }

            records.push(fields);
        }

        Ok(records)
    }

    pub fn validate_numeric_column(&self, records: &[Vec<String>], column_index: usize) -> Result<Vec<f64>, CsvError> {
        if records.is_empty() {
            return Err(CsvError::EmptyFile);
        }

        if column_index >= records[0].len() {
            return Err(CsvError::ParseError(
                format!("Column index {} out of bounds", column_index),
                0,
            ));
        }

        let start_index = if self.has_header { 1 } else { 0 };
        let mut numeric_values = Vec::new();

        for (i, record) in records.iter().enumerate().skip(start_index) {
            let value = &record[column_index];
            match value.parse::<f64>() {
                Ok(num) => numeric_values.push(num),
                Err(_) => {
                    return Err(CsvError::ParseError(
                        format!("Invalid numeric value '{}'", value),
                        i + 1,
                    ));
                }
            }
        }

        Ok(numeric_values)
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
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "John Doe,30,50000.50").unwrap();
        writeln!(temp_file, "Jane Smith,25,60000.75").unwrap();

        let processor = CsvProcessor::default();
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["name", "age", "salary"]);
    }

    #[test]
    fn test_numeric_validation() {
        let records = vec![
            vec!["name".to_string(), "value".to_string()],
            vec!["test1".to_string(), "42.5".to_string()],
            vec!["test2".to_string(), "100.0".to_string()],
        ];

        let processor = CsvProcessor::default();
        let result = processor.validate_numeric_column(&records, 1);
        
        assert!(result.is_ok());
        let values = result.unwrap();
        assert_eq!(values, vec![42.5, 100.0]);
    }

    #[test]
    fn test_invalid_numeric() {
        let records = vec![
            vec!["name".to_string(), "value".to_string()],
            vec!["test1".to_string(), "not_a_number".to_string()],
        ];

        let processor = CsvProcessor::default();
        let result = processor.validate_numeric_column(&records, 1);
        
        assert!(result.is_err());
    }
}