
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

    pub fn read_and_validate(&self, file_path: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line?;
            
            if line_content.trim().is_empty() {
                continue;
            }

            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.is_empty() {
                return Err(format!("Line {}: Empty record", line_number).into());
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err("File contains no valid data".into());
        }

        Ok(records)
    }

    pub fn transform_numeric_fields(&self, data: &[Vec<String>]) -> Vec<Vec<f64>> {
        let start_index = if self.has_headers { 1 } else { 0 };
        
        data.iter()
            .skip(start_index)
            .filter_map(|record| {
                let numeric_fields: Vec<f64> = record
                    .iter()
                    .filter_map(|field| field.parse::<f64>().ok())
                    .collect();
                
                if numeric_fields.is_empty() {
                    None
                } else {
                    Some(numeric_fields)
                }
            })
            .collect()
    }

    pub fn calculate_column_averages(&self, numeric_data: &[Vec<f64>]) -> Vec<f64> {
        if numeric_data.is_empty() {
            return Vec::new();
        }

        let column_count = numeric_data[0].len();
        let mut sums = vec![0.0; column_count];
        let mut counts = vec![0; column_count];

        for row in numeric_data {
            for (i, &value) in row.iter().enumerate() {
                if i < column_count {
                    sums[i] += value;
                    counts[i] += 1;
                }
            }
        }

        sums.iter()
            .zip(counts.iter())
            .map(|(&sum, &count)| if count > 0 { sum / count as f64 } else { 0.0 })
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
        writeln!(temp_file, "col1,col2,col3").unwrap();
        writeln!(temp_file, "1.5,2.0,3.5").unwrap();
        writeln!(temp_file, "2.0,3.0,4.0").unwrap();
        writeln!(temp_file, "3.5,4.0,5.5").unwrap();

        let processor = CsvProcessor::new(',', true);
        let data = processor.read_and_validate(temp_file.path().to_str().unwrap()).unwrap();
        let numeric_data = processor.transform_numeric_fields(&data);
        let averages = processor.calculate_column_averages(&numeric_data);

        assert_eq!(averages.len(), 3);
        assert!((averages[0] - 2.333).abs() < 0.001);
        assert!((averages[1] - 3.0).abs() < 0.001);
        assert!((averages[2] - 4.333).abs() < 0.001);
    }
}