
use serde_json::{Value, Map};
use std::fs;
use std::path::Path;
use std::collections::HashSet;

pub fn merge_json_files<P: AsRef<Path>>(paths: &[P], output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged = Map::new();
    let mut all_keys = HashSet::new();
    let mut file_values = Vec::new();

    for path in paths {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;
        if let Value::Object(map) = json {
            for key in map.keys() {
                all_keys.insert(key.clone());
            }
            file_values.push(map);
        } else {
            return Err("Each JSON file must contain an object".into());
        }
    }

    for key in all_keys {
        let mut values = Vec::new();
        for file_map in &file_values {
            if let Some(value) = file_map.get(&key) {
                values.push(value.clone());
            }
        }

        let merged_value = if values.is_empty() {
            Value::Null
        } else if values.len() == 1 {
            values[0].clone()
        } else {
            resolve_conflict(&key, &values)
        };

        merged.insert(key, merged_value);
    }

    let output_json = Value::Object(merged);
    let output_str = serde_json::to_string_pretty(&output_json)?;
    fs::write(output_path, output_str)?;

    Ok(())
}

fn resolve_conflict(key: &str, values: &[Value]) -> Value {
    let first = &values[0];
    
    if values.iter().all(|v| v == first) {
        return first.clone();
    }

    if key.ends_with("_list") || key.ends_with("_items") {
        let mut combined = Vec::new();
        for value in values {
            if let Value::Array(arr) = value {
                combined.extend(arr.clone());
            } else {
                combined.push(value.clone());
            }
        }
        return Value::Array(combined);
    }

    if key.ends_with("_count") || key.ends_with("_total") {
        let numbers: Vec<f64> = values.iter()
            .filter_map(|v| v.as_f64())
            .collect();
        
        if !numbers.is_empty() {
            return Value::from(numbers.iter().sum::<f64>());
        }
    }

    Value::Array(values.to_vec())
}