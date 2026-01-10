
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct CsvProcessor {
    delimiter: char,
    has_headers: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_headers: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_headers,
        }
    }

    pub fn validate_file(&self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut line_count = 0;
        let mut column_count: Option<usize> = None;

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            if column_count.is_none() {
                column_count = Some(columns.len());
            } else if columns.len() != column_count.unwrap() {
                return Err(format!("Inconsistent column count at line {}", index + 1).into());
            }
            
            line_count += 1;
        }

        if line_count == 0 {
            return Err("Empty CSV file".into());
        }

        Ok(line_count)
    }

    pub fn transform_column(
        &self,
        input_path: &str,
        output_path: &str,
        column_index: usize,
        transform_fn: fn(&str) -> String,
    ) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(output_path)?;

        for (line_num, line) in reader.lines().enumerate() {
            let mut line = line?;
            
            if line_num == 0 && self.has_headers {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            let mut columns: Vec<String> = line
                .split(self.delimiter)
                .map(String::from)
                .collect();

            if column_index < columns.len() {
                columns[column_index] = transform_fn(&columns[column_index]);
                line = columns.join(&self.delimiter.to_string());
            }

            writeln!(output_file, "{}", line)?;
        }

        Ok(())
    }

    pub fn filter_rows(
        &self,
        input_path: &str,
        output_path: &str,
        filter_fn: fn(&[&str]) -> bool,
    ) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(output_path)?;
        let mut kept_rows = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 && self.has_headers {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            if filter_fn(&columns) {
                writeln!(output_file, "{}", line)?;
                kept_rows += 1;
            }
        }

        Ok(kept_rows)
    }
}

fn uppercase_transform(value: &str) -> String {
    value.to_uppercase()
}

fn numeric_filter(columns: &[&str]) -> bool {
    if columns.len() > 1 {
        columns[1].parse::<f64>().is_ok()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_validation() {
        let processor = CsvProcessor::new(',', true);
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,25,New York").unwrap();
        writeln!(temp_file, "Jane,30,London").unwrap();
        
        let result = processor.validate_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
    }

    #[test]
    fn test_column_transformation() {
        let processor = CsvProcessor::new(',', true);
        let mut input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        
        writeln!(input_file, "name,age,city").unwrap();
        writeln!(input_file, "john,25,new york").unwrap();
        writeln!(input_file, "jane,30,london").unwrap();
        
        processor.transform_column(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            0,
            uppercase_transform,
        ).unwrap();

        let mut content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        
        assert!(content.contains("JOHN"));
        assert!(content.contains("JANE"));
    }

    #[test]
    fn test_row_filtering() {
        let processor = CsvProcessor::new(',', true);
        let mut input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        
        writeln!(input_file, "name,age,city").unwrap();
        writeln!(input_file, "John,25,New York").unwrap();
        writeln!(input_file, "Jane,invalid,London").unwrap();
        writeln!(input_file, "Bob,35,Paris").unwrap();
        
        let kept = processor.filter_rows(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            numeric_filter,
        ).unwrap();
        
        assert_eq!(kept, 2);
    }
}