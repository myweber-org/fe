
use regex::Regex;
use std::collections::HashSet;

pub fn clean_text(input: &str) -> String {
    let trimmed = input.trim();
    
    let re_multispace = Regex::new(r"\s+").unwrap();
    let normalized_spaces = re_multispace.replace_all(trimmed, " ");
    
    let re_special = Regex::new(r"[^\w\s\-.,!?;:]").unwrap();
    let cleaned = re_special.replace_all(&normalized_spaces, "");
    
    cleaned.to_string()
}

pub fn deduplicate_lines(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let unique_lines: HashSet<&str> = lines.into_iter().collect();
    
    let mut sorted_lines: Vec<&str> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    
    sorted_lines.join("\n")
}

pub fn normalize_whitespace(text: &str) -> String {
    let re_newlines = Regex::new(r"\r\n|\r").unwrap();
    let unified = re_newlines.replace_all(text, "\n");
    
    let re_trailing = Regex::new(r"[ \t]+$").unwrap();
    let trimmed_lines: Vec<String> = unified
        .lines()
        .map(|line| re_trailing.replace_all(line, "").to_string())
        .collect();
    
    trimmed_lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_text() {
        let input = "  Hello   World!!!  ";
        let expected = "Hello World!!!";
        assert_eq!(clean_text(input), expected);
    }

    #[test]
    fn test_deduplicate_lines() {
        let input = "apple\nbanana\napple\ncherry\nbanana";
        let result = deduplicate_lines(input);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines.contains(&"apple"));
        assert!(lines.contains(&"banana"));
        assert!(lines.contains(&"cherry"));
    }
}