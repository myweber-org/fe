
fn clean_string(input: &str) -> String {
    input.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_string() {
        assert_eq!(clean_string("Hello, World! 123"), "HelloWorld123");
        assert_eq!(clean_string("Rust_2024!"), "Rust2024");
        assert_eq!(clean_string(""), "");
        assert_eq!(clean_string("!@#$%"), "");
    }
}