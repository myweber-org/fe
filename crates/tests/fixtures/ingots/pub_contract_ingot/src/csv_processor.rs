
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
            let fields: Vec<String> = line
                .split(self.config.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            let processed_fields = if let Some(ref selected) = self.config.selected_columns {
                selected
                    .iter()
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

    pub fn filter_records<F>(&self, records: &[Vec<String>], predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        records
            .iter()
            .filter(|record| predicate(record))
            .cloned()
            .collect()
    }
}

pub fn create_tab_delimited_config() -> CsvConfig {
    CsvConfig {
        delimiter: '\t',
        selected_columns: Some(vec![0, 2, 4]),
        has_headers: false,
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    file_path: String,
    delimiter: char,
}

impl CsvProcessor {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        CsvProcessor {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn filter_by_column_value(&self, column_index: usize, target_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let columns: Vec<String> = line.split(self.delimiter).map(|s| s.to_string()).collect();
            
            if column_index < columns.len() && columns[column_index] == target_value {
                results.push(columns);
            }
        }

        Ok(results)
    }

    pub fn count_rows(&self) -> Result<usize, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        Ok(reader.lines().count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_by_column_value() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,30,Paris").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let result = processor.filter_by_column_value(1, "30").unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], "Alice");
        assert_eq!(result[1][0], "Charlie");
    }

    #[test]
    fn test_count_rows() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "header1,header2").unwrap();
        writeln!(temp_file, "value1,value2").unwrap();
        writeln!(temp_file, "value3,value4").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap(), ',');
        assert_eq!(processor.count_rows().unwrap(), 3);
    }
}