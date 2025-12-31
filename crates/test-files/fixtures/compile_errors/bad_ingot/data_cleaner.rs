use std::collections::HashSet;

pub struct DataCleaner;

impl DataCleaner {
    pub fn deduplicate_strings(strings: Vec<String>) -> Vec<String> {
        let mut seen = HashSet::new();
        strings
            .into_iter()
            .filter(|s| seen.insert(s.clone()))
            .collect()
    }

    pub fn normalize_whitespace(input: &str) -> String {
        input
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    pub fn remove_empty_lines(text: &str) -> String {
        text.lines()
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<&str>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplicate_strings() {
        let input = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "cherry".to_string(),
        ];
        let result = DataCleaner::deduplicate_strings(input);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"apple".to_string()));
        assert!(result.contains(&"banana".to_string()));
        assert!(result.contains(&"cherry".to_string()));
    }

    #[test]
    fn test_normalize_whitespace() {
        let input = "  hello    world  \t\n  test  ";
        let result = DataCleaner::normalize_whitespace(input);
        assert_eq!(result, "hello world test");
    }

    #[test]
    fn test_remove_empty_lines() {
        let input = "line1\n\nline2\n  \nline3";
        let result = DataCleaner::remove_empty_lines(input);
        assert_eq!(result, "line1\nline2\nline3");
    }
}