
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category,
            valid,
        }
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    total_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            total_value: 0.0,
        }
    }

    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 3 {
                continue;
            }

            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(0.0);
            let category = parts[2].to_string();

            let record = DataRecord::new(id, value, category);
            self.total_value += record.value;
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn filter_valid_records(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            0.0
        } else {
            self.total_value / self.records.len() as f64
        }
    }

    pub fn get_category_summary(&self) -> Vec<(String, usize, f64)> {
        use std::collections::HashMap;

        let mut summary: HashMap<String, (usize, f64)> = HashMap::new();

        for record in &self.records {
            let entry = summary.entry(record.category.clone()).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += record.value;
        }

        summary
            .into_iter()
            .map(|(category, (count, total))| (category, count, total))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,TypeA").unwrap();
        writeln!(temp_file, "2,15.0,TypeB").unwrap();
        writeln!(temp_file, "3,-5.0,TypeA").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        
        let valid_records = processor.filter_valid_records();
        assert_eq!(valid_records.len(), 2);
        
        let average = processor.calculate_average();
        assert!((average - 6.8333).abs() < 0.001);
        
        let summary = processor.get_category_summary();
        assert_eq!(summary.len(), 2);
    }
}