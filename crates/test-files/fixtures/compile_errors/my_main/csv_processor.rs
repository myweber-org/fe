use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>], expected_columns: usize) -> Result<(), String> {
        for (index, record) in records.iter().enumerate() {
            if record.len() != expected_columns {
                return Err(format!(
                    "Record {} has {} columns, expected {}",
                    index + 1,
                    record.len(),
                    expected_columns
                ));
            }
        }
        Ok(())
    }

    pub fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Result<Vec<String>, String> {
        let mut column_data = Vec::new();
        
        for (index, record) in records.iter().enumerate() {
            if column_index >= record.len() {
                return Err(format!(
                    "Record {} doesn't have column at index {}",
                    index + 1,
                    column_index
                ));
            }
            column_data.push(record[column_index].clone());
        }
        
        Ok(column_data)
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
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor.parse_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["John", "30", "New York"]);
        assert_eq!(records[1], vec!["Alice", "25", "London"]);
    }

    #[test]
    fn test_validation() {
        let records = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];
        
        let processor = CsvProcessor::new(',', false);
        assert!(processor.validate_records(&records, 3).is_ok());
        assert!(processor.validate_records(&records, 2).is_err());
    }

    #[test]
    fn test_column_extraction() {
        let records = vec![
            vec!["John".to_string(), "30".to_string()],
            vec!["Alice".to_string(), "25".to_string()],
        ];
        
        let processor = CsvProcessor::new(',', false);
        let names = processor.extract_column(&records, 0).unwrap();
        let ages = processor.extract_column(&records, 1).unwrap();
        
        assert_eq!(names, vec!["John", "Alice"]);
        assert_eq!(ages, vec!["30", "25"]);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        Ok(())
    }
}

pub fn parse_csv(file_path: &Path) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line_num == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid number of columns at line {}", line_num + 1).into());
        }

        let id: u32 = parts[0].parse()?;
        let name = parts[1].to_string();
        let value: f64 = parts[2].parse()?;
        let active: bool = parts[3].parse()?;

        let record = Record::new(id, name, value, active);
        record.validate()?;
        records.push(record);
    }

    Ok(records)
}

pub fn calculate_total(records: &[Record]) -> f64 {
    records.iter().map(|r| r.value).sum()
}

pub fn filter_active(records: &[Record]) -> Vec<&Record> {
    records.iter().filter(|r| r.active).collect()
}