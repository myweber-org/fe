use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Record {
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
    records: Vec<Record>,
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

    fn parse_line(&self, line: &str, line_number: usize) -> Result<Record, CsvError> {
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
            return Err(CsvError::ValidationError(format!(
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
                    "Line {}: Invalid boolean format '{}'",
                    line_number, parts[3]
                )))
            }
        };

        Ok(Record {
            id,
            name,
            value,
            active,
        })
    }

    fn validate_record(&self, record: &Record, line_number: usize) -> Result<(), CsvError> {
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

        if self.records.iter().any(|r| r.id == record.id) {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Duplicate ID {}",
                line_number, record.id
            )));
        }

        Ok(())
    }

    pub fn get_records(&self) -> &[Record] {
        &self.records
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records.iter().filter(|r| r.active).collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn find_by_id(&self, id: u32) -> Option<&Record> {
        self.records.iter().find(|r| r.id == id)
    }

    pub fn transform_values<F>(&mut self, transform_fn: F)
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            record.value = transform_fn(record.value);
        }
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
        writeln!(temp_file, "1,Alice,100.5,true").unwrap();
        writeln!(temp_file, "2,Bob,200.0,false").unwrap();
        writeln!(temp_file, "3,Charlie,300.75,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
    }

    #[test]
    fn test_validation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "0,Invalid,100.5,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_active() {
        let mut processor = CsvProcessor::new();
        processor.records = vec![
            Record {
                id: 1,
                name: "Test1".to_string(),
                value: 10.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Test2".to_string(),
                value: 20.0,
                active: false,
            },
        ];

        let active = processor.filter_active();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, 1);
    }
}