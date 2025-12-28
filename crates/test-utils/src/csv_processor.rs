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

        let fields: Vec<&str> = line_content.split(',').collect();
        if fields.len() != 4 {
            return Err(CsvError::ParseError(format!(
                "Line {}: Expected 4 fields, found {}",
                line_number,
                fields.len()
            )));
        }

        let id = fields[0]
            .parse::<u32>()
            .map_err(|_| CsvError::ParseError(format!("Line {}: Invalid ID format", line_number)))?;

        let name = fields[1].trim().to_string();
        if name.is_empty() {
            return Err(CsvError::ValidationError(format!(
                "Line {}: Name cannot be empty",
                line_number
            )));
        }

        let value = fields[2]
            .parse::<f64>()
            .map_err(|_| CsvError::ParseError(format!("Line {}: Invalid value format", line_number)))?;

        let active = match fields[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => {
                return Err(CsvError::ParseError(format!(
                    "Line {}: Invalid boolean value",
                    line_number
                )))
            }
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
    let active_records: Vec<&CsvRecord> = records.iter().filter(|r| r.active).collect();
    
    if active_records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = active_records.iter().map(|r| r.value).sum();
    let count = active_records.len() as f64;
    let mean = sum / count;

    let variance: f64 = active_records
        .iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>()
        / count;

    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
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
        writeln!(temp_file, "3,Charlie,29.3,true").unwrap();

        let result = process_csv_file(temp_file.path());
        assert!(result.is_ok());
        
        let records = result.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "Alice");
        assert_eq!(records[1].value, 37.8);
        assert!(!records[1].active);
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            CsvRecord {
                id: 1,
                name: "Test1".to_string(),
                value: 10.0,
                active: true,
            },
            CsvRecord {
                id: 2,
                name: "Test2".to_string(),
                value: 20.0,
                active: true,
            },
            CsvRecord {
                id: 3,
                name: "Test3".to_string(),
                value: 30.0,
                active: false,
            },
        ];

        let (mean, variance, std_dev) = calculate_statistics(&records);
        assert_eq!(mean, 15.0);
        assert_eq!(variance, 25.0);
        assert_eq!(std_dev, 5.0);
    }

    #[test]
    fn test_empty_name_validation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,,42.5,true").unwrap();

        let result = process_csv_file(temp_file.path());
        assert!(matches!(result, Err(CsvError::ValidationError(_))));
    }
}