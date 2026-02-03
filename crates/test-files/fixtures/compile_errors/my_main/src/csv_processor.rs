
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    pub delimiter: char,
    pub has_headers: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
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

    pub fn filter_rows<P: AsRef<Path>>(
        &self,
        file_path: P,
        predicate: impl Fn(&[String]) -> bool,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();
        let mut lines = reader.lines();

        if self.config.has_headers {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.config.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if predicate(&fields) {
                results.push(fields);
            }
        }

        Ok(results)
    }

    pub fn extract_column<P: AsRef<Path>>(
        &self,
        file_path: P,
        column_index: usize,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut column_data = Vec::new();
        let mut lines = reader.lines();

        if self.config.has_headers {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<&str> = line.split(self.config.delimiter).collect();

            if let Some(&value) = fields.get(column_index) {
                column_data.push(value.trim().to_string());
            }
        }

        Ok(column_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,city").unwrap();
        writeln!(file, "Alice,30,New York").unwrap();
        writeln!(file, "Bob,25,London").unwrap();
        writeln!(file, "Charlie,35,Paris").unwrap();
        file
    }

    #[test]
    fn test_filter_rows() {
        let csv_file = create_test_csv();
        let processor = CsvProcessor::new(CsvConfig::default());

        let result = processor
            .filter_rows(csv_file.path(), |fields| {
                fields.get(1).and_then(|age| age.parse::<i32>().ok()) > Some(30)
            })
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0][0], "Charlie");
    }

    #[test]
    fn test_extract_column() {
        let csv_file = create_test_csv();
        let processor = CsvProcessor::new(CsvConfig::default());

        let column = processor.extract_column(csv_file.path(), 2).unwrap();

        assert_eq!(column, vec!["New York", "London", "Paris"]);
    }
}