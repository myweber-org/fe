
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

impl Default for CsvProcessor {
    fn default() -> Self {
        CsvProcessor {
            delimiter: ',',
            strict_mode: false,
        }
    }
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
            
            if line_content.trim().is_empty() || line_content.starts_with('#') {
                continue;
            }

            let record = self.parse_line(&line_content, line_number)?;
            records.push(record);
        }

        if self.strict_mode && records.is_empty() {
            return Err(CsvError::ValidationError(
                "No valid records found in CSV file".to_string()
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

        let id = parts[0].trim().parse::<u32>()
            .map_err(|e| CsvError::ParseError(format!(
                "Line {}: Invalid ID '{}': {}",
                line_number, parts[0], e
            )))?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Name cannot be empty",
                line_number
            )));
        }

        let value = parts[2].trim().parse::<f64>()
            .map_err(|e| CsvError::ParseError(format!(
                "Line {}: Invalid value '{}': {}",
                line_number, parts[2], e
            )))?;

        if value < 0.0 && self.strict_mode {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Value cannot be negative in strict mode",
                line_number
            )));
        }

        let active = parts[3].trim().parse::<bool>()
            .map_err(|e| CsvError::ParseError(format!(
                "Line {}: Invalid boolean '{}': {}",
                line_number, parts[3], e
            )))?;

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
        let mean = sum / count as f64;
        
        let variance: f64 = records.iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count as f64;
        
        let std_dev = variance.sqrt();
        
        let active_count = records.iter().filter(|r| r.active).count();
        
        (mean, std_dev, active_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5,true").unwrap();
        writeln!(temp_file, "2,Bob,37.8,false").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "3,Charlie,99.9,true").unwrap();

        let processor = CsvProcessor::default();
        let records = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Alice");
        assert_eq!(records[1].id, 2);
        assert!(records[2].active);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            CsvRecord { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "Test2".to_string(), value: 20.0, active: false },
            CsvRecord { id: 3, name: "Test3".to_string(), value: 30.0, active: true },
        ];

        let (mean, std_dev, active_count) = CsvProcessor::calculate_statistics(&records);
        
        assert!((mean - 20.0).abs() < 0.001);
        assert!((std_dev - 8.1649).abs() < 0.001);
        assert_eq!(active_count, 2);
    }

    #[test]
    fn test_strict_mode_validation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,,-5.0,true").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_err());
    }
}