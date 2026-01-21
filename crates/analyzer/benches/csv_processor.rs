
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

    pub fn validate_file<P: AsRef<Path>>(&self, file_path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut line_count = 0;
        let mut column_count: Option<usize> = None;

        for (index, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            if index == 0 && self.has_header {
                continue;
            }

            if let Some(expected_count) = column_count {
                if columns.len() != expected_count {
                    return Err(format!("Row {} has {} columns, expected {}", 
                        index + 1, columns.len(), expected_count).into());
                }
            } else {
                column_count = Some(columns.len());
            }

            for (col_idx, value) in columns.iter().enumerate() {
                if value.trim().is_empty() {
                    return Err(format!("Empty value at row {}, column {}", 
                        index + 1, col_idx + 1).into());
                }
            }

            line_count += 1;
        }

        if line_count == 0 {
            return Err("File contains no data rows".into());
        }

        Ok(line_count)
    }

    pub fn transform_column<P: AsRef<Path>>(
        &self, 
        file_path: P, 
        column_index: usize,
        transform_fn: fn(&str) -> String
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for (index, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            
            if index == 0 && self.has_header {
                continue;
            }

            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            if column_index >= columns.len() {
                return Err(format!("Column index {} out of bounds for row {}", 
                    column_index, index + 1).into());
            }

            let transformed = transform_fn(columns[column_index]);
            results.push(transformed);
        }

        Ok(results)
    }
}

pub fn uppercase_transform(value: &str) -> String {
    value.to_uppercase()
}

pub fn trim_transform(value: &str) -> String {
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
        let result = processor.validate_file(temp_file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_column_transformation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age").unwrap();
        writeln!(temp_file, "john,25").unwrap();
        writeln!(temp_file, "jane,30").unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.transform_column(temp_file.path(), 0, uppercase_transform);
        
        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert_eq!(transformed, vec!["JOHN", "JANE"]);
    }
}