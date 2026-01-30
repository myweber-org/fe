use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct Record {
    category: String,
    value: f64,
}

fn process_csv_file(file_path: &str) -> Result<HashMap<String, f64>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut aggregates = HashMap::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 2 {
            continue;
        }

        let category = parts[0].trim().to_string();
        let value: f64 = match parts[1].trim().parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        let entry = aggregates.entry(category).or_insert(0.0);
        *entry += value;
    }

    Ok(aggregates)
}

fn main() {
    let file_path = "data.csv";
    
    match process_csv_file(file_path) {
        Ok(results) => {
            for (category, total) in results {
                println!("{}: {:.2}", category, total);
            }
        }
        Err(e) => {
            eprintln!("Error processing file: {}", e);
        }
    }
}