use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, usize>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
        }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            for part in parts {
                if let Ok(value) = part.trim().parse::<f64>() {
                    self.data.push(value);
                } else {
                    let count = self.frequency_map.entry(part.trim().to_string()).or_insert(0);
                    *count += 1;
                }
            }
        }
        
        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        let sum: f64 = self.data.iter().sum();
        Some(sum / self.data.len() as f64)
    }

    pub fn calculate_median(&mut self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }
        
        self.data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = self.data.len() / 2;
        
        if self.data.len() % 2 == 0 {
            Some((self.data[mid - 1] + self.data[mid]) / 2.0)
        } else {
            Some(self.data[mid])
        }
    }

    pub fn get_top_categories(&self, limit: usize) -> Vec<(String, usize)> {
        let mut entries: Vec<_> = self.frequency_map.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1));
        
        entries
            .iter()
            .take(limit)
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }

    pub fn data_summary(&self) -> String {
        let mean_str = match self.calculate_mean() {
            Some(mean) => format!("{:.2}", mean),
            None => "N/A".to_string(),
        };
        
        let top_categories = self.get_top_categories(3);
        let categories_str: Vec<String> = top_categories
            .iter()
            .map(|(cat, count)| format!("{}: {}", cat, count))
            .collect();
        
        format!(
            "Data points: {}, Mean: {}, Top categories: {}",
            self.data.len(),
            mean_str,
            categories_str.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "10.5,20.3,15.7").unwrap();
        writeln!(temp_file, "category_a,category_b,category_a").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        assert_eq!(processor.calculate_mean(), Some(15.5));
        assert_eq!(processor.get_top_categories(2).len(), 2);
    }
}