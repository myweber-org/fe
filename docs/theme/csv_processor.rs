use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs::File;

pub fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(file);

    let output_file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    let headers = rdr.headers()?.clone();
    wtr.write_record(&headers)?;

    for result in rdr.records() {
        let record = result?;
        let cleaned_record: Vec<String> = record
            .iter()
            .map(|field| {
                let trimmed = field.trim();
                if trimmed.is_empty() {
                    "N/A".to_string()
                } else {
                    trimmed.to_string()
                }
            })
            .collect();

        wtr.write_record(&cleaned_record)?;
    }

    wtr.flush()?;
    Ok(())
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvFilter {
    delimiter: char,
    has_header: bool,
}

impl CsvFilter {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvFilter {
            delimiter,
            has_header,
        }
    }

    pub fn filter_rows<P>(
        &self,
        file_path: P,
        predicate: impl Fn(&[String]) -> bool,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>>
    where
        P: AsRef<std::path::Path>,
    {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        let mut filtered_rows = Vec::new();

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if predicate(&fields) {
                filtered_rows.push(fields);
            }
        }

        Ok(filtered_rows)
    }

    pub fn extract_column(&self, rows: &[Vec<String>], column_index: usize) -> Vec<String> {
        rows.iter()
            .filter_map(|row| row.get(column_index).cloned())
            .collect()
    }
}

pub fn calculate_average(values: &[String]) -> Option<f64> {
    let mut sum = 0.0;
    let mut count = 0;

    for value in values {
        if let Ok(num) = value.parse::<f64>() {
            sum += num;
            count += 1;
        }
    }

    if count > 0 {
        Some(sum / count as f64)
    } else {
        None
    }
}