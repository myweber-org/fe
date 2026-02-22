
use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub fn clean_data(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let unique_lines: HashSet<&str> = lines.iter().cloned().collect();
    
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    
    sorted_lines.join("\n")
}

pub fn process_from_stdin() -> io::Result<()> {
    let stdin = io::stdin();
    let mut buffer = String::new();
    
    for line in stdin.lock().lines() {
        buffer.push_str(&line?);
        buffer.push('\n');
    }
    
    let cleaned = clean_data(&buffer);
    io::stdout().write_all(cleaned.as_bytes())?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_clean_data() {
        let input = "banana\napple\ncherry\napple\nbanana";
        let expected = "apple\nbanana\ncherry";
        assert_eq!(clean_data(input), expected);
    }
    
    #[test]
    fn test_empty_input() {
        assert_eq!(clean_data(""), "");
    }
}
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

        let q1 = Self::calculate_percentile(&sorted_data, 25.0);
        let q3 = Self::calculate_percentile(&sorted_data, 75.0);
        let iqr = q3 - q1;
        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;

        self.data
            .iter()
            .filter(|&&x| x >= lower_bound && x <= upper_bound)
            .cloned()
            .collect()
    }

    fn calculate_percentile(sorted_data: &[f64], percentile: f64) -> f64 {
        let index = (percentile / 100.0) * (sorted_data.len() - 1) as f64;
        let lower = index.floor() as usize;
        let upper = index.ceil() as usize;

        if lower == upper {
            sorted_data[lower]
        } else {
            let weight = index - lower as f64;
            sorted_data[lower] * (1.0 - weight) + sorted_data[upper] * weight
        }
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

        let mut sorted = self.data.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = if count as usize % 2 == 0 {
            (sorted[count as usize / 2 - 1] + sorted[count as usize / 2]) / 2.0
        } else {
            sorted[count as usize / 2]
        };

        stats.insert("mean".to_string(), mean);
        stats.insert("median".to_string(), median);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("count".to_string(), count);
        stats.insert("min".to_string(), *sorted.first().unwrap());
        stats.insert("max".to_string(), *sorted.last().unwrap());

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_outliers() {
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
        assert_eq!(stats["median"], 3.0);
        assert_eq!(stats["count"], 5.0);
    }
}