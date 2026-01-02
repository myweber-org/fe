
use serde_json::{Value, Map};
use std::collections::HashSet;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ValidationError {
    MissingField(String),
    TypeMismatch(String, String),
    InvalidValue(String, String),
    SchemaError(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::MissingField(field) => write!(f, "Missing required field: {}", field),
            ValidationError::TypeMismatch(field, expected) => 
                write!(f, "Field '{}' must be of type: {}", field, expected),
            ValidationError::InvalidValue(field, reason) => 
                write!(f, "Invalid value for field '{}': {}", field, reason),
            ValidationError::SchemaError(msg) => write!(f, "Schema error: {}", msg),
        }
    }
}

impl Error for ValidationError {}

#[derive(Debug, Clone)]
pub struct FieldSchema {
    pub required: bool,
    pub field_type: FieldType,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone)]
pub enum FieldType {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Null,
}

#[derive(Debug, Clone)]
pub enum Constraint {
    MinLength(usize),
    MaxLength(usize),
    MinValue(f64),
    MaxValue(f64),
    Pattern(String),
    Enum(Vec<Value>),
}

pub struct JsonValidator {
    schema: Map<String, Value>,
    field_schemas: Map<String, FieldSchema>,
}

impl JsonValidator {
    pub fn new(schema: Value) -> Result<Self, ValidationError> {
        let schema_map = match schema {
            Value::Object(map) => map,
            _ => return Err(ValidationError::SchemaError("Schema must be a JSON object".to_string())),
        };

        let mut field_schemas = Map::new();
        
        for (key, value) in schema_map.iter() {
            let field_schema = Self::parse_field_schema(key, value)?;
            field_schemas.insert(key.clone(), field_schema);
        }

        Ok(Self {
            schema: schema_map,
            field_schemas,
        })
    }

    fn parse_field_schema(field_name: &str, value: &Value) -> Result<FieldSchema, ValidationError> {
        let obj = match value {
            Value::Object(map) => map,
            _ => return Err(ValidationError::SchemaError(
                format!("Field '{}' schema must be an object", field_name)
            )),
        };

        let required = obj.get("required")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let field_type = match obj.get("type").and_then(|v| v.as_str()) {
            Some("string") => FieldType::String,
            Some("number") => FieldType::Number,
            Some("boolean") => FieldType::Boolean,
            Some("object") => FieldType::Object,
            Some("array") => FieldType::Array,
            Some("null") => FieldType::Null,
            Some(t) => return Err(ValidationError::SchemaError(
                format!("Invalid type '{}' for field '{}'", t, field_name)
            )),
            None => return Err(ValidationError::SchemaError(
                format!("Missing type for field '{}'", field_name)
            )),
        };

        let mut constraints = Vec::new();
        
        if let Some(min_len) = obj.get("minLength").and_then(|v| v.as_u64()) {
            constraints.push(Constraint::MinLength(min_len as usize));
        }
        
        if let Some(max_len) = obj.get("maxLength").and_then(|v| v.as_u64()) {
            constraints.push(Constraint::MaxLength(max_len as usize));
        }
        
        if let Some(min_val) = obj.get("minimum").and_then(|v| v.as_f64()) {
            constraints.push(Constraint::MinValue(min_val));
        }
        
        if let Some(max_val) = obj.get("maximum").and_then(|v| v.as_f64()) {
            constraints.push(Constraint::MaxValue(max_val));
        }
        
        if let Some(pattern) = obj.get("pattern").and_then(|v| v.as_str()) {
            constraints.push(Constraint::Pattern(pattern.to_string()));
        }
        
        if let Some(enum_vals) = obj.get("enum").and_then(|v| v.as_array()) {
            constraints.push(Constraint::Enum(enum_vals.clone()));
        }

        Ok(FieldSchema {
            required,
            field_type,
            constraints,
        })
    }

    pub fn validate(&self, data: &Value) -> Result<(), Vec<ValidationError>> {
        let data_map = match data {
            Value::Object(map) => map,
            _ => return Err(vec![ValidationError::SchemaError(
                "Data must be a JSON object".to_string()
            )]),
        };

        let mut errors = Vec::new();
        let mut validated_fields = HashSet::new();

        for (field_name, field_schema) in self.field_schemas.iter() {
            match data_map.get(field_name) {
                Some(value) => {
                    validated_fields.insert(field_name.clone());
                    if let Err(err) = self.validate_field(field_name, value, field_schema) {
                        errors.push(err);
                    }
                }
                None => {
                    if field_schema.required {
                        errors.push(ValidationError::MissingField(field_name.clone()));
                    }
                }
            }
        }

        for field_name in data_map.keys() {
            if !self.field_schemas.contains_key(field_name) && !validated_fields.contains(field_name) {
                errors.push(ValidationError::SchemaError(
                    format!("Unexpected field: {}", field_name)
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_field(&self, field_name: &str, value: &Value, schema: &FieldSchema) -> Result<(), ValidationError> {
        match (&schema.field_type, value) {
            (FieldType::String, Value::String(s)) => {
                for constraint in &schema.constraints {
                    match constraint {
                        Constraint::MinLength(min) if s.len() < *min => {
                            return Err(ValidationError::InvalidValue(
                                field_name.to_string(),
                                format!("String length {} is less than minimum {}", s.len(), min)
                            ));
                        }
                        Constraint::MaxLength(max) if s.len() > *max => {
                            return Err(ValidationError::InvalidValue(
                                field_name.to_string(),
                                format!("String length {} exceeds maximum {}", s.len(), max)
                            ));
                        }
                        Constraint::Pattern(pattern) => {
                            let re = regex::Regex::new(pattern).map_err(|_| 
                                ValidationError::SchemaError(format!("Invalid regex pattern for field '{}'", field_name))
                            )?;
                            if !re.is_match(s) {
                                return Err(ValidationError::InvalidValue(
                                    field_name.to_string(),
                                    format!("String does not match pattern: {}", pattern)
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }
            (FieldType::Number, Value::Number(n)) => {
                if let Some(num) = n.as_f64() {
                    for constraint in &schema.constraints {
                        match constraint {
                            Constraint::MinValue(min) if num < *min => {
                                return Err(ValidationError::InvalidValue(
                                    field_name.to_string(),
                                    format!("Value {} is less than minimum {}", num, min)
                                ));
                            }
                            Constraint::MaxValue(max) if num > *max => {
                                return Err(ValidationError::InvalidValue(
                                    field_name.to_string(),
                                    format!("Value {} exceeds maximum {}", num, max)
                                ));
                            }
                            _ => {}
                        }
                    }
                }
            }
            (FieldType::Boolean, Value::Bool(_)) => {}
            (FieldType::Object, Value::Object(_)) => {}
            (FieldType::Array, Value::Array(_)) => {}
            (FieldType::Null, Value::Null) => {}
            (expected_type, actual_value) => {
                return Err(ValidationError::TypeMismatch(
                    field_name.to_string(),
                    format!("Expected {:?}, got {:?}", expected_type, actual_value)
                ));
            }
        }

        if let Some(Constraint::Enum(allowed_values)) = schema.constraints.iter().find(|c| matches!(c, Constraint::Enum(_))) {
            if let Constraint::Enum(vals) = allowed_values {
                if !vals.contains(value) {
                    return Err(ValidationError::InvalidValue(
                        field_name.to_string(),
                        "Value not in allowed enum values".to_string()
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_validation() {
        let schema = json!({
            "name": {
                "type": "string",
                "required": true,
                "minLength": 2,
                "maxLength": 50
            },
            "age": {
                "type": "number",
                "required": false,
                "minimum": 0,
                "maximum": 150
            },
            "active": {
                "type": "boolean",
                "required": true
            }
        });

        let validator = JsonValidator::new(schema).unwrap();
        
        let valid_data = json!({
            "name": "John Doe",
            "age": 30,
            "active": true
        });
        
        assert!(validator.validate(&valid_data).is_ok());
        
        let invalid_data = json!({
            "name": "J",
            "age": 200,
            "active": "yes"
        });
        
        let errors = validator.validate(&invalid_data).unwrap_err();
        assert_eq!(errors.len(), 3);
    }
}