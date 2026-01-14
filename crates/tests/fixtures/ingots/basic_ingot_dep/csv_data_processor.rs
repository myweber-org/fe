
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

    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 && self.has_header {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_numeric_column(&self, records: &[Vec<String>], column_index: usize) -> Result<Vec<f64>, String> {
        let mut numeric_values = Vec::new();

        for (row_num, record) in records.iter().enumerate() {
            if column_index >= record.len() {
                return Err(format!("Row {}: Column index out of bounds", row_num + 1));
            }

            match record[column_index].parse::<f64>() {
                Ok(value) => numeric_values.push(value),
                Err(_) => return Err(format!("Row {}: Invalid numeric value '{}'", 
                    row_num + 1, record[column_index])),
            }
        }

        Ok(numeric_values)
    }

    pub fn calculate_statistics(&self, numeric_data: &[f64]) -> (f64, f64, f64) {
        if numeric_data.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = numeric_data.iter().sum();
        let count = numeric_data.len() as f64;
        let mean = sum / count;

        let variance: f64 = numeric_data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn filter_records<F>(&self, records: &[Vec<String>], predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        records.iter()
            .filter(|record| predicate(record))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.5").unwrap();
        writeln!(temp_file, "Charlie,35,55000.0").unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor.read_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "50000.0"]);
        
        let ages = processor.validate_numeric_column(&records, 1).unwrap();
        assert_eq!(ages, vec![30.0, 25.0, 35.0]);
        
        let (mean, variance, std_dev) = processor.calculate_statistics(&ages);
        assert!((mean - 30.0).abs() < 0.001);
        
        let filtered = processor.filter_records(&records, |record| {
            record[0].starts_with('A')
        });
        assert_eq!(filtered.len(), 1);
    }
}