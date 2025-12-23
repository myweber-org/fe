
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

    pub fn validate_file(&self, file_path: &str) -> Result<bool, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        let mut line_count = 0;
        let mut column_count: Option<usize> = None;
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() {
                continue;
            }
            
            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            if let Some(expected_count) = column_count {
                if columns.len() != expected_count {
                    return Err(format!(
                        "Line {} has {} columns, expected {}",
                        index + 1,
                        columns.len(),
                        expected_count
                    ).into());
                }
            } else {
                column_count = Some(columns.len());
            }
            
            line_count += 1;
        }
        
        if line_count == 0 {
            return Err("File is empty".into());
        }
        
        Ok(true)
    }

    pub fn transform_column(&self, file_path: &str, column_index: usize, transform_fn: fn(&str) -> String) -> Result<Vec<String>, Box<dyn Error>> {
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
            
            if line.trim().is_empty() {
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

    pub fn calculate_column_stats(&self, file_path: &str, column_index: usize) -> Result<(f64, f64, f64), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        let mut values = Vec::new();
        let mut skip_first = self.has_headers;
        
        for line in reader.lines() {
            let line = line?;
            
            if skip_first {
                skip_first = false;
                continue;
            }
            
            if line.trim().is_empty() {
                continue;
            }
            
            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            if column_index < columns.len() {
                if let Ok(value) = columns[column_index].parse::<f64>() {
                    values.push(value);
                }
            }
        }
        
        if values.is_empty() {
            return Err("No valid numeric values found".into());
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

fn uppercase_transform(value: &str) -> String {
    value.to_uppercase()
}

fn trim_transform(value: &str) -> String {
    value.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_validation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,25,New York").unwrap();
        writeln!(temp_file, "Jane,30,London").unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.validate_file(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_column_transformation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age").unwrap();
        writeln!(temp_file, "john,25").unwrap();
        writeln!(temp_file, "jane,30").unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.transform_column(temp_file.path().to_str().unwrap(), 0, uppercase_transform);
        
        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert_eq!(transformed, vec!["JOHN", "JANE"]);
    }

    #[test]
    fn test_statistics_calculation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "value").unwrap();
        writeln!(temp_file, "10.5").unwrap();
        writeln!(temp_file, "20.5").unwrap();
        writeln!(temp_file, "30.5").unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.calculate_column_stats(temp_file.path().to_str().unwrap(), 0);
        
        assert!(result.is_ok());
        let (mean, variance, std_dev) = result.unwrap();
        assert!((mean - 20.5).abs() < 0.001);
        assert!((variance - 66.666).abs() < 0.001);
        assert!((std_dev - 8.1649).abs() < 0.001);
    }
}