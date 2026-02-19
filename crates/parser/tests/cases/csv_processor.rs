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

pub fn process_csv_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<CsvRecord>, CsvError> {
    let file = File::open(&file_path).map_err(|e| {
        CsvError::IoError(format!("Failed to open file {}: {}", file_path.as_ref().display(), e))
    })?;

    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line_content = line.map_err(|e| {
            CsvError::IoError(format!("Failed to read line {}: {}", line_number, e))
        })?;

        if line_content.trim().is_empty() || line_content.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line_content.split(',').map(|s| s.trim()).collect();
        
        if fields.len() != 4 {
            return Err(CsvError::ParseError(format!(
                "Line {}: expected 4 fields, found {}",
                line_number,
                fields.len()
            )));
        }

        let id = fields[0].parse::<u32>().map_err(|_| {
            CsvError::ParseError(format!("Line {}: invalid ID format '{}'", line_number, fields[0]))
        })?;

        let name = fields[1].to_string();
        if name.is_empty() {
            return Err(CsvError::ValidationError(format!(
                "Line {}: name cannot be empty",
                line_number
            )));
        }

        let value = fields[2].parse::<f64>().map_err(|_| {
            CsvError::ParseError(format!("Line {}: invalid value format '{}'", line_number, fields[2]))
        })?;

        let active = match fields[3].to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(CsvError::ParseError(format!(
                "Line {}: invalid boolean value '{}'",
                line_number, fields[3]
            ))),
        };

        records.push(CsvRecord {
            id,
            name,
            value,
            active,
        });
    }

    if records.is_empty() {
        return Err(CsvError::ValidationError("No valid records found in file".to_string()));
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = if count > 0.0 { sum / count } else { 0.0 };
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5,true").unwrap();
        writeln!(temp_file, "2,Bob,37.8,false").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "3,Charlie,29.1,yes").unwrap();

        let result = process_csv_file(temp_file.path());
        assert!(result.is_ok());
        
        let records = result.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Alice");
        assert_eq!(records[1].active, false);
        assert_eq!(records[2].id, 3);
    }

    #[test]
    fn test_invalid_csv_format() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Alice,42.5").unwrap();

        let result = process_csv_file(temp_file.path());
        assert!(result.is_err());
        
        if let Err(CsvError::ParseError(msg)) = result {
            assert!(msg.contains("expected 4 fields"));
        } else {
            panic!("Expected ParseError");
        }
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            CsvRecord { id: 1, name: "A".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "B".to_string(), value: 20.0, active: false },
            CsvRecord { id: 3, name: "C".to_string(), value: 30.0, active: true },
        ];
        
        let (sum, mean, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!((std_dev - 8.164965).abs() < 0.0001);
    }
}use std::error::Error;
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
            CsvError::IoError(format!("Failed to open file {}: {}", path.as_ref().display(), e))
        })?;

        let reader = BufReader::new(file);
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line.map_err(|e| {
                CsvError::IoError(format!("Failed to read line {}: {}", line_number, e))
            })?;

            if line_content.trim().is_empty() || line_content.starts_with('#') {
                continue;
            }

            let record = self.parse_line(&line_content, line_number)?;
            self.validate_record(&record, line_number)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

        if parts.len() != 4 {
            return Err(CsvError::ParseError(format!(
                "Line {}: Expected 4 fields, found {}",
                line_number,
                parts.len()
            )));
        }

        let id = parts[0].parse::<u32>().map_err(|_| {
            CsvError::ParseError(format!("Line {}: Invalid ID format '{}'", line_number, parts[0]))
        })?;

        let name = parts[1].to_string();
        if name.is_empty() {
            return Err(CsvError::ParseError(format!(
                "Line {}: Name cannot be empty",
                line_number
            )));
        }

        let value = parts[2].parse::<f64>().map_err(|_| {
            CsvError::ParseError(format!(
                "Line {}: Invalid value format '{}'",
                line_number, parts[2]
            ))
        })?;

        let active = match parts[3].to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => {
                return Err(CsvError::ParseError(format!(
                    "Line {}: Invalid boolean value '{}'",
                    line_number, parts[3]
                )))
            }
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
            return Err(CsvError::ValidationError(format!(
                "Line {}: ID cannot be zero",
                line_number
            )));
        }

        if record.value < 0.0 {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Value cannot be negative",
                line_number
            )));
        }

        if record.name.len() > 100 {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Name exceeds maximum length of 100 characters",
                line_number
            )));
        }

        Ok(())
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn filter_active(&self) -> Vec<&CsvRecord> {
        self.records.iter().filter(|r| r.active).collect()
    }

    pub fn find_by_id(&self, id: u32) -> Option<&CsvRecord> {
        self.records.iter().find(|r| r.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv_parsing() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "1,John Doe,100.5,true").unwrap();
        writeln!(csv_data, "2,Jane Smith,200.75,false").unwrap();
        writeln!(csv_data, "# This is a comment").unwrap();
        writeln!(csv_data, "").unwrap();
        writeln!(csv_data, "3,Bob Johnson,300.0,yes").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.calculate_total(), 601.25);
    }

    #[test]
    fn test_invalid_csv_parsing() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "invalid,John Doe,100.5,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_errors() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "0,John Doe,100.5,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        assert!(result.is_err());
    }
}