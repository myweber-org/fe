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
    valid_count: usize,
    invalid_count: usize,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
            valid_count: 0,
            invalid_count: 0,
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
            
            match self.parse_record(&line, line_num + 1) {
                Ok(record) => {
                    self.records.push(record);
                    self.valid_count += 1;
                }
                Err(e) => {
                    eprintln!("Line {}: {}", line_num + 1, e);
                    self.invalid_count += 1;
                }
            }
        }
        
        Ok(())
    }
    
    fn parse_record(&self, line: &str, line_num: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if parts.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Expected 4 columns, found {}", parts.len())
            ));
        }
        
        let id = parts[0].parse::<u32>().map_err(|_| {
            CsvError::ParseError(format!("Invalid ID format: {}", parts[0]))
        })?;
        
        let name = parts[1].to_string();
        if name.is_empty() {
            return Err(CsvError::ValidationError("Name cannot be empty".to_string()));
        }
        
        let value = parts[2].parse::<f64>().map_err(|_| {
            CsvError::ParseError(format!("Invalid value format: {}", parts[2]))
        })?;
        
        if value < 0.0 {
            return Err(CsvError::ValidationError(
                format!("Value cannot be negative: {}", value)
            ));
        }
        
        let active = match parts[3].to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(CsvError::ParseError(
                format!("Invalid boolean format: {}", parts[3])
            )),
        };
        
        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }
    
    pub fn get_stats(&self) -> (usize, usize, usize) {
        (self.records.len(), self.valid_count, self.invalid_count)
    }
    
    pub fn filter_active(&self) -> Vec<&CsvRecord> {
        self.records.iter().filter(|r| r.active).collect()
    }
    
    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }
    
    pub fn find_by_id(&self, target_id: u32) -> Option<&CsvRecord> {
        self.records.iter().find(|r| r.id == target_id)
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
        writeln!(temp_file, "1,Alice,42.5,true").unwrap();
        writeln!(temp_file, "2,Bob,-10.0,false").unwrap();
        writeln!(temp_file, "3,Charlie,100.0,yes").unwrap();
        
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.get_stats(), (2, 2, 1));
    }
    
    #[test]
    fn test_record_filtering() {
        let mut processor = CsvProcessor::new();
        processor.records = vec![
            CsvRecord { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            CsvRecord { id: 2, name: "Test2".to_string(), value: 20.0, active: false },
            CsvRecord { id: 3, name: "Test3".to_string(), value: 30.0, active: true },
        ];
        
        let active = processor.filter_active();
        assert_eq!(active.len(), 2);
        assert_eq!(active[0].id, 1);
        assert_eq!(active[1].id, 3);
    }
}