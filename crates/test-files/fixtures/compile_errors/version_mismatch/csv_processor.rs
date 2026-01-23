use std::error::Error;
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
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        let mut processed_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let mut fields: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            if let (Some(filter_col), Some(filter_val)) = (&self.filter_column, &self.filter_value) {
                if *filter_col >= fields.len() || fields[*filter_col] != filter_val {
                    continue;
                }
            }

            if let (Some(transform_col), Some(transform_fn)) = (&self.transform_column, &self.transform_fn) {
                if *transform_col < fields.len() {
                    fields[*transform_col] = &transform_fn(fields[*transform_col]);
                }
            }

            let processed_line = fields.join(",");
            writeln!(output_file, "{}", processed_line)?;
            processed_count += 1;
        }

        Ok(processed_count)
    }
}

pub fn validate_csv_path(path: &str) -> Result<(), String> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err(format!("File does not exist: {}", path));
    }
    if path_obj.extension().and_then(|ext| ext.to_str()) != Some("csv") {
        return Err("File must have .csv extension".to_string());
    }
    Ok(())
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

        let mut processor = CsvProcessor::new(test_input, test_output);
        processor
            .set_filter(1, "test")
            .set_transform(2, |val| format!("{}_modified", val));

        let result = processor.process();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);

        let output_content = fs::read_to_string(test_output).unwrap();
        assert!(output_content.contains("1,test,100_modified"));
        assert!(output_content.contains("3,test,300_modified"));
        assert!(!output_content.contains("2,other,200"));

        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}