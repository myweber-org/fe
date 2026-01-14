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

    pub fn read_and_validate(&self, file_path: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line?;
            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.is_empty() {
                return Err(format!("Empty line found at line {}", line_number).into());
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err("CSV file is empty".into());
        }

        Ok(records)
    }

    pub fn transform_numeric_fields(
        &self,
        data: &[Vec<String>],
        column_index: usize,
        multiplier: f64,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let mut transformed = Vec::new();
        let start_index = if self.has_headers { 1 } else { 0 };

        for (row_index, row) in data.iter().enumerate() {
            if row_index == 0 && self.has_headers {
                transformed.push(row.clone());
                continue;
            }

            if column_index >= row.len() {
                return Err(format!(
                    "Column index {} out of bounds for row {}",
                    column_index, row_index
                )
                .into());
            }

            let mut new_row = row.clone();
            match row[column_index].parse::<f64>() {
                Ok(value) => {
                    let transformed_value = value * multiplier;
                    new_row[column_index] = transformed_value.to_string();
                }
                Err(_) => {
                    return Err(format!(
                        "Invalid numeric value at row {}, column {}: '{}'",
                        row_index, column_index, row[column_index]
                    )
                    .into());
                }
            }
            transformed.push(new_row);
        }

        Ok(transformed)
    }

    pub fn write_to_file(&self, data: &[Vec<String>], output_path: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(output_path)?;
        
        for row in data {
            let line = row.join(&self.delimiter.to_string());
            writeln!(file, "{}", line)?;
        }
        
        Ok(())
    }

    pub fn calculate_column_summary(&self, data: &[Vec<String>], column_index: usize) -> Result<(f64, f64, f64), Box<dyn Error>> {
        let start_index = if self.has_headers { 1 } else { 0 };
        let mut values = Vec::new();
        let mut sum = 0.0;

        for (row_index, row) in data.iter().enumerate().skip(start_index) {
            if column_index >= row.len() {
                return Err(format!("Column index {} out of bounds for row {}", column_index, row_index).into());
            }

            match row[column_index].parse::<f64>() {
                Ok(value) => {
                    values.push(value);
                    sum += value;
                }
                Err(_) => {
                    return Err(format!("Invalid numeric value at row {}, column {}: '{}'", 
                        row_index, column_index, row[column_index]).into());
                }
            }
        }

        if values.is_empty() {
            return Err("No valid numeric values found in specified column".into());
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let min = values[0];
        let max = values[values.len() - 1];
        let average = sum / values.len() as f64;

        Ok((min, max, average))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,60000").unwrap();

        let processor = CsvProcessor::new(',', true);
        let data = processor.read_and_validate(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(data.len(), 3);
        assert_eq!(data[0], vec!["name", "age", "salary"]);
        
        let transformed = processor.transform_numeric_fields(&data, 2, 1.1).unwrap();
        assert_eq!(transformed[1][2], "55000");
        
        let summary = processor.calculate_column_summary(&data, 1).unwrap();
        assert_eq!(summary.0, 25.0);
        assert_eq!(summary.1, 35.0);
    }
}