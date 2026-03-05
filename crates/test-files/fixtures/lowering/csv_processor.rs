
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

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                return Err(CsvError::ParseError(
                    format!("Line {}: Expected 4 columns, found {}", line_num + 1, parts.len())
                ));
            }

            let id = parts[0].parse::<u32>()
                .map_err(|e| CsvError::ParseError(
                    format!("Line {}: Invalid ID format: {}", line_num + 1, e)
                ))?;

            let name = parts[1].trim().to_string();
            if name.is_empty() {
                return Err(CsvError::ValidationError(
                    format!("Line {}: Name cannot be empty", line_num + 1)
                ));
            }

            let value = parts[2].parse::<f64>()
                .map_err(|e| CsvError::ParseError(
                    format!("Line {}: Invalid value format: {}", line_num + 1, e)
                ))?;

            let active = match parts[3].trim().to_lowercase().as_str() {
                "true" | "1" | "yes" => true,
                "false" | "0" | "no" => false,
                _ => return Err(CsvError::ParseError(
                    format!("Line {}: Invalid boolean value: {}", line_num + 1, parts[3])
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_loading() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "1,ItemA,25.5,true").unwrap();
        writeln!(csv_data, "2,ItemB,30.0,false").unwrap();
        writeln!(csv_data, "3,ItemC,15.75,true").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(csv_data.path());
        
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
        assert_eq!(processor.get_active_records().len(), 2);
        assert!((processor.calculate_total_value() - 71.25).abs() < 0.001);
    }

    #[test]
    fn test_find_by_name() {
        let mut csv_data = NamedTempFile::new().unwrap();
        writeln!(csv_data, "1,TestItem,10.0,true").unwrap();

        let mut processor = CsvProcessor::new();
        processor.load_from_file(csv_data.path()).unwrap();
        
        let found = processor.find_by_name("TestItem");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 1);
    }
}