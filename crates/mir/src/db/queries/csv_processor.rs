
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

        if self.has_header {
            if let Some(header_line) = lines.next() {
                let header = header_line?;
                self.validate_header(&header)?;
                line_number += 1;
            } else {
                return Err(CsvError::EmptyFile);
            }
        }

        for line_result in lines {
            line_number += 1;
            let line = line_result?;
            
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

    fn validate_header(&self, header: &str) -> Result<(), CsvError> {
        let columns: Vec<&str> = header.split(self.delimiter).collect();
        
        if columns.is_empty() {
            return Err(CsvError::InvalidHeader("Header has no columns".to_string()));
        }

        for (i, col) in columns.iter().enumerate() {
            if col.trim().is_empty() {
                return Err(CsvError::InvalidHeader(
                    format!("Column {} in header is empty", i + 1)
                ));
            }
        }

        Ok(())
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<Vec<String>, CsvError> {
        let mut record = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            if ch == '"' {
                if in_quotes && i + 1 < chars.len() && chars[i + 1] == '"' {
                    current_field.push('"');
                    i += 1;
                } else {
                    in_quotes = !in_quotes;
                }
            } else if ch == self.delimiter && !in_quotes {
                record.push(current_field.trim().to_string());
                current_field.clear();
            } else {
                current_field.push(ch);
            }

            i += 1;
        }

        record.push(current_field.trim().to_string());

        if in_quotes {
            return Err(CsvError::ParseError(
                "Unclosed quotes in field".to_string(),
                line_number
            ));
        }

        Ok(record)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), CsvError> {
        if records.is_empty() {
            return Err(CsvError::EmptyFile);
        }

        let expected_columns = records[0].len();
        
        for (i, record) in records.iter().enumerate() {
            if record.len() != expected_columns {
                return Err(CsvError::ParseError(
                    format!("Expected {} columns, found {}", expected_columns, record.len()),
                    i + 1 + if self.has_header { 1 } else { 0 }
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
        writeln!(temp_file, "John Doe,30,New York").unwrap();
        writeln!(temp_file, "Jane Smith,25,London").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["John Doe", "30", "New York"]);
    }

    #[test]
    fn test_quoted_fields() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "data").unwrap();
        writeln!(temp_file, "\"Value, with comma\",normal").unwrap();

        let processor = CsvProcessor::new(',', false);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records[0], vec!["Value, with comma", "normal"]);
    }

    #[test]
    fn test_invalid_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "col1,,col3").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(matches!(result, Err(CsvError::InvalidHeader(_))));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

    pub fn validate_file(&self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut line_count = 0;
        let mut column_count: Option<usize> = None;

        for (index, line) in reader.lines().enumerate() {
            let line_content = line?;
            let columns: Vec<&str> = line_content.split(self.delimiter).collect();
            
            if index == 0 && self.has_header {
                continue;
            }

            match column_count {
                Some(expected) => {
                    if columns.len() != expected {
                        return Err(format!("Line {} has {} columns, expected {}", 
                            index + 1, columns.len(), expected).into());
                    }
                }
                None => {
                    column_count = Some(columns.len());
                }
            }

            for (col_idx, value) in columns.iter().enumerate() {
                if value.trim().is_empty() {
                    return Err(format!("Empty value at line {}, column {}", 
                        index + 1, col_idx + 1).into());
                }
            }

            line_count += 1;
        }

        if line_count == 0 {
            return Err("File contains no data rows".into());
        }

        Ok(line_count)
    }

    pub fn transform_column(&self, file_path: &str, column_index: usize, 
                           transform_fn: fn(&str) -> String) -> Result<Vec<String>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line_content = line?;
            
            if index == 0 && self.has_header {
                continue;
            }

            let columns: Vec<&str> = line_content.split(self.delimiter).collect();
            
            if column_index >= columns.len() {
                return Err(format!("Column index {} out of bounds for line {}", 
                    column_index, index + 1).into());
            }

            let transformed = transform_fn(columns[column_index]);
            results.push(transformed);
        }

        Ok(results)
    }
}

fn uppercase_transform(value: &str) -> String {
    value.to_uppercase()
}

fn numeric_validation(value: &str) -> String {
    if value.parse::<f64>().is_ok() {
        String::from("VALID_NUMERIC")
    } else {
        String::from("INVALID_NUMERIC")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_validation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,25,New York").unwrap();
        writeln!(temp_file, "Alice,30,London").unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.validate_file(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_column_transformation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age").unwrap();
        writeln!(temp_file, "john,25").unwrap();
        writeln!(temp_file, "alice,30").unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.transform_column(
            temp_file.path().to_str().unwrap(),
            0,
            uppercase_transform
        ).unwrap();
        
        assert_eq!(result, vec!["JOHN", "ALICE"]);
    }
}use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
enum CsvError {
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

struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, CsvError> {
        let file = File::open(path)?;
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

            if self.has_header && line_number == 1 {
                if fields.iter().any(|f| f.is_empty()) {
                    return Err(CsvError::InvalidHeader(
                        "Header contains empty fields".to_string()
                    ));
                }
            }

            if fields.len() != 3 {
                return Err(CsvError::ParseError(
                    format!("Expected 3 fields, found {}", fields.len()),
                    line_number
                ));
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err(CsvError::ParseError(
                "File contains no valid data".to_string(),
                0
            ));
        }

        Ok(records)
    }

    fn validate_numeric_field(&self, records: &[Vec<String>], field_index: usize) -> Result<(), CsvError> {
        let start_index = if self.has_header { 1 } else { 0 };
        
        for (i, record) in records.iter().enumerate().skip(start_index) {
            if let Some(field) = record.get(field_index) {
                if field.parse::<f64>().is_err() {
                    return Err(CsvError::ParseError(
                        format!("Field {} is not numeric: '{}'", field_index, field),
                        i + 1
                    ));
                }
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let processor = CsvProcessor::new(',', true);
    
    match processor.process_file("data.csv") {
        Ok(records) => {
            println!("Successfully processed {} records", records.len());
            
            if let Err(e) = processor.validate_numeric_field(&records, 2) {
                eprintln!("Validation error: {}", e);
            }
            
            for record in records.iter().take(5) {
                println!("{:?}", record);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Processing failed: {}", e);
            Err(Box::new(e))
        }
    }
}