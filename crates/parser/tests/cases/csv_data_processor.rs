
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
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
}

fn read_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
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

fn aggregate_by_category(records: &[Record]) -> Vec<AggregatedData> {
    use std::collections::HashMap;

    let mut category_map: HashMap<String, (f64, usize)> = HashMap::new();

    for record in records {
        let entry = category_map.entry(record.category.clone()).or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }

    category_map
        .into_iter()
        .map(|(category, (total, count))| AggregatedData {
            category,
            total_value: total,
            average_value: total / count as f64,
            record_count: count,
        })
        .collect()
}

fn write_aggregated_csv<P: AsRef<Path>>(
    aggregated_data: &[AggregatedData],
    path: P,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut writer = Writer::from_writer(file);

    for data in aggregated_data {
        writer.serialize(data)?;
    }

    writer.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = read_csv_file(input_path)?;
    let active_records = filter_active_records(&records);
    let aggregated = aggregate_by_category(&active_records);
    write_aggregated_csv(&aggregated, output_path)?;

    println!("Processed {} records", records.len());
    println!("Found {} active records", active_records.len());
    println!("Generated {} aggregated categories", aggregated.len());

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";

    match process_csv_data(input_file, output_file) {
        Ok(_) => println!("CSV processing completed successfully"),
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }

    Ok(())
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
    
    let mut records = Vec::new();
    for result in rdr.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    
    Ok(records)
}

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter()
        .filter(|r| r.active)
        .collect()
}

fn calculate_category_totals(records: &[Record]) -> Vec<(String, f64)> {
    use std::collections::HashMap;
    
    let mut totals: HashMap<String, f64> = HashMap::new();
    
    for record in records {
        *totals.entry(record.category.clone()).or_insert(0.0) += record.value;
    }
    
    let mut result: Vec<(String, f64)> = totals.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

fn save_results_to_csv(results: &[(String, f64)], output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(file);
    
    wtr.write_record(&["Category", "Total"])?;
    
    for (category, total) in results {
        wtr.write_record(&[category, &total.to_string()])?;
    }
    
    wtr.flush()?;
    Ok(())
}

fn process_csv_data(input_file: &str, output_file: &str) -> Result<(), Box<dyn Error>> {
    let records = load_csv(input_file)?;
    
    println!("Loaded {} total records", records.len());
    
    let active_records = filter_active_records(&records);
    println!("Found {} active records", active_records.len());
    
    let category_totals = calculate_category_totals(&active_records);
    
    for (category, total) in &category_totals {
        println!("Category '{}': total = {:.2}", category, total);
    }
    
    save_results_to_csv(&category_totals, output_file)?;
    println!("Results saved to {}", output_file);
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    
    match process_csv_data(input_file, output_file) {
        Ok(_) => println!("Processing completed successfully"),
        Err(e) => eprintln!("Error processing CSV data: {}", e),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_filter_active_records() {
        let records = vec![
            Record { id: 1, name: "Item A".to_string(), category: "Electronics".to_string(), value: 100.0, active: true },
            Record { id: 2, name: "Item B".to_string(), category: "Books".to_string(), value: 50.0, active: false },
            Record { id: 3, name: "Item C".to_string(), category: "Electronics".to_string(), value: 75.0, active: true },
        ];
        
        let active = filter_active_records(&records);
        assert_eq!(active.len(), 2);
        assert!(active.iter().all(|r| r.active));
    }
    
    #[test]
    fn test_calculate_category_totals() {
        let records = vec![
            Record { id: 1, name: "Item A".to_string(), category: "Electronics".to_string(), value: 100.0, active: true },
            Record { id: 2, name: "Item B".to_string(), category: "Books".to_string(), value: 50.0, active: true },
            Record { id: 3, name: "Item C".to_string(), category: "Electronics".to_string(), value: 75.0, active: true },
        ];
        
        let totals = calculate_category_totals(&records);
        assert_eq!(totals.len(), 2);
        
        let electronics_total: f64 = totals.iter()
            .find(|(cat, _)| cat == "Electronics")
            .map(|(_, total)| *total)
            .unwrap_or(0.0);
        
        assert_eq!(electronics_total, 175.0);
    }
}