
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
        if name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if value < 0.0 {
            return Err("Value must be non-negative".to_string());
        }
        if category.trim().is_empty() {
            return Err("Category cannot be empty".to_string());
        }

        Ok(Self {
            id,
            name: name.trim().to_string(),
            value,
            category: category.trim().to_string(),
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
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }

            let id = parts[0].parse::<u32>()
                .map_err(|_| format!("Invalid ID at line {}", line_num + 1))?;
            
            let name = parts[1].to_string();
            
            let value = parts[2].parse::<f64>()
                .map_err(|_| format!("Invalid value at line {}", line_num + 1))?;
            
            let category = parts[3].to_string();

            let record = CsvRecord::new(id, name, value, category)
                .map_err(|e| format!("Validation error at line {}: {}", line_num + 1, e))?;
            
            self.records.push(record);
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
        
        writeln!(file, "# CSV Export - Processed Data")?;
        writeln!(file, "ID,Name,Value,Category")?;
        
        for record in &self.records {
            writeln!(file, "{}", record.to_csv_row())?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = CsvRecord::new(1, "Test".to_string(), 100.0, "A".to_string());
        assert!(valid_record.is_ok());

        let invalid_name = CsvRecord::new(2, "  ".to_string(), 50.0, "B".to_string());
        assert!(invalid_name.is_err());

        let negative_value = CsvRecord::new(3, "Item".to_string(), -10.0, "C".to_string());
        assert!(negative_value.is_err());
    }

    #[test]
    fn test_csv_processor() {
        let mut processor = CsvProcessor::new();
        
        let record1 = CsvRecord::new(1, "Item1".to_string(), 100.0, "CategoryA".to_string()).unwrap();
        let record2 = CsvRecord::new(2, "Item2".to_string(), 200.0, "CategoryB".to_string()).unwrap();
        
        processor.add_record(record1);
        processor.add_record(record2);
        
        assert_eq!(processor.get_records().len(), 2);
        assert_eq!(processor.calculate_total_value(), 300.0);
        assert_eq!(processor.calculate_average_value(), 150.0);
        
        let category_a = processor.filter_by_category("CategoryA");
        assert_eq!(category_a.len(), 1);
    }

    #[test]
    fn test_file_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = CsvProcessor::new();
        
        let temp_input = NamedTempFile::new()?;
        let temp_output = NamedTempFile::new()?;
        
        let csv_data = "1,Item1,100.0,CategoryA\n2,Item2,200.0,CategoryB\n";
        std::fs::write(&temp_input, csv_data)?;
        
        processor.load_from_file(&temp_input)?;
        assert_eq!(processor.get_records().len(), 2);
        
        processor.save_to_file(&temp_output)?;
        
        let saved_content = std::fs::read_to_string(&temp_output)?;
        assert!(saved_content.contains("Item1"));
        assert!(saved_content.contains("Item2"));
        
        Ok(())
    }
}