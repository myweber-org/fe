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

fn load_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(path)?;
    let mut records = Vec::new();
    
    for result in reader.deserialize() {
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

fn aggregate_by_category(records: &[Record]) -> Vec<(String, f64, usize)> {
    use std::collections::HashMap;
    
    let mut aggregates: HashMap<String, (f64, usize)> = HashMap::new();
    
    for record in records {
        let entry = aggregates.entry(record.category.clone())
            .or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }
    
    aggregates.into_iter()
        .map(|(category, (total, count))| (category, total, count))
        .collect()
}

fn save_results<P: AsRef<Path>>(results: &[(String, f64, usize)], path: P) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(path)?;
    
    writer.write_record(&["Category", "Total Value", "Record Count"])?;
    
    for (category, total, count) in results {
        writer.write_record(&[
            category,
            &format!("{:.2}", total),
            &count.to_string()
        ])?;
    }
    
    writer.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = load_csv(input_path)?;
    
    println!("Loaded {} total records", records.len());
    
    let active_records = filter_active_records(&records);
    println!("Found {} active records", active_records.len());
    
    let aggregates = aggregate_by_category(&active_records);
    
    for (category, total, count) in &aggregates {
        println!("Category '{}': {} records, total value: {:.2}", 
                category, count, total);
    }
    
    save_results(&aggregates, output_path)?;
    println!("Results saved to {}", output_path);
    
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
    
    #[test]
    fn test_filter_active_records() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), category: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "Test2".to_string(), category: "B".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "Test3".to_string(), category: "A".to_string(), value: 30.0, active: true },
        ];
        
        let active = filter_active_records(&records);
        assert_eq!(active.len(), 2);
        assert!(active.iter().all(|r| r.active));
    }
    
    #[test]
    fn test_aggregate_by_category() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), category: "A".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "Test2".to_string(), category: "A".to_string(), value: 20.0, active: true },
            Record { id: 3, name: "Test3".to_string(), category: "B".to_string(), value: 30.0, active: true },
        ];
        
        let aggregates = aggregate_by_category(&records);
        assert_eq!(aggregates.len(), 2);
        
        let category_a = aggregates.iter().find(|(cat, _, _)| cat == "A").unwrap();
        assert_eq!(category_a.1, 30.0);
        assert_eq!(category_a.2, 2);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub fn load_csv_records(file_path: &str) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        if index == 0 {
            continue;
        }

        let line = line?;
        let fields: Vec<&str> = line.split(',').collect();

        if fields.len() >= 4 {
            let id = fields[0].parse::<u32>()?;
            let name = fields[1].to_string();
            let value = fields[2].parse::<f64>()?;
            let category = fields[3].to_string();

            records.push(CsvRecord {
                id,
                name,
                value,
                category,
            });
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[CsvRecord], category: &str) -> Vec<&CsvRecord> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_average_value(records: &[CsvRecord]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|record| record.value).sum();
    Some(sum / records.len() as f64)
}

pub fn find_max_value_record(records: &[CsvRecord]) -> Option<&CsvRecord> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let csv_data = "id,name,value,category\n\
                        1,ItemA,25.5,Electronics\n\
                        2,ItemB,42.8,Books\n\
                        3,ItemC,18.3,Electronics\n\
                        4,ItemD,99.9,Books";

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();

        let records = load_csv_records(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 4);

        let electronics = filter_by_category(&records, "Electronics");
        assert_eq!(electronics.len(), 2);

        let avg = calculate_average_value(&records).unwrap();
        assert!(avg > 0.0);

        let max_record = find_max_value_record(&records).unwrap();
        assert_eq!(max_record.id, 4);
    }
}