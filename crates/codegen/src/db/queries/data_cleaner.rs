use std::collections::HashSet;
use std::hash::Hash;

pub struct DataCleaner<T> {
    data: Vec<T>,
}

impl<T> DataCleaner<T>
where
    T: Eq + Hash + Clone,
{
    pub fn new(data: Vec<T>) -> Self {
        DataCleaner { data }
    }

    pub fn deduplicate(&mut self) -> &mut Self {
        let mut seen = HashSet::new();
        self.data.retain(|item| seen.insert(item.clone()));
        self
    }

    pub fn normalize<F>(&mut self, normalizer: F) -> &mut Self
    where
        F: Fn(&T) -> T,
    {
        for item in &mut self.data {
            *item = normalizer(item);
        }
        self
    }

    pub fn filter<F>(&mut self, predicate: F) -> &mut Self
    where
        F: Fn(&T) -> bool,
    {
        self.data.retain(predicate);
        self
    }

    pub fn get_data(&self) -> &Vec<T> {
        &self.data
    }

    pub fn into_data(self) -> Vec<T> {
        self.data
    }
}

pub fn clean_string_data(strings: Vec<String>) -> Vec<String> {
    let mut cleaner = DataCleaner::new(strings);
    cleaner
        .deduplicate()
        .normalize(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .into_data()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let data = vec![1, 2, 2, 3, 3, 3];
        let mut cleaner = DataCleaner::new(data);
        cleaner.deduplicate();
        assert_eq!(cleaner.get_data(), &vec![1, 2, 3]);
    }

    #[test]
    fn test_string_cleaning() {
        let data = vec![
            "  HELLO  ".to_string(),
            "world".to_string(),
            "  HELLO  ".to_string(),
            "".to_string(),
            "  TEST  ".to_string(),
        ];
        let cleaned = clean_string_data(data);
        assert_eq!(cleaned, vec!["hello", "world", "test"]);
    }
}use regex::Regex;

pub fn sanitize_input(input: &str) -> String {
    let trimmed = input.trim();
    
    let re_multispace = Regex::new(r"\s+").unwrap();
    let normalized_spaces = re_multispace.replace_all(trimmed, " ");
    
    let re_special = Regex::new(r"[^\w\s\-.,!?]").unwrap();
    let cleaned = re_special.replace_all(&normalized_spaces, "");
    
    cleaned.to_string()
}

pub fn normalize_whitespace(text: &str) -> String {
    let lines: Vec<&str> = text.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect();
    
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_input() {
        let input = "  Hello   World!!  @#$  ";
        let expected = "Hello World!!";
        assert_eq!(sanitize_input(input), expected);
    }

    #[test]
    fn test_normalize_whitespace() {
        let input = "  Line 1  \n\n  Line 2  \n  \n  Line 3  ";
        let expected = "Line 1\nLine 2\nLine 3";
        assert_eq!(normalize_whitespace(input), expected);
    }
}