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

pub fn process_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<CsvRecord>, CsvError> {
    let file = File::open(&path).map_err(|e| {
        CsvError::IoError(format!("Failed to open file {}: {}", path.as_ref().display(), e))
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

        let record = parse_csv_line(&line_content, line_number)?;
        validate_record(&record, line_number)?;
        records.push(record);
    }

    if records.is_empty() {
        return Err(CsvError::ValidationError(
            "CSV file contains no valid records".to_string(),
        ));
    }

    Ok(records)
}

fn parse_csv_line(line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

    if parts.len() != 4 {
        return Err(CsvError::ParseError(format!(
            "Line {}: Expected 4 fields, found {}",
            line_number,
            parts.len()
        )));
    }

    let id = parts[0]
        .parse::<u32>()
        .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid ID '{}': {}", line_number, parts[0], e)))?;

    let name = parts[1].to_string();
    if name.is_empty() {
        return Err(CsvError::ValidationError(format!(
            "Line {}: Name cannot be empty",
            line_number
        )));
    }

    let value = parts[2]
        .parse::<f64>()
        .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid value '{}': {}", line_number, parts[2], e)))?;

    let active = parts[3]
        .parse::<bool>()
        .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid active flag '{}': {}", line_number, parts[3], e)))?;

    Ok(CsvRecord {
        id,
        name,
        value,
        active,
    })
}

fn validate_record(record: &CsvRecord, line_number: usize) -> Result<(), CsvError> {
    if record.id == 0 {
        return Err(CsvError::ValidationError(format!(
            "Line {}: ID must be greater than 0",
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

pub fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, usize) {
    if records.is_empty() {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = sum / count as f64;

    let active_count = records.iter().filter(|r| r.active).count();

    (sum, average, active_count)
}