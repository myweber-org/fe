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

    pub fn process_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 && self.has_header {
                continue;
            }

            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn validate_numeric_fields(&self, records: &[Vec<String>], field_index: usize) -> Vec<f64> {
        let mut numeric_values = Vec::new();

        for record in records {
            if let Some(field) = record.get(field_index) {
                if let Ok(value) = field.parse::<f64>() {
                    numeric_values.push(value);
                }
            }
        }

        numeric_values
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
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.5").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.75").unwrap();

        let processor = DataProcessor::new(',', true);
        let records = processor.process_csv(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "50000.5"]);
    }

    #[test]
    fn test_statistics_calculation() {
        let processor = DataProcessor::new(',', false);
        let values = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let (mean, variance, std_dev) = processor.calculate_statistics(&values);
        
        assert_eq!(mean, 30.0);
        assert_eq!(variance, 200.0);
        assert_eq!(std_dev, 14.142135623730951);
    }
}
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.message)
    }
}

impl Error for ValidationError {}

pub struct DataProcessor {
    threshold: f64,
}

impl DataProcessor {
    pub fn new(threshold: f64) -> Result<Self, ValidationError> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(ValidationError {
                message: format!("Threshold {} must be between 0.0 and 1.0", threshold),
            });
        }
        Ok(Self { threshold })
    }

    pub fn process_values(&self, values: &[f64]) -> Result<Vec<f64>, ValidationError> {
        if values.is_empty() {
            return Err(ValidationError {
                message: "Input values cannot be empty".to_string(),
            });
        }

        let normalized = self.normalize_data(values)?;
        let filtered = self.filter_values(&normalized);
        Ok(filtered)
    }

    fn normalize_data(&self, values: &[f64]) -> Result<Vec<f64>, ValidationError> {
        let max_value = values
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        if max_value <= 0.0 {
            return Err(ValidationError {
                message: "All values must be positive for normalization".to_string(),
            });
        }

        Ok(values
            .iter()
            .map(|&v| v / max_value)
            .collect())
    }

    fn filter_values(&self, values: &[f64]) -> Vec<f64> {
        values
            .iter()
            .filter(|&&v| v >= self.threshold)
            .cloned()
            .collect()
    }

    pub fn calculate_statistics(&self, values: &[f64]) -> Result<(f64, f64), ValidationError> {
        if values.len() < 2 {
            return Err(ValidationError {
                message: "At least two values required for statistics".to_string(),
            });
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values
            .iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;

        Ok((mean, variance.sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let processor = DataProcessor::new(0.5);
        assert!(processor.is_ok());
        
        let invalid = DataProcessor::new(1.5);
        assert!(invalid.is_err());
    }

    #[test]
    fn test_normalization() {
        let processor = DataProcessor::new(0.5).unwrap();
        let values = vec![1.0, 2.0, 3.0];
        let normalized = processor.normalize_data(&values).unwrap();
        assert_eq!(normalized, vec![1.0/3.0, 2.0/3.0, 1.0]);
    }

    #[test]
    fn test_value_filtering() {
        let processor = DataProcessor::new(0.6).unwrap();
        let values = vec![0.5, 0.7, 0.8, 0.4];
        let filtered = processor.filter_values(&values);
        assert_eq!(filtered, vec![0.7, 0.8]);
    }
}