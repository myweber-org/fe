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
}use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid number of fields".into());
        }

        let id = parts[0].parse()?;
        let name = parts[1].to_string();
        let value = parts[2].parse()?;
        let active = parts[3].parse()?;

        Ok(Record {
            id,
            name,
            value,
            active,
        })
    }

    fn to_csv_line(&self) -> String {
        format!("{},{},{},{}", self.id, self.name, self.value, self.active)
    }
}

fn process_csv_file(input_path: &Path, output_path: &Path, min_value: f64) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut output_file = File::create(output_path)?;

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        match Record::from_csv_line(&line) {
            Ok(record) => {
                if record.value >= min_value && record.active {
                    writeln!(output_file, "{}", record.to_csv_line())?;
                }
            }
            Err(e) => {
                eprintln!("Warning: Line {}: {} - {}", line_num + 1, e, line);
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = Path::new("data/input.csv");
    let output_path = Path::new("data/filtered.csv");
    let min_value = 100.0;

    if !input_path.exists() {
        return Err("Input file does not exist".into());
    }

    process_csv_file(input_path, output_path, min_value)?;
    println!("Processing completed successfully");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_parsing() {
        let record = Record::from_csv_line("42,Test Item,150.5,true").unwrap();
        assert_eq!(record.id, 42);
        assert_eq!(record.name, "Test Item");
        assert_eq!(record.value, 150.5);
        assert_eq!(record.active, true);
    }

    #[test]
    fn test_invalid_record() {
        let result = Record::from_csv_line("invalid,data");
        assert!(result.is_err());
    }

    #[test]
    fn test_csv_processing() -> Result<(), Box<dyn Error>> {
        let mut input_file = NamedTempFile::new()?;
        writeln!(input_file, "1,Item A,50.0,true")?;
        writeln!(input_file, "2,Item B,150.0,true")?;
        writeln!(input_file, "3,Item C,200.0,false")?;
        writeln!(input_file, "# Comment line")?;
        writeln!(input_file, "")?;

        let output_file = NamedTempFile::new()?;
        
        process_csv_file(input_file.path(), output_file.path(), 100.0)?;
        
        let output_content = std::fs::read_to_string(output_file.path())?;
        assert!(output_content.contains("Item B"));
        assert!(!output_content.contains("Item A"));
        assert!(!output_content.contains("Item C"));
        
        Ok(())
    }
}