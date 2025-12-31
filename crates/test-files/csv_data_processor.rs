
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
        self.records.iter().map(|record| record.value).sum()
    }

    fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            0.0
        } else {
            self.calculate_total_value() / self.records.len() as f64
        }
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    fn find_min_value(&self) -> Option<&Record> {
        self.records.iter().min_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    fn export_filtered(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let mut wtr = Writer::from_path(output_path)?;
        
        for record in filtered {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    fn generate_summary(&self) -> String {
        format!(
            "Total records: {}\nTotal value: {:.2}\nAverage value: {:.2}\nActive records: {}",
            self.records.len(),
            self.calculate_total_value(),
            self.calculate_average_value(),
            self.filter_active().len()
        )
    }
}

fn process_sample_data() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_file("data/input.csv")?;
    
    println!("Data Summary:");
    println!("{}", processor.generate_summary());
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Max value record: {:?}", max_record);
    }
    
    if let Some(min_record) = processor.find_min_value() {
        println!("Min value record: {:?}", min_record);
    }
    
    let electronics = processor.filter_by_category("Electronics");
    println!("Electronics records: {}", electronics.len());
    
    processor.export_filtered("Electronics", "data/electronics.csv")?;
    
    Ok(())
}

fn main() {
    match process_sample_data() {
        Ok(_) => println!("Processing completed successfully"),
        Err(e) => eprintln!("Error occurred: {}", e),
    }
}