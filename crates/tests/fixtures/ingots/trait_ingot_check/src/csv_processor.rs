use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, Write};
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
        let input_file = File::open(&self.input_path)?;
        let reader = io::BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();

            if parts.len() <= self.filter_column.max(self.transform_column) {
                eprintln!("Warning: Line {} has insufficient columns", line_num + 1);
                continue;
            }

            if parts[self.filter_column] == self.filter_value {
                let mut transformed_parts = parts.clone();
                transformed_parts[self.transform_column] = &(self.transform_fn)(parts[self.transform_column]);
                writeln!(output_file, "{}", transformed_parts.join(","))?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_csv_processing() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        let test_data = "id,name,value\n1,test,100\n2,other,200\n3,test,300\n";

        fs::write(test_input, test_data).unwrap();

        let processor = CsvProcessor::new(
            test_input.to_string(),
            test_output.to_string(),
            1,
            "test".to_string(),
            2,
            Box::new(|val| format!("{}_modified", val)),
        );

        processor.process().unwrap();

        let output = fs::read_to_string(test_output).unwrap();
        assert_eq!(output, "1,test,100_modified\n3,test,300_modified\n");

        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}