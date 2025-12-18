use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug)]
pub struct CsvConfig {
    delimiter: char,
    selected_columns: Vec<usize>,
    has_header: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        Self {
            delimiter: ',',
            selected_columns: Vec::new(),
            has_header: true,
        }
    }
}

pub struct CsvProcessor {
    config: CsvConfig,
}

impl CsvProcessor {
    pub fn new(config: CsvConfig) -> Self {
        Self { config }
    }

    pub fn process_file<P: AsRef<Path>>(&self, input_path: P, output_path: P) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(output_path)?;

        let mut lines = reader.lines();
        
        if self.config.has_header {
            if let Some(header) = lines.next() {
                let header = header?;
                let processed_header = self.process_line(&header)?;
                writeln!(output_file, "{}", processed_header)?;
            }
        }

        for line_result in lines {
            let line = line_result?;
            let processed_line = self.process_line(&line)?;
            writeln!(output_file, "{}", processed_line)?;
        }

        Ok(())
    }

    fn process_line(&self, line: &str) -> Result<String, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(self.config.delimiter).collect();
        
        if self.config.selected_columns.is_empty() {
            return Ok(line.to_string());
        }

        let selected: Vec<&str> = self.config.selected_columns
            .iter()
            .filter_map(|&idx| parts.get(idx).copied())
            .collect();

        Ok(selected.join(&self.config.delimiter.to_string()))
    }

    pub fn process_stdin(&self) -> Result<(), Box<dyn Error>> {
        let stdin = io::stdin();
        let reader = stdin.lock();

        for line_result in reader.lines() {
            let line = line_result?;
            let processed_line = self.process_line(&line)?;
            println!("{}", processed_line);
        }

        Ok(())
    }
}

pub fn create_config(delimiter: char, columns: Option<Vec<usize>>, has_header: bool) -> CsvConfig {
    CsvConfig {
        delimiter,
        selected_columns: columns.unwrap_or_default(),
        has_header,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let config = CsvConfig {
            delimiter: ',',
            selected_columns: vec![0, 2],
            has_header: true,
        };

        let processor = CsvProcessor::new(config);

        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "Name,Age,City").unwrap();
        writeln!(input_file, "Alice,30,New York").unwrap();
        writeln!(input_file, "Bob,25,London").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        processor.process_file(input_file.path(), output_file.path()).unwrap();

        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        assert_eq!(output_content, "Name,City\nAlice,New York\nBob,London\n");
    }

    #[test]
    fn test_line_processing() {
        let config = CsvConfig::default();
        let processor = CsvProcessor::new(config);
        
        let line = "a,b,c,d,e";
        let processed = processor.process_line(line).unwrap();
        assert_eq!(processed, "a,b,c,d,e");

        let config2 = CsvConfig {
            selected_columns: vec![1, 3],
            ..CsvConfig::default()
        };
        let processor2 = CsvProcessor::new(config2);
        let processed2 = processor2.process_line(line).unwrap();
        assert_eq!(processed2, "b,d");
    }
}