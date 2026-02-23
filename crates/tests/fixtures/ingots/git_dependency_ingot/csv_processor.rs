
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

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

    pub fn validate_file<P: AsRef<Path>>(&self, file_path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut line_count = 0;
        let mut column_count: Option<usize> = None;

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            if let Some(expected) = column_count {
                if columns.len() != expected {
                    return Err(format!("Line {} has {} columns, expected {}", 
                        index + 1, columns.len(), expected).into());
                }
            } else {
                column_count = Some(columns.len());
            }
            
            line_count += 1;
        }

        if line_count == 0 {
            return Err("File is empty".into());
        }

        Ok(line_count)
    }

    pub fn transform_column<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
        column_index: usize,
        transformer: fn(&str) -> String,
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
                columns[column_index] = transformer(&columns[column_index]);
            }

            let transformed_line = columns.join(&self.delimiter.to_string());
            writeln!(output_file, "{}", transformed_line)?;
        }

        Ok(())
    }

    pub fn filter_rows<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
        predicate: fn(&[String]) -> bool,
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

            let columns: Vec<String> = line
                .split(self.delimiter)
                .map(String::from)
                .collect();

            if predicate(&columns) {
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

fn numeric_filter(columns: &[String]) -> bool {
    if let Some(first_col) = columns.get(0) {
        first_col.parse::<f64>().is_ok()
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
        let processor = CsvProcessor::new(',', false);
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "a,b,c").unwrap();
        writeln!(temp_file, "1,2,3").unwrap();
        
        let result = processor.validate_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_column_transformation() {
        let processor = CsvProcessor::new(',', true);
        let mut input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        
        writeln!(input_file, "name,value").unwrap();
        writeln!(input_file, "test,123").unwrap();
        
        processor.transform_column(
            input_file.path(),
            output_file.path(),
            0,
            uppercase_transform,
        ).unwrap();

        let mut content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        
        assert!(content.contains("TEST,123"));
    }

    #[test]
    fn test_row_filtering() {
        let processor = CsvProcessor::new(',', false);
        let mut input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();
        
        writeln!(input_file, "abc,def").unwrap();
        writeln!(input_file, "123,456").unwrap();
        writeln!(input_file, "xyz,789").unwrap();
        
        let kept = processor.filter_rows(
            input_file.path(),
            output_file.path(),
            numeric_filter,
        ).unwrap();
        
        assert_eq!(kept, 1);
    }
}