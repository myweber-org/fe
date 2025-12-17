
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: Option<usize>,
    filter_value: Option<String>,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            filter_column: None,
            filter_value: None,
        }
    }

    pub fn with_filter(mut self, column: usize, value: &str) -> Self {
        self.filter_column = Some(column);
        self.filter_value = Some(value.to_string());
        self
    }

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;

        let mut processed_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            let fields: Vec<&str> = line.split(',').collect();

            if line_num == 0 {
                writeln!(&mut output_file, "{}", line)?;
                continue;
            }

            let should_include = match (self.filter_column, &self.filter_value) {
                (Some(col), Some(val)) if col < fields.len() => fields[col] == val,
                _ => true,
            };

            if should_include {
                let transformed_line = self.transform_line(&fields);
                writeln!(&mut output_file, "{}", transformed_line)?;
                processed_count += 1;
            }
        }

        Ok(processed_count)
    }

    fn transform_line(&self, fields: &[&str]) -> String {
        fields
            .iter()
            .map(|field| field.trim().to_uppercase())
            .collect::<Vec<String>>()
            .join(",")
    }
}

pub fn validate_csv_file(path: &str) -> Result<bool, Box<dyn Error>> {
    let path_obj = Path::new(path);
    if !path_obj.exists() {
        return Err("File does not exist".into());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    for line in reader.lines().take(5) {
        let line = line?;
        if line.split(',').count() < 2 {
            return Ok(false);
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let input_data = "id,name,value\n1,test,100\n2,example,200\n3,test,300";
        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", input_data).unwrap();

        let output_file = NamedTempFile::new().unwrap();
        let output_path = output_file.path().to_str().unwrap().to_string();

        let processor = CsvProcessor::new(
            input_file.path().to_str().unwrap(),
            &output_path,
        ).with_filter(1, "test");

        let result = processor.process();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);

        let mut output_content = String::new();
        File::open(&output_path)
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();

        assert!(output_content.contains("1,TEST,100"));
        assert!(output_content.contains("3,TEST,300"));
        assert!(!output_content.contains("2,EXAMPLE,200"));
    }

    #[test]
    fn test_csv_validation() {
        let valid_data = "field1,field2,field3\nvalue1,value2,value3";
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", valid_data).unwrap();

        let result = validate_csv_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}