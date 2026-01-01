
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

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

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            let parts: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if parts.len() > self.filter_column {
                if parts[self.filter_column] == self.filter_value {
                    writeln!(output_file, "{}", line)?;
                    processed_count += 1;
                }
            }
        }

        Ok(processed_count)
    }

    pub fn transform_column(&self, column_index: usize, transformer: fn(&str) -> String) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        let mut transformed_count = 0;

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            let mut parts: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if parts.len() > column_index {
                parts[column_index] = &transformer(parts[column_index]);
                transformed_count += 1;
            }

            let transformed_line = parts.join(",");
            writeln!(output_file, "{}", transformed_line)?;
        }

        Ok(transformed_count)
    }
}

pub fn uppercase_transform(value: &str) -> String {
    value.to_uppercase()
}

pub fn trim_transform(value: &str) -> String {
    value.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_csv_processing() {
        let test_data = "id,name,status\n1,alice,active\n2,bob,inactive\n3,charlie,active\n";
        let input_path = "test_input.csv";
        let output_path = "test_output.csv";

        fs::write(input_path, test_data).unwrap();

        let processor = CsvProcessor::new(input_path, output_path, 2, "active");
        let result = processor.process();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);

        let output_content = fs::read_to_string(output_path).unwrap();
        assert!(output_content.contains("alice"));
        assert!(!output_content.contains("bob"));
        assert!(output_content.contains("charlie"));

        fs::remove_file(input_path).unwrap();
        fs::remove_file(output_path).unwrap();
    }
}