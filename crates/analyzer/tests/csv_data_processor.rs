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

    pub fn process_file(&self, file_path: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines_iter = reader.lines().enumerate();

        if self.has_headers {
            lines_iter.next();
        }

        for (line_num, line) in lines_iter {
            let line_content = line?;
            let record: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !record.is_empty() && !record.iter().all(|field| field.is_empty()) {
                records.push(record);
            } else {
                eprintln!("Warning: Skipping empty line at {}", line_num + 1);
            }
        }

        Ok(records)
    }

    pub fn validate_numeric_fields(&self, records: &[Vec<String>], field_index: usize) -> Vec<f64> {
        let mut numeric_values = Vec::new();

        for (row_num, record) in records.iter().enumerate() {
            if field_index < record.len() {
                match record[field_index].parse::<f64>() {
                    Ok(value) => numeric_values.push(value),
                    Err(_) => eprintln!(
                        "Warning: Non-numeric value at row {}, field {}: '{}'",
                        row_num + 1,
                        field_index,
                        record[field_index]
                    ),
                }
            } else {
                eprintln!(
                    "Warning: Row {} doesn't have field at index {}",
                    row_num + 1,
                    field_index
                );
            }
        }

        numeric_values
    }

    pub fn calculate_statistics(&self, numeric_values: &[f64]) -> (f64, f64, f64) {
        if numeric_values.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = numeric_values.iter().sum();
        let count = numeric_values.len() as f64;
        let mean = sum / count;

        let variance: f64 = numeric_values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>()
            / count;

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
        let csv_content = "name,age,salary\nJohn,30,50000.0\nJane,25,60000.0\nBob,35,55000.0";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();

        let processor = CsvProcessor::new(',', true);
        let records = processor.process_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["John", "30", "50000.0"]);

        let salaries = processor.validate_numeric_fields(&records, 2);
        assert_eq!(salaries.len(), 3);

        let stats = processor.calculate_statistics(&salaries);
        assert!((stats.0 - 55000.0).abs() < 0.01);
    }

    #[test]
    fn test_empty_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "").unwrap();

        let processor = CsvProcessor::new(',', false);
        let records = processor.process_file(temp_file.path().to_str().unwrap()).unwrap();

        assert!(records.is_empty());
    }
}