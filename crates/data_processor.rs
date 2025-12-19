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

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if fields.len() < 2 {
                return Err(format!("Invalid data at line {}: insufficient columns", index + 1).into());
            }
            
            records.push(fields);
        }

        if records.is_empty() {
            return Err("File contains no valid data".into());
        }

        Ok(records)
    }

    pub fn calculate_average(&self, column_index: usize) -> Result<f64, Box<dyn Error>> {
        let records = self.process()?;
        let mut sum = 0.0;
        let mut count = 0;

        for record in records.iter().skip(1) {
            if let Some(value) = record.get(column_index) {
                if let Ok(num) = value.parse::<f64>() {
                    sum += num;
                    count += 1;
                }
            }
        }

        if count == 0 {
            return Err("No valid numeric data found in specified column".into());
        }

        Ok(sum / count as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process().unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["name", "age", "salary"]);
    }

    #[test]
    fn test_average_calculation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,salary").unwrap();
        writeln!(temp_file, "Alice,50000.0").unwrap();
        writeln!(temp_file, "Bob,45000.0").unwrap();
        writeln!(temp_file, "Charlie,55000.0").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let average = processor.calculate_average(1).unwrap();
        
        assert!((average - 50000.0).abs() < 0.01);
    }
}