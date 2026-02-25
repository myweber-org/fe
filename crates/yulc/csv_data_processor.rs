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
        DataProcessor { records: Vec::new() }
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

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    fn save_filtered_results(&self, file_path: &str, records: Vec<&Record>) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let mut wtr = Writer::from_writer(file);
        
        for record in records {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    fn aggregate_by_category(&self) -> Vec<(String, f64)> {
        use std::collections::HashMap;
        
        let mut aggregates: HashMap<String, (f64, usize)> = HashMap::new();
        
        for record in &self.records {
            let entry = aggregates.entry(record.category.clone()).or_insert((0.0, 0));
            entry.0 += record.value;
            entry.1 += 1;
        }
        
        aggregates
            .into_iter()
            .map(|(category, (sum, count))| (category, sum / count as f64))
            .collect()
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_file("input.csv")?;
    
    println!("Total records: {}", processor.records.len());
    println!("Average value: {:.2}", processor.calculate_average());
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Max value record: {:?}", max_record);
    }
    
    let electronics = processor.filter_by_category("Electronics");
    println!("Electronics records: {}", electronics.len());
    
    let active_records = processor.filter_active();
    println!("Active records: {}", active_records.len());
    
    processor.save_filtered_results("electronics.csv", electronics)?;
    
    let category_avg = processor.aggregate_by_category();
    println!("Category averages:");
    for (category, avg) in category_avg {
        println!("  {}: {:.2}", category, avg);
    }
    
    Ok(())
}

fn main() {
    if let Err(e) = process_data_sample() {
        eprintln!("Error processing data: {}", e);
        std::process::exit(1);
    }
}