use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

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
    expected_columns: usize,
}

impl CsvProcessor {
    fn new(delimiter: char, expected_columns: usize) -> Self {
        CsvProcessor {
            delimiter,
            expected_columns,
        }
    }

    fn process_file(&self, path: &str) -> Result<Vec<Vec<String>>, CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            let record = self.parse_line(&line, line_num + 1)?;
            records.push(record);
        }

        if !records.is_empty() {
            self.validate_header(&records[0])?;
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<Vec<String>, CsvError> {
        let fields: Vec<String> = line
            .split(self.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        if fields.len() != self.expected_columns {
            return Err(CsvError::ParseError(
                format!(
                    "Expected {} columns, found {}",
                    self.expected_columns,
                    fields.len()
                ),
                line_num,
            ));
        }

        Ok(fields)
    }

    fn validate_header(&self, header: &[String]) -> Result<(), CsvError> {
        for (idx, field) in header.iter().enumerate() {
            if field.is_empty() {
                return Err(CsvError::InvalidHeader(format!(
                    "Empty header field at position {}",
                    idx + 1
                )));
            }
        }
        Ok(())
    }

    fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Vec<String> {
        records
            .iter()
            .skip(1)
            .filter_map(|record| record.get(column_index).cloned())
            .collect()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let processor = CsvProcessor::new(',', 3);
    
    match processor.process_file("data.csv") {
        Ok(records) => {
            println!("Successfully processed {} records", records.len());
            
            if records.len() > 1 {
                let first_column = processor.extract_column(&records, 0);
                println!("First column values: {:?}", first_column);
            }
        }
        Err(e) => eprintln!("Processing failed: {}", e),
    }

    Ok(())
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
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = CsvProcessor::new(',', 3);
        let result = processor.process_file(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_invalid_column_count() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30").unwrap();

        let processor = CsvProcessor::new(',', 3);
        let result = processor.process_file(temp_file.path().to_str().unwrap());
        
        assert!(result.is_err());
    }
}