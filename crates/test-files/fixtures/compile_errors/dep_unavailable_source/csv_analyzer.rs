use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvSummary {
    pub row_count: usize,
    pub column_count: usize,
    pub headers: Vec<String>,
}

pub fn analyze_csv(file_path: &str) -> Result<CsvSummary, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let headers = match lines.next() {
        Some(header_line) => header_line?
            .split(',')
            .map(|s| s.trim().to_string())
            .collect(),
        None => return Err("Empty CSV file".into()),
    };

    let column_count = headers.len();
    let mut row_count = 0;

    for line_result in lines {
        let line = line_result?;
        if !line.trim().is_empty() {
            row_count += 1;
        }
    }

    Ok(CsvSummary {
        row_count,
        column_count,
        headers,
    })
}

pub fn display_summary(summary: &CsvSummary) {
    println!("CSV File Analysis Summary");
    println!("=========================");
    println!("Total Rows: {}", summary.row_count);
    println!("Total Columns: {}", summary.column_count);
    println!("Column Headers:");
    for (index, header) in summary.headers.iter().enumerate() {
        println!("  {}. {}", index + 1, header);
    }
}