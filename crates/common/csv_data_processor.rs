
use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    fn new(id: u32, category: &str, value: f64, active: bool) -> Self {
        Self {
            id,
            category: category.to_string(),
            value,
            active,
        }
    }
}

struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category && r.active)
            .cloned()
            .collect()
    }

    fn calculate_average(&self, category: &str) -> Option<f64> {
        let filtered = self.filter_by_category(category);
        
        if filtered.is_empty() {
            return None;
        }
        
        let sum: f64 = filtered.iter().map(|r| r.value).sum();
        Some(sum / filtered.len() as f64)
    }

    fn save_filtered_to_csv(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        
        let mut wtr = Writer::from_path(output_path)?;
        
        for record in filtered {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        Ok(())
    }

    fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    fn get_statistics(&self) -> (usize, f64, f64) {
        let count = self.records.len();
        let min = self.records.iter().map(|r| r.value).fold(f64::INFINITY, f64::min);
        let max = self.records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max);
        
        (count, min, max)
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.add_record(DataRecord::new(1, "A", 10.5, true));
    processor.add_record(DataRecord::new(2, "B", 20.3, true));
    processor.add_record(DataRecord::new(3, "A", 15.7, false));
    processor.add_record(DataRecord::new(4, "A", 12.8, true));
    processor.add_record(DataRecord::new(5, "C", 30.1, true));
    
    let category_a_avg = processor.calculate_average("A");
    println!("Average for category A: {:?}", category_a_avg);
    
    let stats = processor.get_statistics();
    println!("Statistics - Count: {}, Min: {}, Max: {}", stats.0, stats.1, stats.2);
    
    let filtered = processor.filter_by_category("A");
    println!("Filtered records for category A: {}", filtered.len());
    
    Ok(())
}