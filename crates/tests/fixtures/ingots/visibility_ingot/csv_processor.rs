
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
            CsvError::IoError(msg) => write!(f, "IO Error: {}", msg),
            CsvError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            CsvError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
        }
    }
}

impl Error for CsvError {}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), CsvError> {
        let file = File::open(&path).map_err(|e| {
            CsvError::IoError(format!("Failed to open file: {}", e))
        })?;

        let reader = BufReader::new(file);
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line = line.map_err(|e| {
                CsvError::IoError(format!("Failed to read line {}: {}", line_number, e))
            })?;

            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let record = self.parse_line(&line, line_number)?;
            self.validate_record(&record, line_number)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Line {}: Expected 4 fields, found {}", line_number, parts.len())
            ));
        }

        let id = parts[0].parse::<u32>().map_err(|_| {
            CsvError::ParseError(format!("Line {}: Invalid ID format '{}'", line_number, parts[0]))
        })?;

        let name = parts[1].to_string();
        
        let value = parts[2].parse::<f64>().map_err(|_| {
            CsvError::ParseError(format!("Line {}: Invalid value format '{}'", line_number, parts[2]))
        })?;

        let active = match parts[3].to_lowercase().as_str() {
            "true" => true,
            "false" => false,
            _ => return Err(CsvError::ParseError(
                format!("Line {}: Invalid boolean format '{}'", line_number, parts[3])
            )),
        };

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    fn validate_record(&self, record: &CsvRecord, line_number: usize) -> Result<(), CsvError> {
        if record.id == 0 {
            return Err(CsvError::ValidationError(
                format!("Line {}: ID cannot be zero", line_number)
            ));
        }

        if record.name.is_empty() {
            return Err(CsvError::ValidationError(
                format!("Line {}: Name cannot be empty", line_number)
            ));
        }

        if record.value < 0.0 {
            return Err(CsvError::ValidationError(
                format!("Line {}: Value cannot be negative", line_number)
            ));
        }

        Ok(())
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter()
            .filter(|r| r.active)
            .map(|r| r.value)
            .sum()
    }

    pub fn find_by_id(&self, id: u32) -> Option<&CsvRecord> {
        self.records.iter().find(|r| r.id == id)
    }

    pub fn filter_by_active(&self) -> Vec<&CsvRecord> {
        self.records.iter()
            .filter(|r| r.active)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_parsing() {
        let mut processor = CsvProcessor::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "1,ItemA,10.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,20.0,false").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "3,ItemC,15.75,true").unwrap();

        let result = processor.load_from_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.calculate_total(), 26.25);
    }

    #[test]
    fn test_invalid_format() {
        let mut processor = CsvProcessor::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "1,ItemA,10.5").unwrap(); // Missing field

        let result = processor.load_from_file(temp_file.path());
        assert!(result.is_err());
    }
}