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
                self.validate_header(&first_record)?;
            } else {
                records.push(first_record);
            }
        } else {
            return Err(CsvError::EmptyFile);
        }

        for line_result in lines {
            let line = line_result?;
            line_number += 1;
            
            if line.trim().is_empty() {
                continue;
            }
            
            let record = self.parse_line(&line, line_number)?;
            records.push(record);
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
                    if in_quotes && chars.peek() == Some(&'"') {
                        current_field.push('"');
                        chars.next();
                    } else {
                        in_quotes = !in_quotes;
                    }
                }
                c if c == self.delimiter && !in_quotes => {
                    fields.push(current_field.trim().to_string());
                    current_field.clear();
                }
                _ => {
                    current_field.push(ch);
                }
            }
        }

        if in_quotes {
            return Err(CsvError::ParseError(
                "Unclosed quotation mark".to_string(),
                line_number,
            ));
        }

        fields.push(current_field.trim().to_string());
        Ok(fields)
    }

    fn validate_header(&self, header: &[String]) -> Result<(), CsvError> {
        if header.is_empty() {
            return Err(CsvError::InvalidHeader("Header cannot be empty".to_string()));
        }

        let mut seen_fields = std::collections::HashSet::new();
        for field in header {
            if field.trim().is_empty() {
                return Err(CsvError::InvalidHeader(
                    "Header fields cannot be empty".to_string(),
                ));
            }
            
            if seen_fields.contains(field) {
                return Err(CsvError::InvalidHeader(
                    format!("Duplicate header field: {}", field),
                ));
            }
            
            seen_fields.insert(field.clone());
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
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_csv_with_quotes() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,description").unwrap();
        writeln!(temp_file, "1,\"Item with, comma\"").unwrap();
        writeln!(temp_file, "2,\"Item with \"\"quotes\"\" inside\"").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records[0], vec!["1", "Item with, comma"]);
        assert_eq!(records[1], vec!["2", "Item with \"quotes\" inside"]);
    }

    #[test]
    fn test_invalid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "field1,field2").unwrap();
        writeln!(temp_file, "\"unclosed quote,value2").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_err());
        match result.unwrap_err() {
            CsvError::ParseError(_, line) => assert_eq!(line, 2),
            _ => panic!("Expected ParseError"),
        }
    }
}