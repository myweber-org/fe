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
                return Err(format!("Line {} contains no valid data", line_number).into());
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err("CSV file is empty".into());
        }

        Ok(records)
    }

    pub fn transform_numeric_fields(&self, data: &[Vec<String>]) -> Vec<Vec<String>> {
        let mut transformed = Vec::with_capacity(data.len());
        
        for record in data {
            let transformed_record: Vec<String> = record
                .iter()
                .map(|field| {
                    if let Ok(num) = field.parse::<f64>() {
                        format!("{:.2}", num)
                    } else {
                        field.clone()
                    }
                })
                .collect();
            transformed.push(transformed_record);
        }
        
        transformed
    }

    pub fn filter_by_column_value(
        &self,
        data: &[Vec<String>],
        column_index: usize,
        filter_value: &str,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let start_index = if self.has_headers { 1 } else { 0 };
        let mut filtered = Vec::new();

        if self.has_headers && !data.is_empty() {
            filtered.push(data[0].clone());
        }

        for record in data.iter().skip(start_index) {
            if column_index < record.len() && record[column_index] == filter_value {
                filtered.push(record.clone());
            }
        }

        Ok(filtered)
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
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();
        writeln!(temp_file, "Bob,30.5,Paris").unwrap();
        writeln!(temp_file, "Charlie,35.123,New York").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor.read_and_validate(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.len(), 4);
        
        let transformed = processor.transform_numeric_fields(&data);
        assert_eq!(transformed[2][1], "30.50");
        assert_eq!(transformed[3][1], "35.12");
        
        let filtered = processor.filter_by_column_value(&data, 2, "London").unwrap();
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[1][0], "Alice");
    }
}