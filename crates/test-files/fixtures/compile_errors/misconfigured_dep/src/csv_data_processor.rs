use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct Record {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl Record {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid CSV format".into());
        }

        Ok(Record {
            id: parts[0].parse()?,
            category: parts[1].to_string(),
            value: parts[2].parse()?,
            active: parts[3].parse()?,
        })
    }
}

fn load_records(filename: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for line in reader.lines().skip(1) {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        records.push(Record::from_csv_line(&line)?);
    }

    Ok(records)
}

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter().filter(|r| r.active).collect()
}

fn calculate_category_totals(records: &[Record]) -> HashMap<String, f64> {
    let mut totals = HashMap::new();
    
    for record in records {
        if record.active {
            *totals.entry(record.category.clone()).or_insert(0.0) += record.value;
        }
    }
    
    totals
}

fn find_max_value_record(records: &[Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

fn process_data(filename: &str) -> Result<(), Box<dyn Error>> {
    let records = load_records(filename)?;
    
    println!("Total records loaded: {}", records.len());
    
    let active_records = filter_active_records(&records);
    println!("Active records: {}", active_records.len());
    
    let category_totals = calculate_category_totals(&records);
    for (category, total) in &category_totals {
        println!("Category '{}' total: {:.2}", category, total);
    }
    
    if let Some(max_record) = find_max_value_record(&records) {
        println!("Record with maximum value: ID={}, Category={}, Value={}", 
                 max_record.id, max_record.category, max_record.value);
    }
    
    Ok(())
}

fn main() {
    let filename = "data.csv";
    
    match process_data(filename) {
        Ok(()) => println!("Data processing completed successfully"),
        Err(e) => eprintln!("Error processing data: {}", e),
    }
}