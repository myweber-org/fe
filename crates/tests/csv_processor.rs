use std::error::Error;
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

            if parts.get(self.filter_column).map_or(false, |&val| val == self.filter_value) {
                writeln!(output_file, "{}", line)?;
                processed_count += 1;
            }
        }

        Ok(processed_count)
    }

    pub fn transform_column<F>(&self, transform_fn: F) -> Result<(), Box<dyn Error>>
    where
        F: Fn(&str) -> String,
    {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let mut parts: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if let Some(cell) = parts.get_mut(self.filter_column) {
                *cell = &transform_fn(cell);
            }

            let transformed_line = parts.join(",");
            writeln!(output_file, "{}", transformed_line)?;
        }

        Ok(())
    }
}

pub fn validate_csv_format(content: &str) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return false;
    }

    let column_count = lines[0].split(',').count();
    lines.iter().all(|line| line.split(',').count() == column_count)
}use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

pub fn filter_csv(input_path: &str, output_path: &str, column_filter: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let reader = BufReader::new(file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
    
    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new().from_writer(writer);
    
    let headers = csv_reader.headers()?.clone();
    csv_writer.write_record(&headers)?;
    
    let column_index = headers.iter()
        .position(|h| h == column_filter)
        .ok_or_else(|| format!("Column '{}' not found", column_filter))?;
    
    for result in csv_reader.records() {
        let record = result?;
        if let Some(field) = record.get(column_index) {
            if !field.trim().is_empty() && field != "null" && field != "NULL" {
                csv_writer.write_record(&record)?;
            }
        }
    }
    
    csv_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_filter_csv() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name,age,city\nJohn,25,NYC\nJane,,London\nBob,30,").unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let result = filter_csv(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            "age"
        );
        
        assert!(result.is_ok());
        
        let output_content = std::fs::read_to_string(output_file.path()).unwrap();
        assert!(output_content.contains("John,25,NYC"));
        assert!(output_content.contains("Bob,30,"));
        assert!(!output_content.contains("Jane,,London"));
    }
}