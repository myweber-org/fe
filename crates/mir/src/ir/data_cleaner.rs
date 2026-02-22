use std::collections::HashSet;

pub fn clean_string_list(strings: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    strings
        .iter()
        .filter_map(|s| {
            let trimmed = s.trim().to_lowercase();
            if trimmed.is_empty() || seen.contains(&trimmed) {
                None
            } else {
                seen.insert(trimmed.clone());
                Some(trimmed)
            }
        })
        .collect()
}