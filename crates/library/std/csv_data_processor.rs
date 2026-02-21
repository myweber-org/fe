
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
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
    
    pub fn to_csv_row(&self) -> String {
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
        
        self.records.clear();
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }
            
            let id = parts[0].parse::<u32>()
                .map_err(|e| format!("Invalid ID at line {}: {}", line_num + 1, e))?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()
                .map_err(|e| format!("Invalid value at line {}: {}", line_num + 1, e))?;
            let category = parts[3].to_string();
            
            let record = CsvRecord::new(id, name, value, category)
                .map_err(|e| format!("Validation error at line {}: {}", line_num + 1, e))?;
            
            self.records.push(record);
        }
        
        Ok(())
    }
    
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(path)?;
        writeln!(file, "id,name,value,category")?;
        
        for record in &self.records {
            writeln!(file, "{}", record.to_csv_row())?;
        }
        
        Ok(())
    }
    
    pub fn add_record(&mut self, record: CsvRecord) {
        self.records.push(record);
    }
    
    pub fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }
    
    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }
    
    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            0.0
        } else {
            self.calculate_total_value() / self.records.len() as f64
        }
    }
    
    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }
    
    pub fn clear(&mut self) {
        self.records.clear();
    }
}

pub fn transform_data(input: &[CsvRecord], multiplier: f64) -> Vec<CsvRecord> {
    input
        .iter()
        .filter_map(|record| {
            let transformed_value = record.value * multiplier;
            CsvRecord::new(
                record.id,
                record.name.clone(),
                transformed_value,
                record.category.clone(),
            ).ok()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_csv_record_creation() {
        let record = CsvRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        assert!(record.is_ok());
        
        let record = record.unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Test");
        assert_eq!(record.value, 100.0);
        assert_eq!(record.category, "A");
    }
    
    #[test]
    fn test_csv_record_validation() {
        let result = CsvRecord::new(1, "".to_string(), 100.0, "A".to_string());
        assert!(result.is_err());
        
        let result = CsvRecord::new(1, "Test".to_string(), -10.0, "A".to_string());
        assert!(result.is_err());
        
        let result = CsvRecord::new(1, "Test".to_string(), 100.0, "".to_string());
        assert!(result.is_err());
    }
    
    #[test]
    fn test_csv_processor_operations() {
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
        
        let category_a_records = processor.filter_by_category("CategoryA");
        assert_eq!(category_a_records.len(), 2);
    }
    
    #[test]
    fn test_file_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = CsvProcessor::new();
        
        let record1 = CsvRecord::new(1, "Test1".to_string(), 100.0, "Cat1".to_string()).unwrap();
        let record2 = CsvRecord::new(2, "Test2".to_string(), 200.0, "Cat2".to_string()).unwrap();
        
        processor.add_record(record1);
        processor.add_record(record2);
        
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path();
        
        processor.save_to_file(temp_path)?;
        
        let mut new_processor = CsvProcessor::new();
        new_processor.load_from_file(temp_path)?;
        
        assert_eq!(new_processor.get_records().len(), 2);
        assert_eq!(new_processor.calculate_total_value(), 300.0);
        
        Ok(())
    }
    
    #[test]
    fn test_data_transformation() {
        let records = vec![
            CsvRecord::new(1, "Item1".to_string(), 100.0, "A".to_string()).unwrap(),
            CsvRecord::new(2, "Item2".to_string(), 200.0, "B".to_string()).unwrap(),
        ];
        
        let transformed = transform_data(&records, 1.5);
        
        assert_eq!(transformed.len(), 2);
        assert_eq!(transformed[0].value, 150.0);
        assert_eq!(transformed[1].value, 300.0);
    }
}