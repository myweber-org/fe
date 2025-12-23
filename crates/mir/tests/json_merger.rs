use serde_json::{Map, Value};

pub fn merge_json(a: &mut Value, b: &Value) {
    match (a, b) {
        (Value::Object(a_map), Value::Object(b_map)) => {
            for (key, b_value) in b_map {
                if let Some(a_value) = a_map.get_mut(key) {
                    merge_json(a_value, b_value);
                } else {
                    a_map.insert(key.clone(), b_value.clone());
                }
            }
        }
        (a, b) => *a = b.clone(),
    }
}

pub fn merge_json_with_strategy(a: &mut Value, b: &Value, strategy: MergeStrategy) {
    match (a, b) {
        (Value::Object(a_map), Value::Object(b_map)) => {
            for (key, b_value) in b_map {
                if let Some(a_value) = a_map.get_mut(key) {
                    merge_json_with_strategy(a_value, b_value, strategy.clone());
                } else {
                    a_map.insert(key.clone(), b_value.clone());
                }
            }
        }
        (Value::Array(a_arr), Value::Array(b_arr)) => {
            match strategy {
                MergeStrategy::Replace => *a_arr = b_arr.clone(),
                MergeStrategy::Append => a_arr.extend(b_arr.clone()),
                MergeStrategy::Merge => {
                    for (i, b_item) in b_arr.iter().enumerate() {
                        if i < a_arr.len() {
                            merge_json_with_strategy(&mut a_arr[i], b_item, strategy.clone());
                        } else {
                            a_arr.push(b_item.clone());
                        }
                    }
                }
            }
        }
        (a, b) => *a = b.clone(),
    }
}

#[derive(Clone)]
pub enum MergeStrategy {
    Replace,
    Append,
    Merge,
}