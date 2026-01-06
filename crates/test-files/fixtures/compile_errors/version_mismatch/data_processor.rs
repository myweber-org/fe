use csv::Reader;
use std::error::Error;
use std::fs::File;

pub struct DataProcessor {
    data: Vec<Vec<f64>>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor { data: Vec::new() }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.records() {
            let record = result?;
            let row: Vec<f64> = record.iter()
                .filter_map(|s| s.parse().ok())
                .collect();
            
            if !row.is_empty() {
                self.data.push(row);
            }
        }
        
        Ok(())
    }

    pub fn calculate_column_averages(&self) -> Vec<f64> {
        if self.data.is_empty() {
            return Vec::new();
        }
        
        let num_columns = self.data[0].len();
        let mut sums = vec![0.0; num_columns];
        let mut counts = vec![0; num_columns];
        
        for row in &self.data {
            for (i, &value) in row.iter().enumerate() {
                if i < num_columns {
                    sums[i] += value;
                    counts[i] += 1;
                }
            }
        }
        
        sums.iter()
            .zip(counts.iter())
            .map(|(&sum, &count)| if count > 0 { sum / count as f64 } else { 0.0 })
            .collect()
    }

    pub fn filter_by_threshold(&self, column_index: usize, threshold: f64) -> Vec<Vec<f64>> {
        self.data.iter()
            .filter(|row| column_index < row.len() && row[column_index] > threshold)
            .cloned()
            .collect()
    }

    pub fn get_data_summary(&self) -> (usize, usize) {
        if self.data.is_empty() {
            (0, 0)
        } else {
            (self.data.len(), self.data[0].len())
        }
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
        writeln!(temp_file, "1.5,2.3,3.7").unwrap();
        writeln!(temp_file, "4.2,5.1,6.8").unwrap();
        writeln!(temp_file, "7.3,8.4,9.2").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let averages = processor.calculate_column_averages();
        assert_eq!(averages.len(), 3);
        
        let filtered = processor.filter_by_threshold(1, 5.0);
        assert_eq!(filtered.len(), 2);
        
        let summary = processor.get_data_summary();
        assert_eq!(summary, (3, 3));
    }
}