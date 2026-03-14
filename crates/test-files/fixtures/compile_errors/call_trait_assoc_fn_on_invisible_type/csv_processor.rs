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
            
            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            let columns: Vec<&str> = line.split(',').collect();
            if columns.len() > self.filter_column {
                if columns[self.filter_column] == self.filter_value {
                    writeln!(output_file, "{}", line)?;
                    processed_count += 1;
                }
            }
        }

        Ok(processed_count)
    }

    pub fn transform_column(&self, target_column: usize, transform_fn: fn(&str) -> String) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        let mut transformed_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            let mut columns: Vec<&str> = line.split(',').collect();
            if columns.len() > target_column {
                let original = columns[target_column];
                columns[target_column] = Box::leak(transform_fn(original).into_boxed_str());
                let transformed_line = columns.join(",");
                writeln!(output_file, "{}", transformed_line)?;
                transformed_count += 1;
            }
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