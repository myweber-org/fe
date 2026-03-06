use std::collections::HashMap;

pub struct DataAnalyzer {
    data: Vec<f64>,
    frequency_map: HashMap<String, i32>,
}

impl DataAnalyzer {
    pub fn new(data: Vec<f64>) -> Self {
        let mut analyzer = DataAnalyzer {
            data: data.clone(),
            frequency_map: HashMap::new(),
        };
        analyzer.build_frequency_map();
        analyzer
    }

    fn build_frequency_map(&mut self) {
        for &value in &self.data {
            let key = format!("{:.2}", value);
            *self.frequency_map.entry(key).or_insert(0) += 1;
        }
    }

    pub fn calculate_mean(&self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.data.iter().sum();
        sum / self.data.len() as f64
    }

    pub fn calculate_median(&mut self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }
        self.data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = self.data.len() / 2;
        if self.data.len() % 2 == 0 {
            (self.data[mid - 1] + self.data[mid]) / 2.0
        } else {
            self.data[mid]
        }
    }

    pub fn calculate_mode(&self) -> Option<String> {
        let mut max_frequency = 0;
        let mut mode = None;

        for (key, &frequency) in &self.frequency_map {
            if frequency > max_frequency {
                max_frequency = frequency;
                mode = Some(key.clone());
            }
        }

        mode
    }

    pub fn calculate_standard_deviation(&self) -> f64 {
        if self.data.len() < 2 {
            return 0.0;
        }
        let mean = self.calculate_mean();
        let variance: f64 = self.data
            .iter()
            .map(|&value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>() / (self.data.len() - 1) as f64;
        variance.sqrt()
    }

    pub fn get_frequency_distribution(&self) -> &HashMap<String, i32> {
        &self.frequency_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistical_calculations() {
        let data = vec![1.5, 2.3, 1.5, 4.7, 2.3, 1.5, 3.2];
        let mut analyzer = DataAnalyzer::new(data);

        assert!((analyzer.calculate_mean() - 2.428).abs() < 0.001);
        assert!((analyzer.calculate_median() - 2.3).abs() < 0.001);
        assert_eq!(analyzer.calculate_mode(), Some("1.50".to_string()));
        assert!((analyzer.calculate_standard_deviation() - 1.189).abs() < 0.001);
    }

    #[test]
    fn test_empty_data() {
        let data = vec![];
        let mut analyzer = DataAnalyzer::new(data);

        assert_eq!(analyzer.calculate_mean(), 0.0);
        assert_eq!(analyzer.calculate_median(), 0.0);
        assert_eq!(analyzer.calculate_mode(), None);
        assert_eq!(analyzer.calculate_standard_deviation(), 0.0);
    }
}