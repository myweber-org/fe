use std::error::Error;
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

            if fields.is_empty() {
                return Err(CsvError::ParseError(
                    "Empty record found".to_string(),
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
                "File contains no data".to_string(),
                0,
            ));
        }

        Ok(records)
    }

    fn validate_records(&self, records: &[Vec<String>]) -> Result<(), CsvError> {
        if records.is_empty() {
            return Ok(());
        }

        let expected_len = records[0].len();
        
        for (idx, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(CsvError::ParseError(
                    format!("Record has {} fields, expected {}", record.len(), expected_len),
                    if self.has_header { idx + 2 } else { idx + 1 },
                ));
            }
        }
        
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let processor = CsvProcessor::new(',', true);
    
    match processor.process_file("data.csv") {
        Ok(records) => {
            if let Err(e) = processor.validate_records(&records) {
                eprintln!("Validation error: {}", e);
                return Err(Box::new(e));
            }
            
            println!("Successfully processed {} records", records.len());
            
            if !records.is_empty() {
                println!("First record: {:?}", records[0]);
            }
        }
        Err(e) => {
            eprintln!("Processing error: {}", e);
            return Err(Box::new(e));
        }
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
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
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
        let result = processor.process_file(temp_file.path());
        
        assert!(matches!(result, Err(CsvError::InvalidHeader(_))));
    }

    #[test]
    fn test_missing_fields() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30").unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor.process_file(temp_file.path()).unwrap();
        let result = processor.validate_records(&records);
        
        assert!(matches!(result, Err(CsvError::ParseError(_, _))));
    }
}