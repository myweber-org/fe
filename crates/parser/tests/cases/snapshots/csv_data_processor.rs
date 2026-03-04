use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    data: Vec<Vec<String>>,
    headers: Vec<String>,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let headers = if let Some(first_line) = lines.next() {
            first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            return Err("Empty CSV file".into());
        };
        
        let mut data = Vec::new();
        for line in lines {
            let line = line?;
            let row: Vec<String> = line
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if row.len() == headers.len() {
                data.push(row);
            }
        }
        
        Ok(CsvProcessor { data, headers })
    }
    
    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        if let Some(col_index) = self.headers.iter().position(|h| h == column_name) {
            self.data
                .iter()
                .filter(|row| row.get(col_index).map_or(false, |v| v == value))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn aggregate_numeric_column(&self, column_name: &str) -> Option<f64> {
        if let Some(col_index) = self.headers.iter().position(|h| h == column_name) {
            let sum: f64 = self.data
                .iter()
                .filter_map(|row| row.get(col_index).and_then(|v| v.parse::<f64>().ok()))
                .sum();
            
            if !self.data.is_empty() {
                Some(sum / self.data.len() as f64)
            } else {
                Some(0.0)
            }
        } else {
            None
        }
    }
    
    pub fn get_unique_values(&self, column_name: &str) -> Vec<String> {
        if let Some(col_index) = self.headers.iter().position(|h| h == column_name) {
            let mut values: Vec<String> = self.data
                .iter()
                .filter_map(|row| row.get(col_index).cloned())
                .collect();
            
            values.sort();
            values.dedup();
            values
        } else {
            Vec::new()
        }
    }
    
    pub fn row_count(&self) -> usize {
        self.data.len()
    }
    
    pub fn column_count(&self) -> usize {
        self.headers.len()
    }
}
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

#[derive(Debug, Serialize)]
struct ProcessedRecord {
    id: u32,
    normalized_value: f64,
    category_code: String,
}

fn validate_record(record: &DataRecord) -> Result<(), String> {
    if record.name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if record.value < 0.0 {
        return Err("Value must be non-negative".to_string());
    }
    if record.category.len() > 10 {
        return Err("Category exceeds maximum length".to_string());
    }
    Ok(())
}

fn transform_record(record: DataRecord) -> ProcessedRecord {
    let normalized_value = if record.value > 100.0 {
        record.value / 10.0
    } else {
        record.value
    };
    
    let category_code = match record.category.as_str() {
        "A" | "B" | "C" => format!("CAT_{}", record.category),
        _ => "CAT_OTHER".to_string(),
    };
    
    ProcessedRecord {
        id: record.id,
        normalized_value,
        category_code,
    }
}

pub fn process_csv_file(input_path: &Path, output_path: &Path) -> Result<usize, Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;
    
    let mut processed_count = 0;
    let mut error_count = 0;
    
    for result in reader.deserialize() {
        let record: DataRecord = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to deserialize record: {}", e);
                error_count += 1;
                continue;
            }
        };
        
        if let Err(e) = validate_record(&record) {
            eprintln!("Validation failed for record {}: {}", record.id, e);
            error_count += 1;
            continue;
        }
        
        let processed_record = transform_record(record);
        
        writer.serialize(&processed_record)?;
        processed_count += 1;
    }
    
    writer.flush()?;
    
    println!("Processing complete: {} records processed, {} errors", processed_count, error_count);
    Ok(processed_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_validate_record_valid() {
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 50.0,
            category: "A".to_string(),
        };
        assert!(validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_validate_record_invalid_name() {
        let record = DataRecord {
            id: 1,
            name: "".to_string(),
            value: 50.0,
            category: "A".to_string(),
        };
        assert!(validate_record(&record).is_err());
    }
    
    #[test]
    fn test_transform_record() {
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 150.0,
            category: "A".to_string(),
        };
        
        let processed = transform_record(record);
        assert_eq!(processed.id, 1);
        assert_eq!(processed.normalized_value, 15.0);
        assert_eq!(processed.category_code, "CAT_A");
    }
}