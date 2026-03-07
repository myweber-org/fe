use std::collections::HashMap;

pub struct DataAnalyzer {
    values: Vec<f64>,
}

impl DataAnalyzer {
    pub fn new() -> Self {
        DataAnalyzer {
            values: Vec::new(),
        }
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }

    pub fn calculate_median(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let mut sorted = self.values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            Some((sorted[mid - 1] + sorted[mid]) / 2.0)
        } else {
            Some(sorted[mid])
        }
    }

    pub fn calculate_mode(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let mut frequency_map = HashMap::new();
        for &value in &self.values {
            *frequency_map.entry(value.to_bits()).or_insert(0) += 1;
        }
        frequency_map
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(bits, _)| f64::from_bits(bits))
    }

    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        if self.values.len() < 2 {
            return None;
        }
        let mean = self.calculate_mean().unwrap();
        let variance: f64 = self.values
            .iter()
            .map(|&value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>() / (self.values.len() - 1) as f64;
        Some(variance.sqrt())
    }

    pub fn get_summary(&self) -> Option<DataSummary> {
        if self.values.is_empty() {
            return None;
        }
        Some(DataSummary {
            count: self.values.len(),
            mean: self.calculate_mean().unwrap(),
            median: self.calculate_median().unwrap(),
            mode: self.calculate_mode(),
            std_dev: self.calculate_standard_deviation(),
        })
    }
}

pub struct DataSummary {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub mode: Option<f64>,
    pub std_dev: Option<f64>,
}

impl std::fmt::Display for DataSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Data Summary:")?;
        writeln!(f, "  Count: {}", self.count)?;
        writeln!(f, "  Mean: {:.4}", self.mean)?;
        writeln!(f, "  Median: {:.4}", self.median)?;
        if let Some(mode) = self.mode {
            writeln!(f, "  Mode: {:.4}", mode)?;
        } else {
            writeln!(f, "  Mode: None")?;
        }
        if let Some(std_dev) = self.std_dev {
            writeln!(f, "  Standard Deviation: {:.4}", std_dev)
        } else {
            writeln!(f, "  Standard Deviation: Not enough data")
        }
    }
}