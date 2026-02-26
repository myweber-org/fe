use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

#[derive(Debug)]
struct CsvStats {
    row_count: usize,
    column_count: usize,
    column_types: HashMap<String, String>,
    numeric_columns: HashMap<String, (f64, f64, f64)>,
}

impl CsvStats {
    fn new() -> Self {
        CsvStats {
            row_count: 0,
            column_count: 0,
            column_types: HashMap::new(),
            numeric_columns: HashMap::new(),
        }
    }

    fn analyze_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut stats = CsvStats::new();
        let mut headers: Vec<String> = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                headers = line.split(',').map(|s| s.trim().to_string()).collect();
                stats.column_count = headers.len();
                for header in &headers {
                    stats.column_types.insert(header.clone(), "unknown".to_string());
                }
                continue;
            }

            stats.row_count += 1;
            let values: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

            for (i, value) in values.iter().enumerate() {
                if i >= headers.len() {
                    continue;
                }

                let header = &headers[i];
                
                if let Ok(num) = value.parse::<f64>() {
                    stats.column_types.insert(header.clone(), "numeric".to_string());
                    
                    let entry = stats.numeric_columns
                        .entry(header.clone())
                        .or_insert((num, num, 0.0));
                    
                    entry.0 = entry.0.min(num);
                    entry.1 = entry.1.max(num);
                    entry.2 += num;
                } else if !value.is_empty() {
                    stats.column_types.insert(header.clone(), "text".to_string());
                }
            }
        }

        Ok(stats)
    }

    fn display_summary(&self) {
        println!("CSV Analysis Summary:");
        println!("Rows: {}", self.row_count);
        println!("Columns: {}", self.column_count);
        println!("\nColumn Types:");
        
        for (column, data_type) in &self.column_types {
            print!("{}: {}", column, data_type);
            
            if let Some(stats) = self.numeric_columns.get(column) {
                let avg = stats.2 / self.row_count as f64;
                println!(" (min: {:.2}, max: {:.2}, avg: {:.2})", stats.0, stats.1, avg);
            } else {
                println!();
            }
        }
    }
}

fn validate_csv_format(path: &str) -> Result<bool, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut line_lengths: Vec<usize> = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let fields: Vec<&str> = line.split(',').collect();
        line_lengths.push(fields.len());
    }

    if line_lengths.is_empty() {
        return Ok(false);
    }

    let first_length = line_lengths[0];
    let consistent = line_lengths.iter().all(|&len| len == first_length);
    
    Ok(consistent)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <csv_file>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    
    match validate_csv_format(filename) {
        Ok(true) => println!("CSV format is consistent"),
        Ok(false) => println!("Warning: CSV format is inconsistent"),
        Err(e) => eprintln!("Validation error: {}", e),
    }

    let stats = CsvStats::analyze_file(filename)?;
    stats.display_summary();

    Ok(())
}