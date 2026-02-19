
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
}