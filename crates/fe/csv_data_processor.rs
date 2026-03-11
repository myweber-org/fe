
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl CsvRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Result<Self, String> {
        if name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        
        Ok(Self {
            id,
            name,
            value,
            category,
        })
    }
    
    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
    
    pub fn to_csv_string(&self) -> String {
        format!("{},{},{:.2},{}", self.id, self.name, self.value, self.category)
    }
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }
    
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }
            
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();
            
            match CsvRecord::new(id, name, value, category) {
                Ok(record) => self.records.push(record),
                Err(e) => eprintln!("Skipping invalid record at line {}: {}", line_num + 1, e),
            }
        }
        
        Ok(())
    }
    
    pub fn add_record(&mut self, record: CsvRecord) {
        self.records.push(record);
    }
    
    pub fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }
    
    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }
    
    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            0.0
        } else {
            self.calculate_total_value() / self.records.len() as f64
        }
    }
    
    pub fn transform_all_values(&mut self, multiplier: f64) {
        for record in &mut self.records {
            record.transform_value(multiplier);
        }
    }
    
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let mut content = String::from("id,name,value,category\n");
        
        for record in &self.records {
            content.push_str(&record.to_csv_string());
            content.push('\n');
        }
        
        std::fs::write(path, content)?;
        Ok(())
    }
    
    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }
    
    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_record_creation() {
        let record = CsvRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        assert!(record.is_ok());
        
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Test");
        assert_eq!(record.value, 100.0);
        assert_eq!(record.category, "A");
    }
    
    #[test]
    fn test_invalid_record() {
        let record = CsvRecord::new(1, "".to_string(), 100.0, "A".to_string());
        assert!(record.is_err());
        
        let record = CsvRecord::new(1, "Test".to_string(), -10.0, "A".to_string());
        assert!(record.is_err());
        
        let record = CsvRecord::new(1, "Test".to_string(), 100.0, "".to_string());
        assert!(record.is_err());
    }
    
    #[test]
    fn test_value_transformation() {
        let mut record = CsvRecord::new(1, "Test".to_string(), 100.0, "A".to_string()).unwrap();
        record.transform_value(1.5);
        assert_eq!(record.value, 150.0);
    }
    
    #[test]
    fn test_csv_processor() {
        let mut processor = CsvProcessor::new();
        
        let record1 = CsvRecord::new(1, "Item1".to_string(), 50.0, "CategoryA".to_string()).unwrap();
        let record2 = CsvRecord::new(2, "Item2".to_string(), 75.0, "CategoryB".to_string()).unwrap();
        let record3 = CsvRecord::new(3, "Item3".to_string(), 25.0, "CategoryA".to_string()).unwrap();
        
        processor.add_record(record1);
        processor.add_record(record2);
        processor.add_record(record3);
        
        assert_eq!(processor.get_records().len(), 3);
        assert_eq!(processor.calculate_total_value(), 150.0);
        assert_eq!(processor.calculate_average_value(), 50.0);
        
        let category_a = processor.filter_by_category("CategoryA");
        assert_eq!(category_a.len(), 2);
        
        processor.transform_all_values(2.0);
        assert_eq!(processor.calculate_total_value(), 300.0);
    }
    
    #[test]
    fn test_file_operations() -> Result<(), Box<dyn Error>> {
        let temp_file = NamedTempFile::new()?;
        let file_path = temp_file.path();
        
        let csv_content = "id,name,value,category\n1,Test1,100.0,A\n2,Test2,200.0,B\n";
        std::fs::write(file_path, csv_content)?;
        
        let mut processor = CsvProcessor::new();
        processor.load_from_file(file_path)?;
        
        assert_eq!(processor.get_records().len(), 2);
        
        let output_file = NamedTempFile::new()?;
        let output_path = output_file.path();
        processor.save_to_file(output_path)?;
        
        let saved_content = std::fs::read_to_string(output_path)?;
        assert!(saved_content.contains("Test1"));
        assert!(saved_content.contains("Test2"));
        
        Ok(())
    }
}