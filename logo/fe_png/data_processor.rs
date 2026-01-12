
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_number == 0 && self.has_header {
                continue;
            }

            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|field| field.trim().to_string())
                .collect();

            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), String> {
        if records.is_empty() {
            return Err("No records found".to_string());
        }

        let expected_fields = records[0].len();
        
        for (index, record) in records.iter().enumerate() {
            if record.len() != expected_fields {
                return Err(format!(
                    "Record {} has {} fields, expected {}",
                    index + 1,
                    record.len(),
                    expected_fields
                ));
            }
        }

        Ok(())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Result<(f64, f64, f64), String> {
        if records.is_empty() {
            return Err("No records to process".to_string());
        }

        if column_index >= records[0].len() {
            return Err(format!("Column index {} out of bounds", column_index));
        }

        let mut values = Vec::new();
        for record in records {
            if let Ok(value) = record[column_index].parse::<f64>() {
                values.push(value);
            }
        }

        if values.is_empty() {
            return Err("No valid numeric values found".to_string());
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Ok((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,95.5").unwrap();
        writeln!(temp_file, "Bob,30,87.2").unwrap();

        let processor = DataProcessor::new(',', true);
        let records = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["Alice", "25", "95.5"]);
    }

    #[test]
    fn test_validate_records() {
        let records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        assert!(processor.validate_records(&records).is_ok());
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            vec!["10.0".to_string(), "20.0".to_string()],
            vec!["20.0".to_string(), "30.0".to_string()],
            vec!["30.0".to_string(), "40.0".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        let (mean, variance, std_dev) = processor.calculate_statistics(&records, 0).unwrap();

        assert!((mean - 20.0).abs() < 0.001);
        assert!((variance - 66.666).abs() < 0.001);
        assert!((std_dev - 8.1649).abs() < 0.001);
    }
}