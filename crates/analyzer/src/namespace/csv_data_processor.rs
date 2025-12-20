use csv::{ReaderBuilder, WriterBuilder};
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

fn load_csv_data(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(file);
    
    let mut records = Vec::new();
    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    
    Ok(records)
}

fn filter_records(records: &[Record], category_filter: &str) -> Vec<Record> {
    records
        .iter()
        .filter(|r| r.category == category_filter && r.active)
        .cloned()
        .collect()
}

fn calculate_average_value(records: &[Record]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }
    
    let total: f64 = records.iter().map(|r| r.value).sum();
    total / records.len() as f64
}

fn save_filtered_results(records: &[Record], output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut writer = WriterBuilder::new().has_headers(true).from_writer(file);
    
    for record in records {
        writer.serialize(record)?;
    }
    
    writer.flush()?;
    Ok(())
}

fn process_data_pipeline(input_path: &str, output_path: &str, category: &str) -> Result<(), Box<dyn Error>> {
    let all_records = load_csv_data(input_path)?;
    let filtered_records = filter_records(&all_records, category);
    
    if !filtered_records.is_empty() {
        let avg_value = calculate_average_value(&filtered_records);
        println!("Processed {} records in category '{}'", filtered_records.len(), category);
        println!("Average value: {:.2}", avg_value);
        
        save_filtered_results(&filtered_records, output_path)?;
        println!("Results saved to: {}", output_path);
    } else {
        println!("No records found for category: {}", category);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_filter_records() {
        let records = vec![
            Record { id: 1, name: "Item A".to_string(), category: "Electronics".to_string(), value: 100.0, active: true },
            Record { id: 2, name: "Item B".to_string(), category: "Books".to_string(), value: 50.0, active: true },
            Record { id: 3, name: "Item C".to_string(), category: "Electronics".to_string(), value: 200.0, active: false },
        ];
        
        let filtered = filter_records(&records, "Electronics");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }
    
    #[test]
    fn test_calculate_average() {
        let records = vec![
            Record { id: 1, name: "Test".to_string(), category: "Test".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "Test".to_string(), category: "Test".to_string(), value: 20.0, active: true },
            Record { id: 3, name: "Test".to_string(), category: "Test".to_string(), value: 30.0, active: true },
        ];
        
        let avg = calculate_average_value(&records);
        assert_eq!(avg, 20.0);
    }
}