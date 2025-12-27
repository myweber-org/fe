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

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
    valid_count: usize,
    invalid_count: usize,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
            valid_count: 0,
            invalid_count: 0,
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), CsvError> {
        let file = File::open(&path).map_err(|e| {
            CsvError::IoError(format!("Failed to open file {}: {}", path.as_ref().display(), e))
        })?;

        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| {
                CsvError::IoError(format!("Failed to read line {}: {}", line_num + 1, e))
            })?;

            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            match self.parse_line(&line, line_num + 1) {
                Ok(record) => {
                    self.records.push(record);
                    self.valid_count += 1;
                }
                Err(e) => {
                    eprintln!("Line {}: {}", line_num + 1, e);
                    self.invalid_count += 1;
                }
            }
        }

        Ok(())
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(format!(
                "Expected 4 fields, found {} at line {}",
                parts.len(),
                line_num
            )));
        }

        let id = parts[0].parse::<u32>().map_err(|_| {
            CsvError::ParseError(format!("Invalid ID format: '{}' at line {}", parts[0], line_num))
        })?;

        let name = parts[1].to_string();
        if name.is_empty() {
            return Err(CsvError::ValidationError(format!(
                "Name cannot be empty at line {}",
                line_num
            )));
        }

        let value = parts[2].parse::<f64>().map_err(|_| {
            CsvError::ParseError(format!("Invalid value format: '{}' at line {}", parts[2], line_num))
        })?;

        let active = match parts[3].to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(CsvError::ParseError(format!(
                "Invalid boolean format: '{}' at line {}",
                parts[3],
                line_num
            ))),
        };

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter()
            .filter(|r| r.active)
            .map(|r| r.value)
            .sum()
    }

    pub fn find_by_name(&self, name: &str) -> Option<&CsvRecord> {
        self.records.iter()
            .find(|r| r.name.to_lowercase() == name.to_lowercase())
    }

    pub fn statistics(&self) -> (usize, usize, f64) {
        let total = self.calculate_total();
        (self.valid_count, self.invalid_count, total)
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_parsing() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "1,Alice,100.5,true").unwrap();
        writeln!(csv_data, "2,Bob,200.0,false").unwrap();
        writeln!(csv_data, "3,Charlie,300.75,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        assert!(result.is_ok());
        assert_eq!(processor.valid_count, 3);
        assert_eq!(processor.invalid_count, 0);
        assert_eq!(processor.calculate_total(), 401.25);
    }

    #[test]
    fn test_invalid_csv() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "1,Alice,100.5,true").unwrap();
        writeln!(csv_data, "invalid,bad,data").unwrap();
        writeln!(csv_data, "3,Charlie,300.75,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        assert!(result.is_ok());
        assert_eq!(processor.valid_count, 2);
        assert_eq!(processor.invalid_count, 1);
    }
}