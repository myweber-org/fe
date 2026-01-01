use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

impl CsvRecord {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Self {
            id,
            name,
            value,
            active,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        Ok(())
    }

    pub fn transform(&mut self, multiplier: f64) {
        self.value *= multiplier;
        self.name = self.name.to_uppercase();
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

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

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
            let active = parts[3].parse::<bool>().unwrap_or(false);

            let record = CsvRecord::new(id, name, value, active);
            self.records.push(record);
        }

        Ok(())
    }

    pub fn validate_all(&self) -> Vec<Result<(), String>> {
        self.records
            .iter()
            .map(|record| record.validate())
            .collect()
    }

    pub fn transform_all(&mut self, multiplier: f64) {
        for record in &mut self.records {
            record.transform(multiplier);
        }
    }

    pub fn filter_active(&self) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn save_to_file(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(file_path)?;
        writeln!(file, "id,name,value,active")?;

        for record in &self.records {
            writeln!(
                file,
                "{},{},{},{}",
                record.id, record.name, record.value, record.active
            )?;
        }

        Ok(())
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let count = values.len() as f64;

        if count == 0.0 {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / count;

        let variance: f64 = values
            .iter()
            .map(|value| {
                let diff = mean - value;
                diff * diff
            })
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

impl Default for CsvProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = CsvRecord::new(1, "Test".to_string(), 100.0, true);
        assert!(valid_record.validate().is_ok());

        let invalid_record = CsvRecord::new(2, "".to_string(), -50.0, false);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_record_transformation() {
        let mut record = CsvRecord::new(1, "test".to_string(), 100.0, true);
        record.transform(2.0);
        assert_eq!(record.name, "TEST");
        assert_eq!(record.value, 200.0);
    }

    #[test]
    fn test_csv_processing() {
        let mut processor = CsvProcessor::new();
        
        let csv_data = "id,name,value,active\n1,item1,100.0,true\n2,item2,200.0,false\n";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(csv_data.as_bytes()).unwrap();
        
        let result = processor.load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 2);
        
        let active_records = processor.filter_active();
        assert_eq!(active_records.len(), 1);
        
        let total = processor.calculate_total();
        assert_eq!(total, 300.0);
    }
}