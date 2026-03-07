
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
            
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();
            
            match CsvRecord::new(id, name, value, category) {
                Ok(record) => self.records.push(record),
                Err(e) => return Err(format!("Validation error at line {}: {}", line_num + 1, e).into()),
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
    
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(path)?;
        writeln!(file, "id,name,value,category")?;
        
        for record in &self.records {
            writeln!(file, "{}", record.to_csv_string())?;
        }
        
        Ok(())
    }
    
    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }
    
    pub fn clear(&mut self) {
        self.records.clear();
    }
}

pub fn process_csv_data(input_path: &str, output_path: &str, filter_category: Option<&str>) -> Result<(), Box<dyn Error>> {
    let mut processor = CsvProcessor::new();
    processor.load_from_file(input_path)?;
    
    println!("Loaded {} records", processor.get_records().len());
    println!("Total value: {:.2}", processor.calculate_total_value());
    println!("Average value: {:.2}", processor.calculate_average_value());
    
    if let Some(category) = filter_category {
        let filtered = processor.filter_by_category(category);
        println!("Found {} records in category '{}'", filtered.len(), category);
    }
    
    processor.save_to_file(output_path)?;
    println!("Results saved to {}", output_path);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
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
        let record = CsvRecord::new(1, "".to_string(), 100.0, "A".to_string());
        assert!(record.is_err());
        
        let record = CsvRecord::new(1, "Test".to_string(), -10.0, "A".to_string());
        assert!(record.is_err());
        
        let record = CsvRecord::new(1, "Test".to_string(), 100.0, "".to_string());
        assert!(record.is_err());
    }
    
    #[test]
    fn test_csv_processor() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "1,Item1,100.0,CategoryA")?;
        writeln!(temp_file, "2,Item2,200.0,CategoryB")?;
        writeln!(temp_file, "3,Item3,300.0,CategoryA")?;
        
        let mut processor = CsvProcessor::new();
        processor.load_from_file(temp_file.path())?;
        
        assert_eq!(processor.get_records().len(), 3);
        assert_eq!(processor.calculate_total_value(), 600.0);
        assert_eq!(processor.calculate_average_value(), 200.0);
        
        let category_a_records = processor.filter_by_category("CategoryA");
        assert_eq!(category_a_records.len(), 2);
        
        Ok(())
    }
}