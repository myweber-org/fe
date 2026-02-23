
use std::collections::HashMap;

pub fn clean_dataset(data: &mut Vec<HashMap<String, String>>, required_keys: &[&str]) -> Vec<HashMap<String, String>> {
    data.drain(..)
        .filter(|entry| {
            required_keys.iter().all(|key| {
                if let Some(value) = entry.get(*key) {
                    !value.trim().is_empty()
                } else {
                    false
                }
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_dataset() {
        let mut data = vec![
            HashMap::from([
                ("id".to_string(), "1".to_string()),
                ("name".to_string(), "Alice".to_string()),
            ]),
            HashMap::from([
                ("id".to_string(), "2".to_string()),
                ("name".to_string(), "".to_string()),
            ]),
            HashMap::from([
                ("id".to_string(), "3".to_string()),
            ]),
        ];

        let required = ["id", "name"];
        let cleaned = clean_dataset(&mut data, &required);

        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0]["id"], "1");
        assert_eq!(cleaned[0]["name"], "Alice");
    }
}