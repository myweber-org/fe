use std::collections::HashMap;

pub fn parse_json_object(input: &str) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    let trimmed = input.trim();

    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return Err("Invalid JSON object format".to_string());
    }

    let content = &trimmed[1..trimmed.len() - 1].trim();
    if content.is_empty() {
        return Ok(map);
    }

    for pair in content.split(',') {
        let parts: Vec<&str> = pair.split(':').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            return Err("Invalid key-value pair".to_string());
        }

        let key = parts[0].trim_matches('"');
        let value = parts[1].trim_matches('"');
        map.insert(key.to_string(), value.to_string());
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_json() {
        let json = r#"{"name": "Alice", "age": "30"}"#;
        let result = parse_json_object(json).unwrap();
        assert_eq!(result.get("name"), Some(&"Alice".to_string()));
        assert_eq!(result.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_parse_empty_object() {
        let json = "{}";
        let result = parse_json_object(json).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = r#"{"name": "Bob""#;
        assert!(parse_json_object(json).is_err());
    }
}