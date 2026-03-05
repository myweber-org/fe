use serde_json;

pub fn is_valid_json(input: &str) -> bool {
    match serde_json::from_str::<serde_json::Value>(input) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        assert!(is_valid_json(r#"{"name": "Alice", "age": 30}"#));
        assert!(is_valid_json(r#"[1, 2, 3, 4, 5]"#));
        assert!(is_valid_json(r#"true"#));
    }

    #[test]
    fn test_invalid_json() {
        assert!(!is_valid_json(r#"{"name": "Alice", "age": }"#));
        assert!(!is_valid_json(r#"[1, 2, 3,,]"#));
        assert!(!is_valid_json(r#"not json"#));
    }
}