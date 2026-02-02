use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub enum CsvError {
    IoError(std::io::Error),
    ParseError(String, usize),
    InvalidHeader(String),
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsvError::IoError(e) => write!(f, "IO error: {}", e),
            CsvError::ParseError(msg, line) => write!(f, "Parse error at line {}: {}", line, msg),
            CsvError::InvalidHeader(msg) => write!(f, "Invalid header: {}", msg),
        }
    }
}

impl Error for CsvError {}

impl From<std::io::Error> for CsvError {
    fn from(error: std::io::Error) -> Self {
        CsvError::IoError(error)
    }
}

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file(&self, file_path: &str) -> Result<Vec<Vec<String>>, CsvError> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line?;
            
            if line_content.trim().is_empty() {
                continue;
            }

            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.is_empty() {
                return Err(CsvError::ParseError(
                    "Empty line with content".to_string(),
                    line_number,
                ));
            }

            if self.has_header && line_number == 1 {
                if fields.iter().any(|f| f.is_empty()) {
                    return Err(CsvError::InvalidHeader(
                        "Header contains empty fields".to_string(),
                    ));
                }
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err(CsvError::ParseError(
                "File contains no valid data".to_string(),
                0,
            ));
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), CsvError> {
        if records.is_empty() {
            return Ok(());
        }

        let expected_len = records[0].len();
        
        for (idx, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(CsvError::ParseError(
                    format!("Record has {} fields, expected {}", record.len(), expected_len),
                    idx + 1,
                ));
            }
            
            if record.iter().any(|field| field.is_empty()) {
                return Err(CsvError::ParseError(
                    "Record contains empty fields".to_string(),
                    idx + 1,
                ));
            }
        }
        
        Ok(())
    }
}

pub fn extract_column(records: &[Vec<String>], column_index: usize) -> Result<Vec<String>, CsvError> {
    if records.is_empty() {
        return Err(CsvError::ParseError(
            "No records to process".to_string(),
            0,
        ));
    }

    if column_index >= records[0].len() {
        return Err(CsvError::ParseError(
            format!("Column index {} out of bounds", column_index),
            0,
        ));
    }

    let mut column_data = Vec::with_capacity(records.len());
    
    for (line_num, record) in records.iter().enumerate() {
        if column_index >= record.len() {
            return Err(CsvError::ParseError(
                format!("Column index {} out of bounds for record", column_index),
                line_num + 1,
            ));
        }
        column_data.push(record[column_index].clone());
    }
    
    Ok(column_data)
}
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
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            
            let fields: Vec<&str> = line.split(',').collect();
            
            if fields.len() != 4 {
                return Err(CsvError::ParseError(
                    format!("Line {}: expected 4 fields, found {}", line_num + 1, fields.len())
                ));
            }
            
            let id = fields[0].parse::<u32>()
                .map_err(|e| CsvError::ParseError(
                    format!("Line {}: invalid id '{}': {}", line_num + 1, fields[0], e)
                ))?;
            
            let name = fields[1].trim().to_string();
            if name.is_empty() {
                return Err(CsvError::ValidationError(
                    format!("Line {}: name cannot be empty", line_num + 1)
                ));
            }
            
            let value = fields[2].parse::<f64>()
                .map_err(|e| CsvError::ParseError(
                    format!("Line {}: invalid value '{}': {}", line_num + 1, fields[2], e)
                ))?;
            
            let active = match fields[3].trim().to_lowercase().as_str() {
                "true" | "1" | "yes" => true,
                "false" | "0" | "no" => false,
                _ => return Err(CsvError::ParseError(
                    format!("Line {}: invalid boolean value '{}'", line_num + 1, fields[3])
                )),
            };
            
            self.records.push(CsvRecord {
                id,
                name,
                value,
                active,
            });
        }
        
        Ok(())
    }

    pub fn get_active_records(&self) -> Vec<&CsvRecord> {
        self.records.iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter()
            .map(|record| record.value)
            .sum()
    }

    pub fn find_by_name(&self, name: &str) -> Option<&CsvRecord> {
        self.records.iter()
            .find(|record| record.name == name)
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

impl Default for CsvProcessor {
    fn default() -> Self {
        Self::new()
    }
}