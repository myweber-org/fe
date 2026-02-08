
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl CsvRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if self.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        Ok(())
    }
}

pub fn process_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_number = 0;

    for line in reader.lines() {
        line_number += 1;
        let line = line?;
        
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid CSV format at line {}", line_number).into());
        }

        let id = parts[0].parse::<u32>()
            .map_err(|_| format!("Invalid ID at line {}", line_number))?;
        
        let name = parts[1].trim().to_string();
        let value = parts[2].parse::<f64>()
            .map_err(|_| format!("Invalid value at line {}", line_number))?;
        let category = parts[3].trim().to_string();

        let record = CsvRecord::new(id, name, value, category);
        record.validate()
            .map_err(|e| format!("Validation error at line {}: {}", line_number, e))?;
        
        records.push(record);
    }

    Ok(records)
}

pub fn filter_by_category(records: &[CsvRecord], category: &str) -> Vec<CsvRecord> {
    records.iter()
        .filter(|r| r.category == category)
        .cloned()
        .collect()
}

pub fn calculate_total_value(records: &[CsvRecord]) -> f64 {
    records.iter().map(|r| r.value).sum()
}

pub fn find_max_value_record(records: &[CsvRecord]) -> Option<&CsvRecord> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,Item A,25.5,Electronics").unwrap();
        writeln!(temp_file, "2,Item B,15.0,Books").unwrap();
        writeln!(temp_file, "3,Item C,42.8,Electronics").unwrap();
        
        let records = process_csv_file(temp_file.path()).unwrap();
        assert_eq!(records.len(), 3);
        
        let electronics = filter_by_category(&records, "Electronics");
        assert_eq!(electronics.len(), 2);
        
        let total = calculate_total_value(&records);
        assert!((total - 83.3).abs() < 0.001);
        
        let max_record = find_max_value_record(&records).unwrap();
        assert_eq!(max_record.id, 3);
    }

    #[test]
    fn test_record_validation() {
        let valid_record = CsvRecord::new(1, "Test".to_string(), 10.0, "Category".to_string());
        assert!(valid_record.validate().is_ok());
        
        let invalid_record = CsvRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(invalid_record.validate().is_err());
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    delimiter: char,
    has_headers: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_headers: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_headers,
        }
    }

    pub fn validate_file(&self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        let mut line_count = 0;
        let mut column_count: Option<usize> = None;
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            if index == 0 && self.has_headers {
                column_count = Some(columns.len());
                continue;
            }
            
            match column_count {
                Some(expected) if columns.len() != expected => {
                    return Err(format!(
                        "Line {} has {} columns, expected {}",
                        index + 1,
                        columns.len(),
                        expected
                    ).into());
                }
                None => column_count = Some(columns.len()),
                _ => {}
            }
            
            line_count += 1;
        }
        
        Ok(line_count)
    }

    pub fn transform_column<F>(&self, file_path: &str, column_index: usize, transform_fn: F) -> Result<Vec<String>, Box<dyn Error>>
    where
        F: Fn(&str) -> String,
    {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        let mut results = Vec::new();
        let mut skip_first = self.has_headers;
        
        for line in reader.lines() {
            let line = line?;
            
            if skip_first {
                skip_first = false;
                continue;
            }
            
            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            if column_index < columns.len() {
                let transformed = transform_fn(columns[column_index]);
                results.push(transformed);
            } else {
                return Err(format!("Column index {} out of bounds", column_index).into());
            }
        }
        
        Ok(results)
    }

    pub fn filter_rows<P>(&self, file_path: &str, predicate: P) -> Result<Vec<String>, Box<dyn Error>>
    where
        P: Fn(&[&str]) -> bool,
    {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        let mut filtered_rows = Vec::new();
        let mut skip_first = self.has_headers;
        
        for line in reader.lines() {
            let line = line?;
            
            if skip_first {
                skip_first = false;
                filtered_rows.push(line);
                continue;
            }
            
            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            if predicate(&columns) {
                filtered_rows.push(line);
            }
        }
        
        Ok(filtered_rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,city").unwrap();
        writeln!(file, "Alice,30,New York").unwrap();
        writeln!(file, "Bob,25,London").unwrap();
        writeln!(file, "Charlie,35,Paris").unwrap();
        file
    }

    #[test]
    fn test_validate_file() {
        let file = create_test_csv();
        let processor = CsvProcessor::new(',', true);
        
        let result = processor.validate_file(file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
    }

    #[test]
    fn test_transform_column() {
        let file = create_test_csv();
        let processor = CsvProcessor::new(',', true);
        
        let result = processor.transform_column(file.path().to_str().unwrap(), 1, |age| {
            format!("Age: {}", age)
        });
        
        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert_eq!(transformed, vec!["Age: 30", "Age: 25", "Age: 35"]);
    }

    #[test]
    fn test_filter_rows() {
        let file = create_test_csv();
        let processor = CsvProcessor::new(',', true);
        
        let result = processor.filter_rows(file.path().to_str().unwrap(), |columns| {
            columns[1].parse::<i32>().unwrap_or(0) > 30
        });
        
        assert!(result.is_ok());
        let filtered = result.unwrap();
        assert_eq!(filtered.len(), 2);
        assert!(filtered[0].contains("name,age,city"));
        assert!(filtered[1].contains("Charlie,35,Paris"));
    }
}