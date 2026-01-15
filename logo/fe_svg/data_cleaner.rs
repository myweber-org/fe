
use std::collections::HashMap;

pub struct DataCleaner {
    data: Vec<f64>,
}

impl DataCleaner {
    pub fn new(data: Vec<f64>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_outliers_iqr(&mut self) -> Vec<f64> {
        if self.data.len() < 4 {
            return self.data.clone();
        }

        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_index = (sorted_data.len() as f64 * 0.25).floor() as usize;
        let q3_index = (sorted_data.len() as f64 * 0.75).floor() as usize;

        let q1 = sorted_data[q1_index];
        let q3 = sorted_data[q3_index];
        let iqr = q3 - q1;

        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;

        self.data
            .iter()
            .filter(|&&value| value >= lower_bound && value <= upper_bound)
            .cloned()
            .collect()
    }

    pub fn get_summary_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if self.data.is_empty() {
            return stats;
        }

        let sum: f64 = self.data.iter().sum();
        let count = self.data.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / count;
        let std_dev = variance.sqrt();

        let min = *self.data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max = *self.data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

        stats.insert("mean".to_string(), mean);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("min".to_string(), min);
        stats.insert("max".to_string(), max);
        stats.insert("count".to_string(), count);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outlier_removal() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let mut cleaner = DataCleaner::new(data);
        let cleaned = cleaner.remove_outliers_iqr();
        
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }

    #[test]
    fn test_summary_stats() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let cleaner = DataCleaner::new(data);
        let stats = cleaner.get_summary_stats();
        
        assert_eq!(stats["mean"], 3.0);
        assert_eq!(stats["count"], 5.0);
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let mut valid_records = Vec::new();
    let mut invalid_count = 0;

    for result in reader.deserialize() {
        match result {
            Ok(record) => {
                let rec: Record = record;
                if validate_record(&rec) {
                    valid_records.push(rec);
                } else {
                    invalid_count += 1;
                }
            }
            Err(e) => {
                eprintln!("Skipping invalid record: {}", e);
                invalid_count += 1;
            }
        }
    }

    if !valid_records.is_empty() {
        let output_file = File::create(output_path)?;
        let mut writer = csv::Writer::from_writer(output_file);
        
        for record in valid_records {
            writer.serialize(record)?;
        }
        
        writer.flush()?;
        println!("Processed {} records, {} invalid records filtered", valid_records.len(), invalid_count);
    } else {
        println!("No valid records found in the input file");
    }

    Ok(())
}

fn validate_record(record: &Record) -> bool {
    !record.name.trim().is_empty() &&
    record.value >= 0.0 &&
    !record.category.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_clean_csv_data() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "id,name,value,category").unwrap();
        writeln!(input_file, "1,Test Product,25.5,Electronics").unwrap();
        writeln!(input_file, "2,,15.0,Books").unwrap();
        writeln!(input_file, "3,Invalid Product,-5.0,Electronics").unwrap();

        let output_file = NamedTempFile::new().unwrap();
        
        let result = clean_csv_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap()
        );
        
        assert!(result.is_ok());
    }
}use std::collections::HashMap;

pub struct DataCleaner {
    data: Vec<f64>,
    thresholds: HashMap<String, f64>,
}

impl DataCleaner {
    pub fn new(data: Vec<f64>) -> Self {
        DataCleaner {
            data,
            thresholds: HashMap::new(),
        }
    }

    pub fn calculate_iqr(&mut self) -> (f64, f64, f64, f64) {
        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_index = (sorted_data.len() as f64 * 0.25) as usize;
        let q3_index = (sorted_data.len() as f64 * 0.75) as usize;

        let q1 = sorted_data[q1_index];
        let q3 = sorted_data[q3_index];
        let iqr = q3 - q1;

        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;

        self.thresholds.insert("lower_bound".to_string(), lower_bound);
        self.thresholds.insert("upper_bound".to_string(), upper_bound);
        self.thresholds.insert("iqr".to_string(), iqr);
        self.thresholds.insert("q1".to_string(), q1);
        self.thresholds.insert("q3".to_string(), q3);

        (q1, q3, iqr, lower_bound)
    }

    pub fn remove_outliers(&self) -> Vec<f64> {
        let lower_bound = self.thresholds.get("lower_bound").unwrap_or(&f64::MIN);
        let upper_bound = self.thresholds.get("upper_bound").unwrap_or(&f64::MAX);

        self.data
            .iter()
            .filter(|&&value| value >= *lower_bound && value <= *upper_bound)
            .cloned()
            .collect()
    }

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        self.thresholds.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outlier_removal() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let mut cleaner = DataCleaner::new(data);
        cleaner.calculate_iqr();
        let cleaned = cleaner.remove_outliers();
        
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }
}
use std::collections::HashMap;

pub struct DataCleaner {
    data: Vec<f64>,
    threshold: f64,
}

impl DataCleaner {
    pub fn new(data: Vec<f64>, threshold: f64) -> Self {
        DataCleaner { data, threshold }
    }

    pub fn remove_outliers(&self) -> Vec<f64> {
        if self.data.len() < 4 {
            return self.data.clone();
        }

        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_index = (sorted_data.len() as f64 * 0.25).floor() as usize;
        let q3_index = (sorted_data.len() as f64 * 0.75).floor() as usize;

        let q1 = sorted_data[q1_index];
        let q3 = sorted_data[q3_index];
        let iqr = q3 - q1;

        let lower_bound = q1 - self.threshold * iqr;
        let upper_bound = q3 + self.threshold * iqr;

        self.data
            .iter()
            .filter(|&&value| value >= lower_bound && value <= upper_bound)
            .cloned()
            .collect()
    }

    pub fn summary_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if self.data.is_empty() {
            return stats;
        }

        let sum: f64 = self.data.iter().sum();
        let count = self.data.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / count;
        let std_dev = variance.sqrt();

        let mut sorted = self.data.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let median = if count as usize % 2 == 0 {
            let mid = count as usize / 2;
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[count as usize / 2]
        };

        stats.insert("mean".to_string(), mean);
        stats.insert("median".to_string(), median);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("count".to_string(), count);
        stats.insert("min".to_string(), *sorted.first().unwrap_or(&0.0));
        stats.insert("max".to_string(), *sorted.last().unwrap_or(&0.0));

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_outliers() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let cleaner = DataCleaner::new(data, 1.5);
        let cleaned = cleaner.remove_outliers();
        
        assert_eq!(cleaned, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_summary_statistics() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let cleaner = DataCleaner::new(data, 1.5);
        let stats = cleaner.summary_statistics();
        
        assert_eq!(stats.get("mean").unwrap(), &3.0);
        assert_eq!(stats.get("median").unwrap(), &3.0);
        assert_eq!(stats.get("count").unwrap(), &5.0);
    }
}