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

            let first_record = self.parse_line(&first_line, line_number)?;
            
            if self.has_header {
                if first_record.iter().any(|field| field.trim().is_empty()) {
                    return Err(CsvError::InvalidHeader(
                        "Header contains empty fields".to_string()
                    ));
                }
            } else {
                records.push(first_record);
            }
        } else {
            return Err(CsvError::EmptyFile);
        }

        for line in lines {
            let line = line?;
            line_number += 1;
            
            if line.trim().is_empty() {
                continue;
            }
            
            let record = self.parse_line(&line, line_number)?;
            records.push(record);
        }

        if records.is_empty() {
            return Err(CsvError::EmptyFile);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<Vec<String>, CsvError> {
        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '"' => {
                    if in_quotes {
                        if chars.peek() == Some(&'"') {
                            current_field.push('"');
                            chars.next();
                        } else {
                            in_quotes = false;
                        }
                    } else {
                        in_quotes = true;
                    }
                }
                _ if ch == self.delimiter && !in_quotes => {
                    fields.push(current_field.trim().to_string());
                    current_field.clear();
                }
                _ => {
                    current_field.push(ch);
                }
            }
        }

        fields.push(current_field.trim().to_string());

        if in_quotes {
            return Err(CsvError::ParseError(
                "Unclosed quotes in field".to_string(),
                line_number,
            ));
        }

        Ok(fields)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), CsvError> {
        if records.is_empty() {
            return Err(CsvError::EmptyFile);
        }

        let expected_len = records[0].len();
        
        for (i, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(CsvError::ParseError(
                    format!("Expected {} fields, found {}", expected_len, record.len()),
                    i + if self.has_header { 2 } else { 1 },
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
    fn test_basic_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Jane,25,London").unwrap();

        let processor = CsvProcessor::default();
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_csv_with_quotes() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,description").unwrap();
        writeln!(temp_file, "John,\"Software, Engineer\"").unwrap();

        let processor = CsvProcessor::default();
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records[0], vec!["John", "Software, Engineer"]);
    }

    #[test]
    fn test_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let processor = CsvProcessor::default();
        let result = processor.process_file(temp_file.path());
        
        assert!(matches!(result, Err(CsvError::EmptyFile)));
    }
}