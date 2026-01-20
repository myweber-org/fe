use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

impl Record {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid number of fields".into());
        }

        let id = parts[0].parse()?;
        let name = parts[1].to_string();
        let value = parts[2].parse()?;
        let active = parts[3].parse()?;

        Ok(Record {
            id,
            name,
            value,
            active,
        })
    }

    fn to_csv_line(&self) -> String {
        format!("{},{},{},{}", self.id, self.name, self.value, self.active)
    }
}

fn read_records_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match Record::from_csv_line(&line) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Warning: Skipping line {}: {}", line_num + 1, e),
        }
    }

    Ok(records)
}

fn filter_records(records: &[Record], min_value: f64) -> Vec<&Record> {
    records
        .iter()
        .filter(|r| r.value >= min_value && r.active)
        .collect()
}

fn write_records_to_file<P: AsRef<Path>>(
    records: &[&Record],
    path: P,
) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(path)?;
    for record in records {
        writeln!(file, "{}", record.to_csv_line())?;
    }
    Ok(())
}

fn process_csv_file(input_path: &str, output_path: &str, threshold: f64) -> Result<(), Box<dyn Error>> {
    println!("Processing CSV file: {}", input_path);
    
    let records = read_records_from_file(input_path)?;
    println!("Loaded {} records", records.len());
    
    let filtered = filter_records(&records, threshold);
    println!("Found {} records meeting criteria (value >= {}, active=true)", 
             filtered.len(), threshold);
    
    write_records_to_file(&filtered, output_path)?;
    println!("Results written to: {}", output_path);
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/filtered.csv";
    let threshold = 50.0;
    
    process_csv_file(input_file, output_file, threshold)
}