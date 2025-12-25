use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn process_csv(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut records = Vec::new();
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() {
                continue;
            }
            
            let fields: Vec<String> = line.split(',')
                .map(|s| s.trim().to_string())
                .collect();
            
            if fields.len() < 2 {
                return Err(format!("Invalid CSV format at line {}", index + 1).into());
            }
            
            records.push(fields);
        }
        
        if records.is_empty() {
            return Err("CSV file is empty".into());
        }
        
        Ok(records)
    }
    
    pub fn validate_numeric_fields(&self, records: &[Vec<String>], column_index: usize) -> Result<Vec<f64>, Box<dyn Error>> {
        let mut numeric_values = Vec::new();
        
        for (row_index, record) in records.iter().enumerate() {
            if column_index >= record.len() {
                return Err(format!("Column index {} out of bounds at row {}", column_index, row_index + 1).into());
            }
            
            let value = &record[column_index];
            match value.parse::<f64>() {
                Ok(num) => numeric_values.push(num),
                Err(_) => return Err(format!("Non-numeric value '{}' at row {}, column {}", value, row_index + 1, column_index + 1).into()),
            }
        }
        
        Ok(numeric_values)
    }
    
    pub fn calculate_statistics(&self, numbers: &[f64]) -> (f64, f64, f64) {
        if numbers.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = numbers.iter().sum();
        let count = numbers.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = numbers.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_process_csv_valid() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process_csv().unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "50000"]);
        assert_eq!(result[1], vec!["Bob", "25", "45000"]);
    }
    
    #[test]
    fn test_validate_numeric_fields() {
        let records = vec![
            vec!["Alice".to_string(), "30".to_string(), "50000".to_string()],
            vec!["Bob".to_string(), "25".to_string(), "45000".to_string()],
        ];
        
        let processor = DataProcessor::new("dummy.csv");
        let ages = processor.validate_numeric_fields(&records, 1).unwrap();
        
        assert_eq!(ages, vec![30.0, 25.0]);
    }
    
    #[test]
    fn test_calculate_statistics() {
        let numbers = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let processor = DataProcessor::new("dummy.csv");
        let (mean, variance, std_dev) = processor.calculate_statistics(&numbers);
        
        assert_eq!(mean, 30.0);
        assert_eq!(variance, 200.0);
        assert_eq!(std_dev, 200.0_f64.sqrt());
    }
}