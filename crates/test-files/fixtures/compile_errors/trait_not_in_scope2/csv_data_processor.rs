use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let record = CsvRecord {
                    id: parts[0].parse()?,
                    category: parts[1].to_string(),
                    value: parts[2].parse()?,
                    active: parts[3].parse().unwrap_or(false),
                };
                self.records.push(record);
            }
        }
        
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average_by_category(&self) -> HashMap<String, f64> {
        let mut category_totals: HashMap<String, (f64, usize)> = HashMap::new();
        
        for record in &self.records {
            if record.active {
                let entry = category_totals
                    .entry(record.category.clone())
                    .or_insert((0.0, 0));
                entry.0 += record.value;
                entry.1 += 1;
            }
        }
        
        category_totals
            .into_iter()
            .map(|(category, (total, count))| (category, total / count as f64))
            .collect()
    }

    pub fn find_max_value(&self) -> Option<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }

    pub fn get_total_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_active_records(&self) -> usize {
        self.records.iter().filter(|record| record.active).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,category,value,active").unwrap();
        writeln!(temp_file, "1,electronics,250.50,true").unwrap();
        writeln!(temp_file, "2,clothing,89.99,true").unwrap();
        writeln!(temp_file, "3,electronics,150.00,false").unwrap();
        writeln!(temp_file, "4,clothing,45.75,true").unwrap();
        
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(processor.get_total_records(), 4);
        assert_eq!(processor.get_active_records(), 3);
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let averages = processor.calculate_average_by_category();
        assert!(averages.contains_key("electronics"));
        assert!(averages.contains_key("clothing"));
        
        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().id, 1);
    }
}