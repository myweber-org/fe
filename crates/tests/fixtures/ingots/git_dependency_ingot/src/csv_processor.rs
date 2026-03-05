use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    delimiter: char,
    selected_columns: Option<Vec<usize>>,
    has_header: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            selected_columns: None,
            has_header: true,
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
        let mut lines = reader.lines();

        if self.config.has_header {
            lines.next();
        }

        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let record = self.parse_line(&line);
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str) -> Vec<String> {
        let fields: Vec<String> = line
            .split(self.config.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        match &self.config.selected_columns {
            Some(indices) => {
                indices
                    .iter()
                    .filter_map(|&idx| fields.get(idx).cloned())
                    .collect()
            }
            None => fields,
        }
    }

    pub fn filter_records<F>(&self, records: Vec<Vec<String>>, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        records
            .into_iter()
            .filter(|record| predicate(record))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let config = CsvConfig {
            selected_columns: Some(vec![0, 2]),
            ..Default::default()
        };
        let processor = CsvProcessor::new(config);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "New York"]);
        assert_eq!(result[1], vec!["Bob", "London"]);
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            vec!["apple".to_string(), "10".to_string()],
            vec!["banana".to_string(), "5".to_string()],
            vec!["orange".to_string(), "15".to_string()],
        ];

        let config = CsvConfig::default();
        let processor = CsvProcessor::new(config);
        let filtered = processor.filter_records(records, |fields| {
            fields.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0) > 8
        });

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|f| f[0] == "apple"));
        assert!(filtered.iter().any(|f| f[0] == "orange"));
    }
}