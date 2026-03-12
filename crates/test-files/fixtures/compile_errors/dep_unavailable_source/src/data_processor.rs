
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    records: Vec<HashMap<String, String>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let headers = if let Some(header_line) = lines.next() {
            header_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>()
        } else {
            return Ok(());
        };

        for line_result in lines {
            let line = line_result?;
            let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            
            if values.len() != headers.len() {
                continue;
            }

            let mut record = HashMap::new();
            for (i, header) in headers.iter().enumerate() {
                record.insert(header.clone(), values[i].to_string());
            }
            
            self.records.push(record);
        }

        Ok(())
    }

    pub fn calculate_average(&self, column_name: &str) -> Option<f64> {
        let mut sum = 0.0;
        let mut count = 0;

        for record in &self.records {
            if let Some(value_str) = record.get(column_name) {
                if let Ok(value) = value_str.parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Some(sum / count as f64)
        } else {
            None
        }
    }

    pub fn get_unique_values(&self, column_name: &str) -> Vec<String> {
        let mut unique_values = HashMap::new();
        
        for record in &self.records {
            if let Some(value) = record.get(column_name) {
                unique_values.insert(value.clone(), ());
            }
        }
        
        unique_values.keys().cloned().collect()
    }

    pub fn filter_records<F>(&self, predicate: F) -> Vec<HashMap<String, String>>
    where
        F: Fn(&HashMap<String, String>) -> bool,
    {
        self.records
            .iter()
            .filter(|record| predicate(record))
            .cloned()
            .collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
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
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,85.5").unwrap();
        writeln!(temp_file, "Bob,30,92.0").unwrap();
        writeln!(temp_file, "Charlie,25,78.5").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);
        
        let avg_age = processor.calculate_average("age");
        assert!(avg_age.is_some());
        assert!((avg_age.unwrap() - 26.6667).abs() < 0.001);
        
        let unique_ages = processor.get_unique_values("age");
        assert_eq!(unique_ages.len(), 2);
        
        let filtered = processor.filter_records(|record| {
            record.get("age").map(|a| a == "25").unwrap_or(false)
        });
        assert_eq!(filtered.len(), 2);
    }
}