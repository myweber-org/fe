use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    file_path: String,
    delimiter: char,
}

impl CsvProcessor {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        CsvProcessor {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn read_and_aggregate(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut aggregated_data = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let fields: Vec<String> = line.split(self.delimiter).map(|s| s.to_string()).collect();
            aggregated_data.push(fields);
        }

        Ok(aggregated_data)
    }

    pub fn filter_by_column(&self, column_index: usize, filter_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let all_data = self.read_and_aggregate()?;
        let filtered: Vec<Vec<String>> = all_data
            .into_iter()
            .filter(|row| {
                if let Some(value) = row.get(column_index) {
                    value == filter_value
                } else {
                    false
                }
            })
            .collect();

        Ok(filtered)
    }
}

pub fn calculate_column_sum(data: &[Vec<String>], column_index: usize) -> Result<f64, Box<dyn Error>> {
    let mut sum = 0.0;
    for row in data {
        if let Some(value_str) = row.get(column_index) {
            let value: f64 = value_str.parse()?;
            sum += value;
        }
    }
    Ok(sum)
}