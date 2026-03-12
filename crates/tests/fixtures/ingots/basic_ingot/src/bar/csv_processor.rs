use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if record.value < 0.0 {
        return Err("Value must be non-negative".to_string());
    }
    Ok(())
}

pub fn process_csv_file(path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = Reader::from_reader(file);
    let mut valid_records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        match validate_record(&record) {
            Ok(_) => valid_records.push(record),
            Err(e) => eprintln!("Invalid record skipped: {}", e),
        }
    }

    Ok(valid_records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Test Item,42.5,true").unwrap();
        writeln!(temp_file, "2,Another Item,100.0,false").unwrap();

        let records = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "Test Item");
        assert_eq!(records[1].value, 100.0);
    }

    #[test]
    fn test_invalid_data_skipping() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,,42.5,true").unwrap();
        writeln!(temp_file, "2,Valid Name,-10.0,false").unwrap();

        let records = process_csv_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum CsvError {
    IoError(String),
    ParseError(String),
    ValidationError(String),
}

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvError::IoError(msg) => write!(f, "IO error: {}", msg),
            CsvError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            CsvError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for CsvError {}

impl From<std::io::Error> for CsvError {
    fn from(err: std::io::Error) -> Self {
        CsvError::IoError(err.to_string())
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
        
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line = line?;
            
            if line.trim().is_empty() {
                continue;
            }

            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if record.is_empty() {
                return Err(CsvError::ParseError(
                    format!("Empty record at line {}", line_number)
                ));
            }

            if self.has_header && line_number == 1 {
                continue;
            }

            self.validate_record(&record, line_number)?;
            records.push(record);
        }

        if records.is_empty() {
            return Err(CsvError::ValidationError(
                "No valid records found in file".to_string()
            ));
        }

        Ok(records)
    }

    fn validate_record(&self, record: &[String], line_number: usize) -> Result<(), CsvError> {
        for (i, field) in record.iter().enumerate() {
            if field.is_empty() {
                return Err(CsvError::ValidationError(
                    format!("Empty field at column {} in line {}", i + 1, line_number)
                ));
            }
            
            if field.contains('\n') || field.contains('\r') {
                return Err(CsvError::ValidationError(
                    format!("Newline character in field at column {} in line {}", i + 1, line_number)
                ));
            }
        }
        Ok(())
    }

    pub fn count_records<P: AsRef<Path>>(&self, path: P) -> Result<usize, CsvError> {
        let records = self.process_file(path)?;
        Ok(records.len())
    }

    pub fn get_column<P: AsRef<Path>>(&self, path: P, column_index: usize) -> Result<Vec<String>, CsvError> {
        let records = self.process_file(path)?;
        
        if column_index >= records[0].len() {
            return Err(CsvError::ValidationError(
                format!("Column index {} out of bounds", column_index)
            ));
        }

        let column_data: Vec<String> = records
            .iter()
            .map(|record| record[column_index].clone())
            .collect();

        Ok(column_data)
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
        writeln!(temp_file, "Charlie,35,Paris").unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_empty_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "").unwrap();

        let processor = CsvProcessor::new(',', false);
        let result = processor.process_file(temp_file.path());
        
        assert!(matches!(result, Err(CsvError::ValidationError(_))));
    }

    #[test]
    fn test_column_extraction() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = CsvProcessor::new(',', true);
        let names = processor.get_column(temp_file.path(), 0).unwrap();
        
        assert_eq!(names, vec!["Alice", "Bob"]);
    }
}