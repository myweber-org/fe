
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
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }

    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
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

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }
            
            let id = parts[0].parse::<u32>().unwrap_or(0);
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>().unwrap_or(0.0);
            let category = parts[3].to_string();
            
            let record = CsvRecord::new(id, name, value, category);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }
        
        Ok(count)
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

    pub fn apply_transformation<F>(&mut self, mut transform_fn: F)
    where
        F: FnMut(&mut CsvRecord),
    {
        for record in &mut self.records {
            transform_fn(record);
        }
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }
}

pub fn process_csv_data(input_path: &str, output_category: &str) -> Result<f64, Box<dyn Error>> {
    let mut processor = CsvProcessor::new();
    let loaded_count = processor.load_from_file(input_path)?;
    
    if loaded_count == 0 {
        return Err("No valid records loaded".into());
    }
    
    let filtered = processor.filter_by_category(output_category);
    let total: f64 = filtered.iter().map(|record| record.value).sum();
    
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_record_validation() {
        let valid_record = CsvRecord::new(1, "Test".to_string(), 10.5, "CategoryA".to_string());
        assert!(valid_record.is_valid());
        
        let invalid_record = CsvRecord::new(2, "".to_string(), -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_processing() -> Result<(), Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "id,name,value,category")?;
        writeln!(temp_file, "1,Item1,10.5,CategoryA")?;
        writeln!(temp_file, "2,Item2,20.0,CategoryB")?;
        writeln!(temp_file, "3,Item3,15.75,CategoryA")?;
        
        let mut processor = CsvProcessor::new();
        let count = processor.load_from_file(temp_file.path())?;
        
        assert_eq!(count, 3);
        assert_eq!(processor.calculate_total_value(), 46.25);
        
        let category_a_records = processor.filter_by_category("CategoryA");
        assert_eq!(category_a_records.len(), 2);
        
        Ok(())
    }
}