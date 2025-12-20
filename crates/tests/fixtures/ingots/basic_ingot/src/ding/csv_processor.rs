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
            let _ = lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !record.is_empty() && !record.iter().all(|field| field.is_empty()) {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_numeric_fields(&self, records: &[Vec<String>], field_index: usize) -> Result<Vec<f64>, String> {
        let mut numeric_values = Vec::new();
        
        for (row_num, record) in records.iter().enumerate() {
            if field_index >= record.len() {
                return Err(format!("Row {}: Field index {} out of bounds", row_num + 1, field_index));
            }
            
            match record[field_index].parse::<f64>() {
                Ok(value) => numeric_values.push(value),
                Err(_) => return Err(format!("Row {}: Invalid numeric value '{}'", 
                    row_num + 1, record[field_index])),
            }
        }
        
        Ok(numeric_values)
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> (f64, f64, f64) {
        if values.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
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
    fn test_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.5").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let records = processor.parse_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["Alice", "30", "50000.5"]);
    }

    #[test]
    fn test_numeric_validation() {
        let records = vec![
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
        ];
        
        let processor = CsvProcessor::new(',', false);
        let numeric_values = processor.validate_numeric_fields(&records, 1).unwrap();
        
        assert_eq!(numeric_values, vec![30.0, 25.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let values = vec![10.0, 20.0, 30.0, 40.0];
        let processor = CsvProcessor::new(',', false);
        let (mean, variance, std_dev) = processor.calculate_statistics(&values);
        
        assert!((mean - 25.0).abs() < 0.001);
        assert!((variance - 125.0).abs() < 0.001);
        assert!((std_dev - 11.18034).abs() < 0.001);
    }
}