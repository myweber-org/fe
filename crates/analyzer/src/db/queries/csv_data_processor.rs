use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub struct CsvProcessor {
    records: Vec<Record>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 4 {
                let record = Record {
                    id: parts[0].parse()?,
                    name: parts[1].to_string(),
                    value: parts[2].parse()?,
                    category: parts[3].to_string(),
                };
                self.records.push(record);
            }
        }
        
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn get_records(&self) -> &Vec<Record> {
        &self.records
    }

    pub fn add_record(&mut self, record: Record) {
        self.records.push(record);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_processor() {
        let processor = CsvProcessor::new();
        assert_eq!(processor.get_records().len(), 0);
        assert_eq!(processor.calculate_average(), 0.0);
    }

    #[test]
    fn test_record_operations() {
        let mut processor = CsvProcessor::new();
        
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };
        
        processor.add_record(record);
        assert_eq!(processor.get_records().len(), 1);
        assert_eq!(processor.calculate_average(), 100.0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    fn new(id: u32, category: String, value: f64, active: bool) -> Self {
        Self {
            id,
            category,
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
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        for result in csv_reader.records() {
            let record = result?;
            let id: u32 = record[0].parse()?;
            let category = record[1].to_string();
            let value: f64 = record[2].parse()?;
            let active: bool = record[3].parse()?;

            self.records.push(DataRecord::new(id, category, value, active));
        }

        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn filter_active(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    fn find_max_value(&self) -> Option<&DataRecord> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }

    fn save_filtered_to_csv(&self, file_path: &str, category: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new().from_writer(writer);

        csv_writer.write_record(&["id", "category", "value", "active"])?;

        for record in filtered {
            csv_writer.write_record(&[
                record.id.to_string(),
                record.category.clone(),
                record.value.to_string(),
                record.active.to_string(),
            ])?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    fn remove_record_by_id(&mut self, id: u32) -> bool {
        let original_len = self.records.len();
        self.records.retain(|record| record.id != id);
        self.records.len() < original_len
    }
}

fn process_sample_data() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.add_record(DataRecord::new(1, "A".to_string(), 100.5, true));
    processor.add_record(DataRecord::new(2, "B".to_string(), 200.3, false));
    processor.add_record(DataRecord::new(3, "A".to_string(), 150.7, true));
    processor.add_record(DataRecord::new(4, "C".to_string(), 300.9, true));

    println!("Total records: {}", processor.records.len());
    println!("Average value: {:.2}", processor.calculate_average());

    let category_a_records = processor.filter_by_category("A");
    println!("Category A records: {}", category_a_records.len());

    if let Some(max_record) = processor.find_max_value() {
        println!("Max value record: ID {}, Value {}", max_record.id, max_record.value);
    }

    let active_records = processor.filter_active();
    println!("Active records: {}", active_records.len());

    processor.save_filtered_to_csv("filtered_a.csv", "A")?;

    let removed = processor.remove_record_by_id(2);
    println!("Record with ID 2 removed: {}", removed);
    println!("Remaining records: {}", processor.records.len());

    Ok(())
}

fn main() {
    if let Err(e) = process_sample_data() {
        eprintln!("Error processing data: {}", e);
    }
}