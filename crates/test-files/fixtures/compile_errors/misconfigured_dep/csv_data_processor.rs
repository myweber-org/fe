
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
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = match lines.next() {
            Some(Ok(line)) => line.split(',').map(|s| s.to_string()).collect(),
            _ => return Err("Failed to read headers".into()),
        };

        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line.split(',').map(|s| s.to_string()).collect();
            if fields.len() == headers.len() {
                records.push(fields);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };

        self.records
            .iter()
            .filter(|record| record.get(column_index) == Some(&value.to_string()))
            .cloned()
            .collect()
    }

    pub fn aggregate_numeric_column(&self, column_name: &str) -> Option<f64> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;

        let sum: f64 = self.records
            .iter()
            .filter_map(|record| record.get(column_index))
            .filter_map(|value| value.parse::<f64>().ok())
            .sum();

        let count = self.records.len() as f64;
        if count > 0.0 {
            Some(sum / count)
        } else {
            None
        }
    }

    pub fn count_by_column(&self, column_name: &str) -> HashMap<String, usize> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return HashMap::new(),
        };

        let mut counts = HashMap::new();
        for record in &self.records {
            if let Some(value) = record.get(column_index) {
                *counts.entry(value.clone()).or_insert(0) += 1;
            }
        }
        counts
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_headers(&self) -> &Vec<String> {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();
        writeln!(temp_file, "Bob,30,Paris").unwrap();
        writeln!(temp_file, "Charlie,25,London").unwrap();
        writeln!(temp_file, "Diana,35,Tokyo").unwrap();
        temp_file
    }

    #[test]
    fn test_csv_loading() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(processor.get_record_count(), 4);
        assert_eq!(processor.get_headers(), &vec!["name", "age", "city"]);
    }

    #[test]
    fn test_filter_by_column() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        let filtered = processor.filter_by_column("city", "London");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_aggregate_numeric() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        let avg_age = processor.aggregate_numeric_column("age").unwrap();
        assert!((avg_age - 28.75).abs() < 0.001);
    }

    #[test]
    fn test_count_by_column() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        let city_counts = processor.count_by_column("city");
        assert_eq!(city_counts.get("London"), Some(&2));
        assert_eq!(city_counts.get("Paris"), Some(&1));
    }
}