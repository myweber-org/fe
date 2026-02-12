use std::collections::HashMap;

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

        (q1, q3, lower_bound, upper_bound)
    }

    pub fn remove_outliers(&self) -> Vec<f64> {
        let lower = self.thresholds.get("lower_bound").unwrap_or(&f64::MIN);
        let upper = self.thresholds.get("upper_bound").unwrap_or(&f64::MAX);

        self.data
            .iter()
            .filter(|&&value| value >= *lower && value <= *upper)
            .cloned()
            .collect()
    }

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        stats.insert("min".to_string(), self.data.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
        stats.insert("max".to_string(), self.data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));
        stats.insert("mean".to_string(), self.data.iter().sum::<f64>() / self.data.len() as f64);
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
        cleaner.calculate_iqr();
        let cleaned = cleaner.remove_outliers();
        
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }
}use std::collections::HashSet;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn normalize_text(&self, input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        if self.dedupe_set.contains(&normalized) {
            false
        } else {
            self.dedupe_set.insert(normalized);
            true
        }
    }

    pub fn clean_dataset(&mut self, data: Vec<&str>) -> Vec<String> {
        data.iter()
            .filter(|&&item| self.deduplicate(item))
            .map(|&item| self.normalize_text(item))
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        let data = vec!["Apple", "apple", "APPLE", "Banana", "banana"];
        let cleaned = cleaner.clean_dataset(data);
        
        assert_eq!(cleaned.len(), 2);
        assert_eq!(cleaner.get_unique_count(), 2);
        assert!(cleaned.contains(&"apple".to_string()));
        assert!(cleaned.contains(&"banana".to_string()));
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  HELLO World  "), "hello world");
    }
}
use std::collections::HashSet;

pub struct DataCleaner {
    pub remove_duplicates: bool,
    pub validate_emails: bool,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            remove_duplicates: true,
            validate_emails: false,
        }
    }

    pub fn deduplicate_strings(&self, items: Vec<String>) -> Vec<String> {
        if !self.remove_duplicates {
            return items;
        }

        let mut seen = HashSet::new();
        items
            .into_iter()
            .filter(|item| seen.insert(item.clone()))
            .collect()
    }

    pub fn is_valid_email(&self, email: &str) -> bool {
        if !self.validate_emails {
            return true;
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }

        let domain_parts: Vec<&str> = parts[1].split('.').collect();
        domain_parts.len() >= 2 && !email.contains(' ')
    }

    pub fn clean_data(&self, emails: Vec<String>) -> Vec<String> {
        let mut cleaned = self.deduplicate_strings(emails);
        
        if self.validate_emails {
            cleaned.retain(|email| self.is_valid_email(email));
        }

        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let cleaner = DataCleaner::new();
        let data = vec![
            "test@example.com".to_string(),
            "test@example.com".to_string(),
            "other@domain.org".to_string(),
        ];
        
        let result = cleaner.deduplicate_strings(data);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_email_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.validate_emails = true;
        
        assert!(cleaner.is_valid_email("user@example.com"));
        assert!(!cleaner.is_valid_email("invalid email@example.com"));
        assert!(!cleaner.is_valid_email("user@com"));
    }
}