
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
            
            if line.is_empty() {
                continue;
            }

            let record: Vec<String> = line.split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if self.has_header && line_number == 0 {
                continue;
            }

            if !self.validate_record(&record) {
                return Err(format!("Invalid record at line {}", line_number + 1).into());
            }

            records.push(record);
        }

        Ok(records)
    }

    fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, data: &[Vec<String>], column_index: usize) -> Result<Statistics, Box<dyn Error>> {
        let mut values = Vec::new();
        
        for record in data {
            if column_index >= record.len() {
                return Err("Column index out of bounds".into());
            }
            
            if let Ok(value) = record[column_index].parse::<f64>() {
                values.push(value);
            }
        }

        if values.is_empty() {
            return Err("No valid numeric data found".into());
        }

        let sum: f64 = values.iter().sum();
        let count = values.len();
        let mean = sum / count as f64;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        
        let std_dev = variance.sqrt();

        Ok(Statistics {
            count,
            mean,
            std_dev,
            min: *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            max: *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
        })
    }
}

pub struct Statistics {
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

impl std::fmt::Display for Statistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Count: {}, Mean: {:.2}, Std Dev: {:.2}, Min: {:.2}, Max: {:.2}",
               self.count, self.mean, self.std_dev, self.min, self.max)
    }
}