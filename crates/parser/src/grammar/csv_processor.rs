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
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: usize,
    filter_value: String,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str, filter_column: usize, filter_value: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            filter_column,
            filter_value: filter_value.to_string(),
        }
    }

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        
        let mut processed_count = 0;
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }
            
            let columns: Vec<&str> = line.split(',').collect();
            
            if columns.len() > self.filter_column {
                if columns[self.filter_column] == self.filter_value {
                    writeln!(output_file, "{}", line)?;
                    processed_count += 1;
                }
            }
        }
        
        Ok(processed_count)
    }
    
    pub fn transform_column(&self, transform_column: usize, transform_fn: fn(&str) -> String) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        
        let mut transformed_count = 0;
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }
            
            let mut columns: Vec<&str> = line.split(',').collect();
            
            if columns.len() > transform_column {
                let original_value = columns[transform_column];
                let transformed_value = transform_fn(original_value);
                columns[transform_column] = &transformed_value;
                
                let new_line = columns.join(",");
                writeln!(output_file, "{}", new_line)?;
                transformed_count += 1;
            }
        }
        
        Ok(transformed_count)
    }
}

fn uppercase_transform(value: &str) -> String {
    value.to_uppercase()
}

pub fn validate_csv_path(path: &str) -> Result<(), String> {
    let path_obj = Path::new(path);
    
    if !path_obj.exists() {
        return Err(format!("File does not exist: {}", path));
    }
    
    if !path_obj.is_file() {
        return Err(format!("Path is not a file: {}", path));
    }
    
    if let Some(extension) = path_obj.extension() {
        if extension != "csv" {
            return Err(format!("File extension must be .csv, found: {:?}", extension));
        }
    } else {
        return Err("File must have an extension".to_string());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_csv_processing() {
        let csv_content = "id,name,status\n1,Alice,active\n2,Bob,inactive\n3,Charlie,active\n";
        
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), csv_content).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let processor = CsvProcessor::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            2,
            "active"
        );
        
        let result = processor.process();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        
        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let expected = "id,name,status\n1,Alice,active\n3,Charlie,active\n";
        assert_eq!(output_content, expected);
    }
    
    #[test]
    fn test_column_transformation() {
        let csv_content = "id,name,status\n1,alice,active\n2,bob,inactive\n";
        
        let input_file = NamedTempFile::new().unwrap();
        fs::write(input_file.path(), csv_content).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let processor = CsvProcessor::new(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            0,
            ""
        );
        
        let result = processor.transform_column(1, uppercase_transform);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        
        let output_content = fs::read_to_string(output_file.path()).unwrap();
        let expected = "id,name,status\n1,ALICE,active\n2,BOB,inactive\n";
        assert_eq!(output_content, expected);
    }
}