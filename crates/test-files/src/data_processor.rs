
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
    delimiter: char,
}

impl DataProcessor {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn process(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut records = Vec::new();
        
        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() {
                continue;
            }
            
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if fields.iter().all(|f| f.is_empty()) {
                continue;
            }
            
            records.push(fields);
            
            if line_number % 1000 == 0 && line_number > 0 {
                println!("Processed {} lines", line_number);
            }
        }
        
        Ok(records)
    }
    
    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), Box<dyn Error>> {
        if records.is_empty() {
            return Err("No valid records found".into());
        }
        
        let header_len = records[0].len();
        
        for (idx, record) in records.iter().enumerate() {
            if record.len() != header_len {
                return Err(format!(
                    "Record {} has {} fields, expected {}",
                    idx,
                    record.len(),
                    header_len
                ).into());
            }
            
            for (field_idx, field) in record.iter().enumerate() {
                if field.is_empty() {
                    return Err(format!(
                        "Empty field at record {}, position {}",
                        idx, field_idx
                    ).into());
                }
            }
        }
        
        Ok(())
    }
}

pub fn calculate_statistics(records: &[Vec<String>]) -> Vec<(String, usize, f64)> {
    if records.is_empty() {
        return Vec::new();
    }
    
    let num_columns = records[0].len();
    let mut results = Vec::with_capacity(num_columns);
    
    for col_idx in 0..num_columns {
        let mut numeric_values = Vec::new();
        let column_name = if col_idx == 0 {
            "ID".to_string()
        } else {
            format!("Column_{}", col_idx)
        };
        
        for record in records.iter().skip(1) {
            if let Some(field) = record.get(col_idx) {
                if let Ok(value) = field.parse::<f64>() {
                    numeric_values.push(value);
                }
            }
        }
        
        if !numeric_values.is_empty() {
            let count = numeric_values.len();
            let sum: f64 = numeric_values.iter().sum();
            let average = sum / count as f64;
            
            results.push((column_name, count, average));
        }
    }
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value").unwrap();
        writeln!(temp_file, "1,item1,10.5").unwrap();
        writeln!(temp_file, "2,item2,20.3").unwrap();
        writeln!(temp_file, "3,item3,15.7").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let records = processor.process().unwrap();
        
        assert_eq!(records.len(), 4);
        assert_eq!(records[0], vec!["id", "name", "value"]);
        assert_eq!(records[1], vec!["1", "item1", "10.5"]);
        
        processor.validate_records(&records).unwrap();
        
        let stats = calculate_statistics(&records);
        assert_eq!(stats.len(), 3);
    }
    
    #[test]
    fn test_empty_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let records = processor.process().unwrap();
        
        assert!(records.is_empty());
        
        let result = processor.validate_records(&records);
        assert!(result.is_err());
    }
}