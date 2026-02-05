use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataProcessor {
    file_path: String,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn process(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if fields.len() < 2 {
                return Err(format!("Invalid data at line {}", line_num + 1).into());
            }
            
            records.push(fields);
        }

        if records.is_empty() {
            return Err("No data found in file".into());
        }

        Ok(records)
    }

    pub fn validate_numeric_fields(&self, records: &[Vec<String>], field_index: usize) -> Result<Vec<f64>, Box<dyn Error>> {
        let mut numeric_values = Vec::new();
        
        for (line_num, record) in records.iter().enumerate() {
            if field_index >= record.len() {
                return Err(format!("Field index out of bounds at line {}", line_num + 1).into());
            }
            
            match record[field_index].parse::<f64>() {
                Ok(value) => numeric_values.push(value),
                Err(_) => return Err(format!("Non-numeric value at line {}: {}", line_num + 1, record[field_index]).into()),
            }
        }
        
        Ok(numeric_values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();
        writeln!(temp_file, "Bob,30,Paris").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process();
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0][1], "25");
    }

    #[test]
    fn test_numeric_validation() {
        let records = vec![
            vec!["test".to_string(), "42.5".to_string()],
            vec!["test".to_string(), "100".to_string()],
        ];
        
        let processor = DataProcessor::new("dummy.csv");
        let result = processor.validate_numeric_fields(&records, 1);
        
        assert!(result.is_ok());
        let values = result.unwrap();
        assert_eq!(values, vec![42.5, 100.0]);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(id) => id,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(value) => value,
                Err(_) => continue,
            };

            let category = parts[2].trim().to_string();
            let valid = value >= 0.0 && value <= 1000.0;

            self.records.push(DataRecord {
                id,
                value,
                category,
                valid,
            });

            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.valid)
            .cloned()
            .collect()
    }

    pub fn average_value(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.records.iter().filter(|r| r.valid).collect();
        
        if valid_records.is_empty() {
            return None;
        }

        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn count_by_category(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        
        for record in &self.records {
            if record.valid {
                *counts.entry(record.category.clone()).or_insert(0) += 1;
            }
        }
        
        counts
    }

    pub fn total_records(&self) -> usize {
        self.records.len()
    }

    pub fn valid_records(&self) -> usize {
        self.records.iter().filter(|r| r.valid).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.5,TypeA").unwrap();
        writeln!(temp_file, "2,200.0,TypeB").unwrap();
        writeln!(temp_file, "3,1500.0,TypeA").unwrap();
        
        let result = processor.load_from_csv(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.total_records(), 3);
        assert_eq!(processor.valid_records(), 2);
        
        let avg = processor.average_value();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 150.25).abs() < 0.001);
        
        let counts = processor.count_by_category();
        assert_eq!(counts.get("TypeA"), Some(&1));
        assert_eq!(counts.get("TypeB"), Some(&1));
    }
}