
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
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| {
                CsvError::IoError(format!("Failed to read line {}: {}", line_num + 1, e))
            })?;

            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let record = self.parse_line(&line, line_num + 1)?;
            self.validate_record(&record, line_num + 1)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Line {}: Expected 4 fields, found {}", line_num, parts.len())
            ));
        }

        let id = parts[0].parse::<u32>().map_err(|e| {
            CsvError::ParseError(format!("Line {}: Invalid ID format: {}", line_num, e))
        })?;

        let name = parts[1].to_string();
        
        let value = parts[2].parse::<f64>().map_err(|e| {
            CsvError::ParseError(format!("Line {}: Invalid value format: {}", line_num, e))
        })?;

        let active = parts[3].parse::<bool>().map_err(|e| {
            CsvError::ParseError(format!("Line {}: Invalid boolean format: {}", line_num, e))
        })?;

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    fn validate_record(&self, record: &CsvRecord, line_num: usize) -> Result<(), CsvError> {
        if record.id == 0 {
            return Err(CsvError::ValidationError(
                format!("Line {}: ID cannot be zero", line_num)
            ));
        }

        if record.name.is_empty() {
            return Err(CsvError::ValidationError(
                format!("Line {}: Name cannot be empty", line_num)
            ));
        }

        if record.value < 0.0 {
            return Err(CsvError::ValidationError(
                format!("Line {}: Value cannot be negative", line_num)
            ));
        }

        Ok(())
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }

    pub fn filter_active(&self) -> Vec<&CsvRecord> {
        self.records.iter()
            .filter(|r| r.active)
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter()
            .map(|r| r.value)
            .sum()
    }

    pub fn find_by_id(&self, id: u32) -> Option<&CsvRecord> {
        self.records.iter()
            .find(|r| r.id == id)
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
        writeln!(csv_data, "1,Item A,10.5,true").unwrap();
        writeln!(csv_data, "2,Item B,20.0,false").unwrap();
        writeln!(csv_data, "# This is a comment").unwrap();
        writeln!(csv_data, "3,Item C,15.75,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
        assert_eq!(processor.calculate_total(), 46.25);
        assert_eq!(processor.filter_active().len(), 2);
    }

    #[test]
    fn test_invalid_data() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "invalid,Item A,10.5,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        
        assert!(result.is_err());
    }
}