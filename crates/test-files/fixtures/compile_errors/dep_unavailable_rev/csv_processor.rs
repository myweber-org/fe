
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: Option<usize>,
    filter_value: Option<String>,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            filter_column: None,
            filter_value: None,
        }
    }

    pub fn set_filter(&mut self, column: usize, value: &str) -> &mut Self {
        self.filter_column = Some(column);
        self.filter_value = Some(value.to_string());
        self
    }

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(Path::new(&self.input_path))?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(Path::new(&self.output_path))?;

        let mut processed_count = 0;

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            let columns: Vec<&str> = line.split(',').collect();

            let should_write = match (self.filter_column, &self.filter_value) {
                (Some(col), Some(val)) if col < columns.len() => columns[col] == val,
                _ => true,
            };

            if should_write {
                let transformed_line = self.transform_line(&columns);
                writeln!(output_file, "{}", transformed_line)?;
                processed_count += 1;
            }

            if line_num % 1000 == 0 && line_num > 0 {
                eprintln!("Processed {} lines...", line_num);
            }
        }

        Ok(processed_count)
    }

    fn transform_line(&self, columns: &[&str]) -> String {
        columns
            .iter()
            .map(|col| col.trim().to_uppercase())
            .collect::<Vec<String>>()
            .join("|")
    }
}

pub fn validate_csv_format(content: &str) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return false;
    }

    let column_count = lines[0].split(',').count();
    lines.iter().all(|line| line.split(',').count() == column_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_csv_processing() {
        let test_input = "data/test_input.csv";
        let test_output = "data/test_output.csv";

        let mut processor = CsvProcessor::new(test_input, test_output);
        processor.set_filter(1, "active");

        let result = processor.process();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_validation() {
        let valid_csv = "name,status,value\njohn,active,100\njane,inactive,200";
        let invalid_csv = "name,status,value\njohn,active\njane,inactive,200,extra";

        assert!(validate_csv_format(valid_csv));
        assert!(!validate_csv_format(invalid_csv));
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    delimiter: char,
    selected_columns: Option<Vec<usize>>,
    has_headers: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            selected_columns: None,
            has_headers: true,
        }
    }
}

pub struct CsvProcessor {
    config: CsvConfig,
}

impl CsvProcessor {
    pub fn new(config: CsvConfig) -> Self {
        CsvProcessor { config }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.config.has_headers {
            let _headers = lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line.split(self.config.delimiter).map(String::from).collect();
            
            let processed_fields = if let Some(ref selected) = self.config.selected_columns {
                selected.iter()
                    .filter_map(|&idx| fields.get(idx).cloned())
                    .collect()
            } else {
                fields
            };
            
            if !processed_fields.is_empty() {
                records.push(processed_fields);
            }
        }

        Ok(records)
    }

    pub fn filter_records<F>(&self, records: Vec<Vec<String>>, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        records.into_iter().filter(|record| predicate(record)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,London").unwrap();
        writeln!(temp_file, "Bob,25,Paris").unwrap();

        let config = CsvConfig {
            selected_columns: Some(vec![0, 2]),
            ..Default::default()
        };
        
        let processor = CsvProcessor::new(config);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "London"]);
        assert_eq!(result[1], vec!["Bob", "Paris"]);
    }

    #[test]
    fn test_filter_function() {
        let records = vec![
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
            vec!["Charlie".to_string(), "35".to_string()],
        ];

        let config = CsvConfig::default();
        let processor = CsvProcessor::new(config);
        
        let filtered = processor.filter_records(records, |fields| {
            if let Ok(age) = fields[1].parse::<i32>() {
                age > 30
            } else {
                false
            }
        });

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0][0], "Charlie");
    }
}