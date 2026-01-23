use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub fields: Vec<String>,
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
    expected_columns: usize,
}

impl CsvProcessor {
    pub fn new(delimiter: char, expected_columns: usize) -> Self {
        CsvProcessor {
            delimiter,
            expected_columns,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let record = self.parse_line(&line, line_num + 1)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, CsvError> {
        let fields: Vec<String> = line
            .split(self.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        if fields.len() != self.expected_columns {
            return Err(CsvError::ValidationError(format!(
                "Line {}: expected {} columns, found {}",
                line_num,
                self.expected_columns,
                fields.len()
            )));
        }

        for (idx, field) in fields.iter().enumerate() {
            if field.is_empty() {
                return Err(CsvError::ParseError(format!(
                    "Line {}: empty field at column {}",
                    line_num,
                    idx + 1
                )));
            }
        }

        Ok(CsvRecord { fields })
    }

    pub fn validate_records(&self, records: &[CsvRecord]) -> Result<(), CsvError> {
        for (idx, record) in records.iter().enumerate() {
            if record.fields.len() != self.expected_columns {
                return Err(CsvError::ValidationError(format!(
                    "Record {} has {} columns, expected {}",
                    idx + 1,
                    record.fields.len(),
                    self.expected_columns
                )));
            }
        }
        Ok(())
    }
}

pub fn calculate_column_stats(records: &[CsvRecord], column_index: usize) -> Option<(f64, f64)> {
    if records.is_empty() {
        return None;
    }

    let values: Vec<f64> = records
        .iter()
        .filter_map(|record| record.fields.get(column_index))
        .filter_map(|field| field.parse::<f64>().ok())
        .collect();

    if values.is_empty() {
        return None;
    }

    let sum: f64 = values.iter().sum();
    let count = values.len() as f64;
    let mean = sum / count;

    let variance: f64 = values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / count;

    Some((mean, variance.sqrt()))
}