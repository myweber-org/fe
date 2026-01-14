
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
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
struct ProcessedData {
    total_records: usize,
    average_value: f64,
    active_count: usize,
    category_summary: Vec<CategorySummary>,
}

#[derive(Debug)]
struct CategorySummary {
    name: String,
    count: usize,
    total_value: f64,
}

impl ProcessedData {
    fn new() -> Self {
        ProcessedData {
            total_records: 0,
            average_value: 0.0,
            active_count: 0,
            category_summary: Vec::new(),
        }
    }
}

fn read_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(path)?;
    let mut records = Vec::new();
    
    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    
    Ok(records)
}

fn filter_records(records: &[Record], category_filter: Option<&str>, active_only: bool) -> Vec<Record> {
    records.iter()
        .filter(|record| {
            if active_only && !record.active {
                return false;
            }
            if let Some(category) = category_filter {
                if record.category != category {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect()
}

fn process_data(records: &[Record]) -> ProcessedData {
    let mut result = ProcessedData::new();
    result.total_records = records.len();
    
    if records.is_empty() {
        return result;
    }
    
    let mut category_map = std::collections::HashMap::new();
    let mut total_value = 0.0;
    
    for record in records {
        total_value += record.value;
        
        if record.active {
            result.active_count += 1;
        }
        
        let entry = category_map.entry(record.category.clone())
            .or_insert((0, 0.0));
        entry.0 += 1;
        entry.1 += record.value;
    }
    
    result.average_value = total_value / records.len() as f64;
    
    for (category, (count, total)) in category_map {
        result.category_summary.push(CategorySummary {
            name: category,
            count,
            total_value: total,
        });
    }
    
    result
}

fn write_processed_data<P: AsRef<Path>>(data: &ProcessedData, path: P) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(path)?;
    
    writer.write_record(&["Metric", "Value"])?;
    writer.write_record(&["Total Records", &data.total_records.to_string()])?;
    writer.write_record(&["Average Value", &format!("{:.2}", data.average_value)])?;
    writer.write_record(&["Active Records", &data.active_count.to_string()])?;
    
    writer.write_record(&[])?;
    writer.write_record(&["Category", "Count", "Total Value"])?;
    
    for summary in &data.category_summary {
        writer.write_record(&[
            &summary.name,
            &summary.count.to_string(),
            &format!("{:.2}", summary.total_value)
        ])?;
    }
    
    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_path = "data/input.csv";
    let output_path = "data/processed_summary.csv";
    
    println!("Reading CSV data from: {}", input_path);
    let records = read_csv_file(input_path)?;
    
    println!("Original records: {}", records.len());
    
    let filtered_records = filter_records(&records, Some("Electronics"), true);
    println!("Filtered records (Electronics, active only): {}", filtered_records.len());
    
    let processed_data = process_data(&filtered_records);
    
    println!("Writing processed data to: {}", output_path);
    write_processed_data(&processed_data, output_path)?;
    
    println!("Processing completed successfully!");
    println!("Total processed records: {}", processed_data.total_records);
    println!("Average value: {:.2}", processed_data.average_value);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_filter_records() {
        let records = vec![
            Record { id: 1, name: "Item A".to_string(), category: "Electronics".to_string(), value: 100.0, active: true },
            Record { id: 2, name: "Item B".to_string(), category: "Books".to_string(), value: 50.0, active: true },
            Record { id: 3, name: "Item C".to_string(), category: "Electronics".to_string(), value: 75.0, active: false },
        ];
        
        let filtered = filter_records(&records, Some("Electronics"), true);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
        
        let filtered_all = filter_records(&records, None, false);
        assert_eq!(filtered_all.len(), 3);
    }
    
    #[test]
    fn test_process_data() {
        let records = vec![
            Record { id: 1, name: "Item A".to_string(), category: "Electronics".to_string(), value: 100.0, active: true },
            Record { id: 2, name: "Item B".to_string(), category: "Electronics".to_string(), value: 50.0, active: true },
        ];
        
        let processed = process_data(&records);
        assert_eq!(processed.total_records, 2);
        assert_eq!(processed.average_value, 75.0);
        assert_eq!(processed.active_count, 2);
        assert_eq!(processed.category_summary.len(), 1);
        assert_eq!(processed.category_summary[0].total_value, 150.0);
    }
}