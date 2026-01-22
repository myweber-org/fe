
use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    pub fn new(id: u32, category: String, value: f64, active: bool) -> Self {
        Self {
            id,
            category,
            value,
            active,
        }
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn get_active_records(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn export_to_csv(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let mut wtr = Writer::from_writer(file);

        for record in &self.records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    pub fn total_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        processor.add_record(DataRecord::new(1, "A".to_string(), 10.5, true));
        processor.add_record(DataRecord::new(2, "B".to_string(), 20.0, false));
        processor.add_record(DataRecord::new(3, "A".to_string(), 30.5, true));
        
        assert_eq!(processor.total_records(), 3);
        assert_eq!(processor.filter_by_category("A").len(), 2);
        assert_eq!(processor.get_active_records().len(), 2);
        
        let avg = processor.calculate_average();
        assert!((avg - 20.333).abs() < 0.001);
    }
}
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

    fn save_filtered_results(&self, file_path: &str, records: Vec<&Record>) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let mut wtr = Writer::from_writer(file);
        
        for record in records {
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

fn process_data_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_file(input_path)?;
    
    println!("{}", processor.generate_summary());
    
    let active_records = processor.filter_active();
    processor.save_filtered_results(output_path, active_records)?;
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Record with maximum value: {:?}", max_record);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,category,value,active").unwrap();
        writeln!(file, "1,ItemA,Electronics,100.5,true").unwrap();
        writeln!(file, "2,ItemB,Books,45.0,false").unwrap();
        writeln!(file, "3,ItemC,Electronics,200.0,true").unwrap();
        writeln!(file, "4,ItemD,Clothing,75.25,true").unwrap();
        file
    }
    
    #[test]
    fn test_data_loading() {
        let test_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        assert!(processor.load_from_file(test_file.path().to_str().unwrap()).is_ok());
        assert_eq!(processor.records.len(), 4);
    }
    
    #[test]
    fn test_filtering() {
        let test_file = create_test_csv();
        let mut processor = DataProcessor::new();
        processor.load_from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let active = processor.filter_active();
        assert_eq!(active.len(), 3);
    }
    
    #[test]
    fn test_calculations() {
        let test_file = create_test_csv();
        let mut processor = DataProcessor::new();
        processor.load_from_file(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.calculate_total_value(), 420.75);
        assert_eq!(processor.calculate_average_value(), 105.1875);
    }
}