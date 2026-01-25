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

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
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

    fn find_max_value(&self) -> Option<&Record> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }

    fn find_min_value(&self) -> Option<&Record> {
        self.records
            .iter()
            .min_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }

    fn save_filtered_results<P: AsRef<Path>>(
        &self,
        filtered_records: Vec<&Record>,
        output_path: P,
    ) -> Result<(), Box<dyn Error>> {
        let mut wtr = Writer::from_path(output_path)?;
        
        for record in filtered_records {
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

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_file("input_data.csv")?;
    
    println!("Data Summary:");
    println!("{}", processor.generate_summary());
    
    let electronics = processor.filter_by_category("Electronics");
    println!("Found {} electronics records", electronics.len());
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Highest value record: {:?}", max_record);
    }
    
    if let Some(min_record) = processor.find_min_value() {
        println!("Lowest value record: {:?}", min_record);
    }
    
    let active_records = processor.filter_active();
    processor.save_filtered_results(active_records, "active_records.csv")?;
    
    Ok(())
}