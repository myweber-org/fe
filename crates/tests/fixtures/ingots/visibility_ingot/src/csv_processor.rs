
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: usize,
    filter_value: String,
    transform_column: usize,
    transform_fn: Box<dyn Fn(&str) -> String>,
}

impl CsvProcessor {
    pub fn new(
        input_path: String,
        output_path: String,
        filter_column: usize,
        filter_value: String,
        transform_column: usize,
        transform_fn: Box<dyn Fn(&str) -> String>,
    ) -> Self {
        CsvProcessor {
            input_path,
            output_path,
            filter_column,
            filter_value,
            transform_column,
            transform_fn,
        }
    }

    pub fn process(&self) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(Path::new(&self.input_path))?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(Path::new(&self.output_path))?;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if parts.len() <= self.filter_column.max(self.transform_column) {
                continue;
            }

            if parts[self.filter_column] != self.filter_value {
                continue;
            }

            let mut transformed_parts = parts.clone();
            transformed_parts[self.transform_column] = &(self.transform_fn)(parts[self.transform_column]);
            
            let transformed_line = transformed_parts.join(",");
            writeln!(output_file, "{}", transformed_line)?;
        }

        Ok(())
    }
}

pub fn create_uppercase_transformer() -> Box<dyn Fn(&str) -> String> {
    Box::new(|s: &str| s.to_uppercase())
}

pub fn create_prefix_transformer(prefix: &str) -> Box<dyn Fn(&str) -> String> {
    let prefix = prefix.to_string();
    Box::new(move |s: &str| format!("{}{}", prefix, s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_csv_processing() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        
        let input_content = "id,name,status\n1,alice,active\n2,bob,inactive\n3,charlie,active\n";
        fs::write(test_input, input_content).unwrap();

        let processor = CsvProcessor::new(
            test_input.to_string(),
            test_output.to_string(),
            2,
            "active".to_string(),
            1,
            create_uppercase_transformer(),
        );

        let result = processor.process();
        assert!(result.is_ok());

        let output_content = fs::read_to_string(test_output).unwrap();
        let expected = "id,name,status\n1,ALICE,active\n3,CHARLIE,active\n";
        assert_eq!(output_content, expected);

        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}