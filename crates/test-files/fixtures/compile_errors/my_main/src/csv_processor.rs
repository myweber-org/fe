
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
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: usize,
    filter_value: String,
}

impl CsvProcessor {
    pub fn new(input: &str, output: &str, column: usize, value: &str) -> Self {
        CsvProcessor {
            input_path: input.to_string(),
            output_path: output.to_string(),
            filter_column,
            filter_value: value.to_string(),
        }
    }

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        let mut processed_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if parts.get(self.filter_column)
                .map(|val| val.trim() == self.filter_value)
                .unwrap_or(false)
            {
                let transformed = parts.iter()
                    .map(|s| s.to_uppercase())
                    .collect::<Vec<String>>()
                    .join(",");
                writeln!(output_file, "{}", transformed)?;
                processed_count += 1;
            }
        }

        Ok(processed_count)
    }
}

pub fn validate_csv_header(file_path: &str) -> Result<bool, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    
    if let Some(first_line) = reader.lines().next() {
        let header = first_line?;
        return Ok(!header.is_empty() && header.contains(','));
    }
    
    Ok(false)
}