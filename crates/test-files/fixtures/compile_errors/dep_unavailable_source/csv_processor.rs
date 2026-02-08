use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

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
        let name = parts[1].trim().to_string();
        let value = parts[2].parse()?;
        let active = parts[3].parse()?;

        Ok(Record {
            id,
            name,
            value,
            active,
        })
    }
}

fn process_csv_file(path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        match Record::from_csv_line(&line) {
            Ok(record) => records.push(record),
            Err(e) => eprintln!("Error parsing line {}: {}", line_num + 1, e),
        }
    }

    Ok(records)
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let active_records: Vec<&Record> = records.iter().filter(|r| r.active).collect();
    
    if active_records.is_empty() {
        return (0.0, 0.0, 0);
    }

    let sum: f64 = active_records.iter().map(|r| r.value).sum();
    let avg = sum / active_records.len() as f64;
    let max = active_records.iter().map(|r| r.value).fold(0.0, f64::max);

    (avg, max, active_records.len())
}

fn main() -> Result<(), Box<dyn Error>> {
    let records = process_csv_file("data.csv")?;
    
    println!("Total records loaded: {}", records.len());
    
    let (average, maximum, active_count) = calculate_statistics(&records);
    println!("Active records: {}", active_count);
    println!("Average value: {:.2}", average);
    println!("Maximum value: {:.2}", maximum);

    Ok(())
}