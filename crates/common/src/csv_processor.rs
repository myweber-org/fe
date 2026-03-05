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
}use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: Option<usize>,
    filter_value: Option<String>,
    transform_column: Option<usize>,
    transform_fn: Option<Box<dyn Fn(&str) -> String>>,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            filter_column: None,
            filter_value: None,
            transform_column: None,
            transform_fn: None,
        }
    }

    pub fn set_filter(&mut self, column: usize, value: &str) -> &mut Self {
        self.filter_column = Some(column);
        self.filter_value = Some(value.to_string());
        self
    }

    pub fn set_transform<F>(&mut self, column: usize, transform_fn: F) -> &mut Self
    where
        F: Fn(&str) -> String + 'static,
    {
        self.transform_column = Some(column);
        self.transform_fn = Some(Box::new(transform_fn));
        self
    }

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(Path::new(&self.input_path))?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(Path::new(&self.output_path))?;
        
        let mut processed_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            let mut fields: Vec<String> = line.split(',').map(|s| s.to_string()).collect();

            if let (Some(filter_col), Some(filter_val)) = (&self.filter_column, &self.filter_value) {
                if *filter_col >= fields.len() || fields[*filter_col] != *filter_val {
                    continue;
                }
            }

            if let (Some(transform_col), Some(transform_fn)) = (&self.transform_column, &self.transform_fn) {
                if *transform_col < fields.len() {
                    fields[*transform_col] = transform_fn(&fields[*transform_col]);
                }
            }

            let processed_line = fields.join(",");
            writeln!(output_file, "{}", processed_line)?;
            processed_count += 1;
        }

        Ok(processed_count)
    }
}

pub fn process_csv(
    input: &str,
    output: &str,
    filter: Option<(usize, &str)>,
    transform: Option<(usize, Box<dyn Fn(&str) -> String>)>,
) -> Result<usize, Box<dyn Error>> {
    let mut processor = CsvProcessor::new(input, output);
    
    if let Some((col, val)) = filter {
        processor.set_filter(col, val);
    }
    
    if let Some((col, func)) = transform {
        processor.set_transform(col, func);
    }
    
    processor.process()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_basic_processing() -> Result<(), Box<dyn Error>> {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        
        let test_data = "name,age,city\nJohn,25,NYC\nAlice,30,LA\nBob,25,Chicago";
        fs::write(test_input, test_data)?;

        let result = process_csv(
            test_input,
            test_output,
            Some((1, "25")),
            Some((2, Box::new(|s| s.to_uppercase()))),
        )?;

        let output_content = fs::read_to_string(test_output)?;
        let expected = "name,age,city\nJohn,25,NYC\nBob,25,CHICAGO\n";
        
        assert_eq!(output_content, expected);
        assert_eq!(result, 2);

        fs::remove_file(test_input)?;
        fs::remove_file(test_output)?;
        
        Ok(())
    }
}