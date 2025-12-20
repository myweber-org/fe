
use csv::Reader;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

#[derive(Debug)]
struct AggregatedData {
    category: String,
    total_value: f64,
    average_value: f64,
    record_count: usize,
    active_count: usize,
}

fn load_csv_data(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter().filter(|r| r.active).collect()
}

fn aggregate_by_category(records: &[Record]) -> HashMap<String, AggregatedData> {
    let mut category_map: HashMap<String, (f64, usize, usize)> = HashMap::new();

    for record in records {
        let entry = category_map.entry(record.category.clone()).or_insert((0.0, 0, 0));
        entry.0 += record.value;
        entry.1 += 1;
        if record.active {
            entry.2 += 1;
        }
    }

    category_map
        .into_iter()
        .map(|(category, (total, count, active_count))| {
            let aggregated = AggregatedData {
                category: category.clone(),
                total_value: total,
                average_value: total / count as f64,
                record_count: count,
                active_count,
            };
            (category, aggregated)
        })
        .collect()
}

fn process_data_file(file_path: &str) -> Result<(), Box<dyn Error>> {
    let records = load_csv_data(file_path)?;
    
    println!("Total records loaded: {}", records.len());
    
    let active_records = filter_active_records(&records);
    println!("Active records: {}", active_records.len());
    
    let aggregated_data = aggregate_by_category(&records);
    
    for (category, data) in &aggregated_data {
        println!(
            "Category: {} | Total: {:.2} | Avg: {:.2} | Records: {} | Active: {}",
            category,
            data.total_value,
            data.average_value,
            data.record_count,
            data.active_count
        );
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = "data/sample.csv";
    
    match process_data_file(file_path) {
        Ok(_) => println!("Data processing completed successfully"),
        Err(e) => eprintln!("Error processing data: {}", e),
    }
    
    Ok(())
}