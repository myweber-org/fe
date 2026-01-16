
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
    IoError(String),
    ParseError(String),
    ValidationError(String),
}

impl From<std::io::Error> for CsvError {
    fn from(error: std::io::Error) -> Self {
        CsvError::IoError(error.to_string())
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

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() {
                if self.strict_mode {
                    return Err(CsvError::ValidationError(
                        format!("Empty line at line {}", line_num + 1)
                    ));
                }
                continue;
            }

            let record = self.parse_line(&line, line_num + 1)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Expected 4 columns, found {} at line {}", parts.len(), line_num)
            ));
        }

        let id = parts[0].parse::<u32>()
            .map_err(|e| CsvError::ParseError(
                format!("Invalid ID at line {}: {}", line_num, e)
            ))?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(CsvError::ValidationError(
                format!("Empty name at line {}", line_num)
            ));
        }

        let value = parts[2].parse::<f64>()
            .map_err(|e| CsvError::ParseError(
                format!("Invalid value at line {}: {}", line_num, e)
            ))?;

        let active = parts[3].parse::<bool>()
            .map_err(|e| CsvError::ParseError(
                format!("Invalid boolean at line {}: {}", line_num, e)
            ))?;

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

        let max_value = records.iter()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);

        (average, max_value, count)
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
        writeln!(temp_file, "1,Alice,100.5,true").unwrap();
        writeln!(temp_file, "2,Bob,200.75,false").unwrap();
        writeln!(temp_file, "3,Charlie,150.25,true").unwrap();

        let processor = CsvProcessor::default();
        let records = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Alice");
        assert_eq!(records[1].value, 200.75);
        assert!(!records[1].active);

        let (avg, max, count) = CsvProcessor::calculate_statistics(&records);
        assert_eq!(count, 3);
        assert!((avg - 150.5).abs() < 0.001);
        assert!((max - 200.75).abs() < 0.001);
    }

    #[test]
    fn test_invalid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,invalid,true").unwrap();

        let processor = CsvProcessor::default();
        let result = processor.process_file(temp_file.path());
        assert!(result.is_err());
    }
}