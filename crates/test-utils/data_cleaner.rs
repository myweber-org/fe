use std::collections::HashSet;

pub fn clean_data(input: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut cleaned = Vec::new();

    for item in input {
        let normalized = item.trim().to_lowercase();
        if seen.insert(normalized.clone()) {
            cleaned.push(normalized);
        }
    }

    cleaned.sort();
    cleaned
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_data() {
        let input = vec![
            "  Apple ".to_string(),
            "apple".to_string(),
            "Banana".to_string(),
            "banana ".to_string(),
            "Cherry".to_string(),
        ];
        let result = clean_data(input);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }
}