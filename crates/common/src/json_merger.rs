
use serde_json::{Value, Map};
use std::collections::HashSet;

pub fn merge_json(base: &mut Value, update: &Value, strategy: MergeStrategy) -> Result<(), String> {
    match (base, update) {
        (Value::Object(base_map), Value::Object(update_map)) => {
            for (key, update_value) in update_map {
                if base_map.contains_key(key) {
                    let base_value = base_map.get_mut(key).unwrap();
                    match strategy {
                        MergeStrategy::Overwrite => {
                            *base_value = update_value.clone();
                        }
                        MergeStrategy::Recursive => {
                            merge_json(base_value, update_value, strategy)?;
                        }
                        MergeStrategy::Skip => {}
                        MergeStrategy::CombineArrays => {
                            if let (Value::Array(base_arr), Value::Array(update_arr)) = (base_value, update_value) {
                                let mut combined = base_arr.clone();
                                combined.extend_from_slice(update_arr);
                                *base_value = Value::Array(combined);
                            } else {
                                merge_json(base_value, update_value, MergeStrategy::Recursive)?;
                            }
                        }
                    }
                } else {
                    base_map.insert(key.clone(), update_value.clone());
                }
            }
            Ok(())
        }
        _ => Err("Both values must be JSON objects".to_string()),
    }
}

pub fn merge_json_with_conflict_list(
    base: &mut Value,
    update: &Value,
    strategy: MergeStrategy,
) -> Result<HashSet<String>, String> {
    let mut conflicts = HashSet::new();
    
    if let (Value::Object(base_map), Value::Object(update_map)) = (base, update) {
        for (key, update_value) in update_map {
            if base_map.contains_key(key) {
                conflicts.insert(key.clone());
                let base_value = base_map.get_mut(key).unwrap();
                match strategy {
                    MergeStrategy::Overwrite => {
                        *base_value = update_value.clone();
                    }
                    MergeStrategy::Recursive => {
                        if let Err(e) = merge_json(base_value, update_value, strategy) {
                            return Err(e);
                        }
                    }
                    MergeStrategy::Skip => {}
                    MergeStrategy::CombineArrays => {
                        if let (Value::Array(base_arr), Value::Array(update_arr)) = (base_value, update_value) {
                            let mut combined = base_arr.clone();
                            combined.extend_from_slice(update_arr);
                            *base_value = Value::Array(combined);
                        } else if let Err(e) = merge_json(base_value, update_value, MergeStrategy::Recursive) {
                            return Err(e);
                        }
                    }
                }
            } else {
                base_map.insert(key.clone(), update_value.clone());
            }
        }
        Ok(conflicts)
    } else {
        Err("Both values must be JSON objects".to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MergeStrategy {
    Overwrite,
    Recursive,
    Skip,
    CombineArrays,
}

pub fn deep_merge(target: &mut Value, source: &Value) {
    match (target, source) {
        (Value::Object(t), Value::Object(s)) => {
            for (key, value) in s {
                if !t.contains_key(key) {
                    t.insert(key.clone(), value.clone());
                } else {
                    deep_merge(t.get_mut(key).unwrap(), value);
                }
            }
        }
        (t, s) => {
            *t = s.clone();
        }
    }
}