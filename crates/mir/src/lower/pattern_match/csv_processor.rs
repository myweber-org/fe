
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
    pub fn new(input_path: &str, output_path: &str, filter_column: usize, filter_value: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            filter_column,
            filter_value: filter_value.to_string(),
        }
    }

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(Path::new(&self.input_path))?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(Path::new(&self.output_path))?;
        
        let mut processed_count = 0;
        
        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            
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
    
    pub fn transform_column(&self, target_column: usize, transformer: fn(&str) -> String) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(Path::new(&self.input_path))?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(Path::new(&self.output_path))?;
        
        let mut transformed_count = 0;
        
        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            
            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }
            
            let mut columns: Vec<&str> = line.split(',').collect();
            
            if columns.len() > target_column {
                let original_value = columns[target_column];
                let transformed_value = transformer(original_value);
                columns[target_column] = &transformed_value;
                
                let new_line = columns.join(",");
                writeln!(output_file, "{}", new_line)?;
                transformed_count += 1;
            }
        }
        
        Ok(transformed_count)
    }
}

pub fn uppercase_transformer(value: &str) -> String {
    value.to_uppercase()
}

pub fn trim_transformer(value: &str) -> String {
    value.trim().to_string()
}