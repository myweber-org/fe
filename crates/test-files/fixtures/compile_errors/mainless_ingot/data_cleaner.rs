
use std::collections::HashMap;

pub struct DataCleaner;

impl DataCleaner {
    pub fn clean_string_vector(data: Vec<Option<String>>) -> Vec<String> {
        data.into_iter()
            .filter_map(|opt| opt)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn clean_hashmap(data: HashMap<String, Option<f64>>) -> HashMap<String, f64> {
        data.into_iter()
            .filter_map(|(key, value)| value.map(|v| (key, v)))
            .collect()
    }

    pub fn remove_outliers(values: &[f64], threshold: f64) -> Vec<f64> {
        if values.is_empty() {
            return Vec::new();
        }

        let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        values.iter()
            .filter(|&&x| (x - mean).abs() <= threshold * std_dev)
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_string_vector() {
        let input = vec![
            Some("  hello  ".to_string()),
            None,
            Some("world".to_string()),
            Some("  ".to_string()),
            Some("".to_string()),
        ];
        let result = DataCleaner::clean_string_vector(input);
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn test_clean_hashmap() {
        let mut input = HashMap::new();
        input.insert("a".to_string(), Some(1.0));
        input.insert("b".to_string(), None);
        input.insert("c".to_string(), Some(2.5));

        let result = DataCleaner::clean_hashmap(input);
        assert_eq!(result.len(), 2);
        assert_eq!(result.get("a"), Some(&1.0));
        assert_eq!(result.get("c"), Some(&2.5));
    }

    #[test]
    fn test_remove_outliers() {
        let values = vec![1.0, 2.0, 3.0, 100.0];
        let result = DataCleaner::remove_outliers(&values, 2.0);
        assert_eq!(result, vec![1.0, 2.0, 3.0]);
    }
}