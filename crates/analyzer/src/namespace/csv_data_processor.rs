use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn read_and_validate<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
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
                return Err(format!("Empty line at {}", line_number).into());
            }

            if self.has_header && line_number == 1 {
                continue;
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err("No valid data records found".into());
        }

        Ok(records)
    }

    pub fn transform_data(
        &self,
        data: Vec<Vec<String>>,
        transform_fn: impl Fn(&str) -> String,
    ) -> Vec<Vec<String>> {
        data.into_iter()
            .map(|record| {
                record
                    .into_iter()
                    .map(|field| transform_fn(&field))
                    .collect()
            })
            .collect()
    }

    pub fn write_to_file<P: AsRef<Path>>(
        &self,
        data: Vec<Vec<String>>,
        output_path: P,
    ) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(output_path)?;
        
        for record in data {
            let line: String = record
                .iter()
                .map(|field| field.replace(self.delimiter, " "))
                .collect::<Vec<String>>()
                .join(&self.delimiter.to_string());
            
            writeln!(file, "{}", line)?;
        }
        
        Ok(())
    }

    pub fn calculate_statistics(&self, data: &[Vec<String>], column_index: usize) -> Result<(f64, f64, f64), Box<dyn Error>> {
        let mut values = Vec::new();
        
        for record in data {
            if column_index >= record.len() {
                return Err(format!("Column index {} out of bounds", column_index).into());
            }
            
            if let Ok(value) = record[column_index].parse::<f64>() {
                values.push(value);
            }
        }
        
        if values.is_empty() {
            return Err("No numeric values found in specified column".into());
        }
        
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        Ok((mean, variance, std_dev))
    }
}

pub fn create_sample_csv() -> Result<(), Box<dyn Error>> {
    let mut file = File::create("sample_data.csv")?;
    writeln!(file, "id,name,value,score")?;
    writeln!(file, "1,Alice,42.5,85.3")?;
    writeln!(file, "2,Bob,38.2,92.1")?;
    writeln!(file, "3,Charlie,45.8,78.4")?;
    writeln!(file, "4,Diana,41.3,88.9")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_csv_processing() {
        create_sample_csv().unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let data = processor.read_and_validate("sample_data.csv").unwrap();
        
        assert_eq!(data.len(), 4);
        assert_eq!(data[0].len(), 4);
        
        let transformed = processor.transform_data(data, |s| s.to_uppercase());
        assert!(transformed[0][1].contains("ALICE"));
        
        let stats = processor.calculate_statistics(&transformed, 3).unwrap();
        assert!(stats.0 > 0.0);
        
        fs::remove_file("sample_data.csv").unwrap();
    }
}