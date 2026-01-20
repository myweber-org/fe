use serde_json::{Value, json};
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

pub fn merge_json_files(input_paths: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_array = Vec::new();

    for input_path in input_paths {
        let path = Path::new(input_path);
        if !path.exists() {
            eprintln!("Warning: File {} not found, skipping.", input_path);
            continue;
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let json_value: Value = serde_json::from_reader(reader)?;

        match json_value {
            Value::Array(arr) => {
                merged_array.extend(arr);
            }
            _ => {
                merged_array.push(json_value);
            }
        }
    }

    let output_file = File::create(output_path)?;
    let merged_json = json!(merged_array);
    serde_json::to_writer_pretty(output_file, &merged_json)?;

    Ok(())
}

pub fn merge_json_directories(input_dirs: &[&str], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut all_json_paths = Vec::new();

    for dir in input_dirs {
        let entries = fs::read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                if let Some(path_str) = path.to_str() {
                    all_json_paths.push(path_str.to_string());
                }
            }
        }
    }

    let path_refs: Vec<&str> = all_json_paths.iter().map(|s| s.as_str()).collect();
    merge_json_files(&path_refs, output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_merge_json_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.path().join("data1.json");
        let file2_path = temp_dir.path().join("data2.json");
        let output_path = temp_dir.path().join("merged.json");

        fs::write(&file1_path, r#"[{"id": 1}, {"id": 2}]"#).unwrap();
        fs::write(&file2_path, r#"{"id": 3}"#).unwrap();

        let inputs = [
            file1_path.to_str().unwrap(),
            file2_path.to_str().unwrap()
        ];

        merge_json_files(&inputs, output_path.to_str().unwrap()).unwrap();

        let content = fs::read_to_string(output_path).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }
}
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub struct JsonMerger {
    conflict_resolution: ConflictResolution,
}

pub enum ConflictResolution {
    PreferFirst,
    PreferSecond,
    MergeArrays,
    FailOnConflict,
}

impl JsonMerger {
    pub fn new(resolution: ConflictResolution) -> Self {
        JsonMerger {
            conflict_resolution: resolution,
        }
    }

    pub fn merge_files(&self, path1: &Path, path2: &Path) -> Result<Value, String> {
        let content1 = fs::read_to_string(path1)
            .map_err(|e| format!("Failed to read {}: {}", path1.display(), e))?;
        let content2 = fs::read_to_string(path2)
            .map_err(|e| format!("Failed to read {}: {}", path2.display(), e))?;

        let json1: Value = serde_json::from_str(&content1)
            .map_err(|e| format!("Invalid JSON in {}: {}", path1.display(), e))?;
        let json2: Value = serde_json::from_str(&content2)
            .map_err(|e| format!("Invalid JSON in {}: {}", path2.display(), e))?;

        self.merge_values(&json1, &json2)
    }

    fn merge_values(&self, val1: &Value, val2: &Value) -> Result<Value, String> {
        match (val1, val2) {
            (Value::Object(map1), Value::Object(map2)) => self.merge_objects(map1, map2),
            (Value::Array(arr1), Value::Array(arr2)) => self.merge_arrays(arr1, arr2),
            _ => self.resolve_leaf_conflict(val1, val2),
        }
    }

    fn merge_objects(&self, map1: &Map<String, Value>, map2: &Map<String, Value>) -> Result<Value, String> {
        let mut result = Map::new();
        let all_keys: HashSet<_> = map1.keys().chain(map2.keys()).collect();

        for key in all_keys {
            match (map1.get(key), map2.get(key)) {
                (Some(v1), Some(v2)) => {
                    let merged = self.merge_values(v1, v2)?;
                    result.insert(key.clone(), merged);
                }
                (Some(v), None) | (None, Some(v)) => {
                    result.insert(key.clone(), v.clone());
                }
                (None, None) => unreachable!(),
            }
        }

        Ok(Value::Object(result))
    }

    fn merge_arrays(&self, arr1: &[Value], arr2: &[Value]) -> Result<Value, String> {
        match self.conflict_resolution {
            ConflictResolution::MergeArrays => {
                let mut merged = Vec::with_capacity(arr1.len() + arr2.len());
                merged.extend_from_slice(arr1);
                merged.extend_from_slice(arr2);
                Ok(Value::Array(merged))
            }
            _ => self.resolve_leaf_conflict(&Value::Array(arr1.to_vec()), &Value::Array(arr2.to_vec())),
        }
    }

    fn resolve_leaf_conflict(&self, val1: &Value, val2: &Value) -> Result<Value, String> {
        if val1 == val2 {
            return Ok(val1.clone());
        }

        match self.conflict_resolution {
            ConflictResolution::PreferFirst => Ok(val1.clone()),
            ConflictResolution::PreferSecond => Ok(val2.clone()),
            ConflictResolution::FailOnConflict => Err(format!(
                "Conflict between values: {} and {}",
                val1, val2
            )),
            ConflictResolution::MergeArrays => Err(
                "MergeArrays strategy only applicable to array conflicts".to_string(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_objects_prefer_first() {
        let merger = JsonMerger::new(ConflictResolution::PreferFirst);
        let json1 = json!({"a": 1, "b": 2});
        let json2 = json!({"b": 99, "c": 3});

        let result = merger.merge_values(&json1, &json2).unwrap();
        assert_eq!(result, json!({"a": 1, "b": 2, "c": 3}));
    }

    #[test]
    fn test_merge_arrays_merge() {
        let merger = JsonMerger::new(ConflictResolution::MergeArrays);
        let json1 = json!([1, 2]);
        let json2 = json!([3, 4]);

        let result = merger.merge_values(&json1, &json2).unwrap();
        assert_eq!(result, json!([1, 2, 3, 4]));
    }
}