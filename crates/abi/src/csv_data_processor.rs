
use csv::{Reader, Writer};
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

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    fn calculate_total_value(&self) -> f64 {
        self.records
            .iter()
            .map(|record| record.value)
            .sum()
    }

    fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.calculate_total_value() / self.records.len() as f64
    }

    fn get_top_records(&self, limit: usize) -> Vec<&Record> {
        let mut sorted_records: Vec<&Record> = self.records.iter().collect();
        sorted_records.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
        sorted_records.into_iter().take(limit).collect()
    }

    fn save_filtered_results(&self, file_path: &str, category: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let mut wtr = Writer::from_path(file_path)?;

        for record in filtered {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    fn generate_summary(&self) -> String {
        format!(
            "Total records: {}\nActive records: {}\nTotal value: {:.2}\nAverage value: {:.2}",
            self.records.len(),
            self.filter_active().len(),
            self.calculate_total_value(),
            self.calculate_average_value()
        )
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_file("data/input.csv")?;
    
    println!("Data Summary:");
    println!("{}", processor.generate_summary());
    
    let electronics = processor.filter_by_category("Electronics");
    println!("Electronics records: {}", electronics.len());
    
    let top_5 = processor.get_top_records(5);
    println!("Top 5 records by value:");
    for record in top_5 {
        println!("  {}: {} - ${:.2}", record.id, record.name, record.value);
    }
    
    processor.save_filtered_results("data/electronics.csv", "Electronics")?;
    
    Ok(())
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        if index == 0 {
            continue;
        }

        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() == 4 {
            let record = Record {
                id: parts[0].parse()?,
                name: parts[1].to_string(),
                value: parts[2].parse()?,
                category: parts[3].to_string(),
            };
            records.push(record);
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_average(records: &[&Record]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|record| record.value).sum();
    Some(sum / records.len() as f64)
}

pub fn find_max_value(records: &[&Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

pub fn process_data(file_path: &str, target_category: &str) -> Result<(), Box<dyn Error>> {
    let all_records = load_csv(file_path)?;
    let filtered = filter_by_category(&all_records, target_category);
    
    if let Some(avg) = calculate_average(&filtered) {
        println!("Average value for category '{}': {:.2}", target_category, avg);
    }
    
    if let Some(max_record) = find_max_value(&filtered) {
        println!("Max value record: ID={}, Name={}, Value={}", 
                 max_record.id, max_record.name, max_record.value);
    }
    
    println!("Total records in category: {}", filtered.len());
    Ok(())
}