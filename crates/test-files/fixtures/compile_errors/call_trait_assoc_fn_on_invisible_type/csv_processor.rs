use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

#[derive(Debug)]
pub enum CsvError {
    IoError(std::io::Error),
    ParseError(String),
    ValidationError(String),
}

impl From<std::io::Error> for CsvError {
    fn from(err: std::io::Error) -> Self {
        CsvError::IoError(err)
    }
}

pub struct CsvProcessor {
    delimiter: char,
    strict_mode: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, strict_mode: bool) -> Self {
        CsvProcessor {
            delimiter,
            strict_mode,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, CsvError> {
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

            let record = self.parse_line(&line_content, line_number)?;
            records.push(record);
        }

        if self.strict_mode && records.is_empty() {
            return Err(CsvError::ValidationError(
                "File contains no valid records".to_string()
            ));
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(format!(
                "Line {}: Expected 4 fields, found {}",
                line_number,
                parts.len()
            )));
        }

        let id = parts[0].parse::<u32>()
            .map_err(|_| CsvError::ParseError(format!(
                "Line {}: Invalid ID format '{}'",
                line_number, parts[0]
            )))?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Name cannot be empty",
                line_number
            )));
        }

        let value = parts[2].parse::<f64>()
            .map_err(|_| CsvError::ParseError(format!(
                "Line {}: Invalid value format '{}'",
                line_number, parts[2]
            )))?;

        let active = match parts[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(CsvError::ParseError(format!(
                "Line {}: Invalid boolean value '{}'",
                line_number, parts[3]
            ))),
        };

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    pub fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, usize) {
        if records.is_empty() {
            return (0.0, 0.0, 0);
        }

        let sum: f64 = records.iter().map(|r| r.value).sum();
        let count = records.len();
        let average = sum / count as f64;
        
        let variance: f64 = records.iter()
            .map(|r| (r.value - average).powi(2))
            .sum::<f64>() / count as f64;
        
        (average, variance.sqrt(), count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,John Doe,42.5,true").unwrap();
        writeln!(temp_file, "2,Jane Smith,37.8,false").unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "John Doe");
        assert_eq!(records[1].value, 37.8);
    }

    #[test]
    fn test_invalid_format() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,John Doe,not_a_number,true").unwrap();
        
        let processor = CsvProcessor::new(',', false);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            CsvRecord { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "Test2".to_string(), value: 20.0, active: false },
            CsvRecord { id: 3, name: "Test3".to_string(), value: 30.0, active: true },
        ];
        
        let (avg, std_dev, count) = CsvProcessor::calculate_statistics(&records);
        
        assert_eq!(count, 3);
        assert_eq!(avg, 20.0);
        assert!((std_dev - 8.164965).abs() < 0.0001);
    }
}