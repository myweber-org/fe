
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

impl Record {
    fn from_csv_line(line: &str) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 5 {
            return Err("Invalid CSV format".into());
        }

        Ok(Record {
            id: parts[0].parse()?,
            name: parts[1].to_string(),
            category: parts[2].to_string(),
            value: parts[3].parse()?,
            active: parts[4].parse()?,
        })
    }
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

    fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines().skip(1) {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let record = Record::from_csv_line(&line)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category && r.active)
            .collect()
    }

    fn calculate_average(&self, category: &str) -> Option<f64> {
        let filtered: Vec<&Record> = self.filter_by_category(category);
        if filtered.is_empty() {
            return None;
        }

        let sum: f64 = filtered.iter().map(|r| r.value).sum();
        Some(sum / filtered.len() as f64)
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        
        groups
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    let sample_data = "1,ProductA,Electronics,299.99,true\n\
                       2,ProductB,Books,19.99,true\n\
                       3,ProductC,Electronics,599.99,false\n\
                       4,ProductD,Books,29.99,true\n\
                       5,ProductE,Electronics,399.99,true";
    
    let temp_file = "temp_sample.csv";
    std::fs::write(temp_file, sample_data)?;
    
    processor.load_from_file(temp_file)?;
    
    let electronics = processor.filter_by_category("Electronics");
    println!("Active Electronics: {}", electronics.len());
    
    if let Some(avg) = processor.calculate_average("Electronics") {
        println!("Average Electronics value: {:.2}", avg);
    }
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Max value record: {:?}", max_record);
    }
    
    let groups = processor.group_by_category();
    for (category, records) in groups {
        println!("Category {}: {} records", category, records.len());
    }
    
    std::fs::remove_file(temp_file)?;
    Ok(())
}

fn main() {
    if let Err(e) = process_data_sample() {
        eprintln!("Error processing data: {}", e);
        std::process::exit(1);
    }
}