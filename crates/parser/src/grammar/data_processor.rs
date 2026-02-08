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

    pub fn process_csv(&self, filter_column: usize, filter_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut results = Vec::new();
        let mut line_count = 0;

        for line in reader.lines() {
            let line = line?;
            line_count += 1;

            if line_count == 1 {
                continue;
            }

            let columns: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();

            if columns.len() > filter_column {
                if columns[filter_column] == filter_value {
                    results.push(columns);
                }
            }
        }

        Ok(results)
    }

    pub fn calculate_average(&self, column_index: usize) -> Result<f64, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut sum = 0.0;
        let mut count = 0;
        let mut line_count = 0;

        for line in reader.lines() {
            let line = line?;
            line_count += 1;

            if line_count == 1 {
                continue;
            }

            let columns: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();

            if columns.len() > column_index {
                if let Ok(value) = columns[column_index].parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Ok(sum / count as f64)
        } else {
            Ok(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value").unwrap();
        writeln!(temp_file, "1,test,100").unwrap();
        writeln!(temp_file, "2,example,200").unwrap();
        writeln!(temp_file, "3,test,150").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process_csv(1, "test").unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], "1");
        assert_eq!(result[1][0], "3");
    }

    #[test]
    fn test_calculate_average() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value").unwrap();
        writeln!(temp_file, "1,10.5").unwrap();
        writeln!(temp_file, "2,20.5").unwrap();
        writeln!(temp_file, "3,30.5").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let average = processor.calculate_average(1).unwrap();

        assert!((average - 20.5).abs() < 0.001);
    }
}