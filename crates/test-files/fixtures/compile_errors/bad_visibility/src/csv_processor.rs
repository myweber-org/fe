use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    delimiter: char,
    selected_columns: Vec<usize>,
    skip_header: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            selected_columns: Vec::new(),
            skip_header: false,
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
        let mut results = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            let line = line?;
            line_number += 1;

            if self.config.skip_header && line_number == 1 {
                continue;
            }

            let columns: Vec<String> = line.split(self.config.delimiter).map(String::from).collect();
            
            if self.config.selected_columns.is_empty() {
                results.push(columns);
            } else {
                let filtered_columns: Vec<String> = self.config.selected_columns
                    .iter()
                    .filter_map(|&idx| columns.get(idx).cloned())
                    .collect();
                
                if !filtered_columns.is_empty() {
                    results.push(filtered_columns);
                }
            }
        }

        Ok(results)
    }

    pub fn extract_column_data(&self, data: &[Vec<String>], column_index: usize) -> Vec<String> {
        data.iter()
            .filter_map(|row| row.get(column_index).cloned())
            .collect()
    }
}

pub fn create_config_with_columns(columns: Vec<usize>) -> CsvConfig {
    CsvConfig {
        selected_columns: columns,
        ..CsvConfig::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let config = CsvConfig::default();
        let processor = CsvProcessor::new(config);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_column_selection() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "A,B,C,D").unwrap();
        writeln!(temp_file, "1,2,3,4").unwrap();

        let config = CsvConfig {
            selected_columns: vec![0, 2],
            ..CsvConfig::default()
        };
        let processor = CsvProcessor::new(config);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result[0], vec!["1", "3"]);
    }
}