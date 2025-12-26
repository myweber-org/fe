use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    pub delimiter: char,
    pub has_headers: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            has_headers: true,
        }
    }
}

pub fn parse_csv<P: AsRef<Path>>(
    path: P,
    config: &CsvConfig,
) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut lines = reader.lines().enumerate();

    if config.has_headers {
        if let Some((_, header_line)) = lines.next() {
            let headers = header_line?;
            println!("Headers: {}", headers);
        }
    }

    for (line_num, line_result) in lines {
        let line = line_result?;
        let fields: Vec<String> = line
            .split(config.delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        if fields.iter().any(|f| f.is_empty()) {
            return Err(format!("Empty field detected at line {}", line_num + 1).into());
        }

        records.push(fields);
    }

    if records.is_empty() {
        return Err("No data records found in CSV file".into());
    }

    Ok(records)
}

pub fn validate_record_lengths(records: &[Vec<String>]) -> Result<usize, Box<dyn Error>> {
    if records.is_empty() {
        return Err("Empty records provided".into());
    }

    let expected_len = records[0].len();
    for (idx, record) in records.iter().enumerate() {
        if record.len() != expected_len {
            return Err(format!(
                "Record {} has {} fields, expected {}",
                idx + 1,
                record.len(),
                expected_len
            )
            .into());
        }
    }

    Ok(expected_len)
}