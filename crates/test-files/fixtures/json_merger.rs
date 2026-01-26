
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;
use std::collections::HashSet;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P]) -> Result<Value, String> {
    if paths.is_empty() {
        return Err("No input files provided".to_string());
    }

    let mut merged = Map::new();
    let mut key_sources = Map::new();

    for (index, path) in paths.iter().enumerate() {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                if let Some(existing) = merged.get(&key) {
                    if existing != &value {
                        let sources = key_sources.entry(key.clone())
                            .or_insert_with(Vec::new);
                        sources.push(index);
                        sources.sort();
                        sources.dedup();
                    }
                } else {
                    merged.insert(key.clone(), value);
                }
            }
        } else {
            return Err(format!("Expected JSON object in {}", path.as_ref().display()));
        }
    }

    let conflicts: Vec<String> = key_sources.iter()
        .filter(|(_, sources)| sources.len() > 1)
        .map(|(key, _)| key.clone())
        .collect();

    if !conflicts.is_empty() {
        return Err(format!("Conflicting keys found: {}", conflicts.join(", ")));
    }

    Ok(Value::Object(merged))
}

pub fn merge_json_with_strategy<P: AsRef<Path>>(
    paths: &[P],
    strategy: MergeStrategy
) -> Result<Value, String> {
    let mut merged = Map::new();
    
    for path in paths {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json {
            for (key, value) in obj {
                match strategy {
                    MergeStrategy::FirstWins => {
                        merged.entry(key).or_insert(value);
                    }
                    MergeStrategy::LastWins => {
                        merged.insert(key, value);
                    }
                    MergeStrategy::MergeObjects => {
                        if let Some(Value::Object(existing)) = merged.get(&key) {
                            if let Value::Object(new_obj) = value {
                                let mut combined = existing.clone();
                                for (k, v) in new_obj {
                                    combined.insert(k, v);
                                }
                                merged.insert(key, Value::Object(combined));
                            } else {
                                merged.insert(key, value);
                            }
                        } else {
                            merged.insert(key, value);
                        }
                    }
                }
            }
        } else {
            return Err(format!("Expected JSON object in {}", path.as_ref().display()));
        }
    }

    Ok(Value::Object(merged))
}

pub enum MergeStrategy {
    FirstWins,
    LastWins,
    MergeObjects,
}

pub fn find_common_keys<P: AsRef<Path>>(paths: &[P]) -> Result<HashSet<String>, String> {
    let mut common_keys: Option<HashSet<String>> = None;

    for path in paths {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.as_ref().display(), e))?;

        if let Value::Object(obj) = json {
            let current_keys: HashSet<String> = obj.keys().cloned().collect();
            
            common_keys = match common_keys {
                None => Some(current_keys),
                Some(existing) => Some(existing.intersection(&current_keys).cloned().collect())
            };
        } else {
            return Err(format!("Expected JSON object in {}", path.as_ref().display()));
        }
    }

    Ok(common_keys.unwrap_or_default())
}