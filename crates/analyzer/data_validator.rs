use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValidationError {
    field: String,
    reason: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation failed for field '{}': {}", self.field, self.reason)
    }
}

impl Error for ValidationError {}

pub struct DataValidator {
    rules: Vec<ValidationRule>,
}

struct ValidationRule {
    field_name: String,
    validator: Box<dyn Fn(&str) -> Result<(), ValidationError>>,
}

impl DataValidator {
    pub fn new() -> Self {
        DataValidator {
            rules: Vec::new(),
        }
    }

    pub fn add_rule<F>(mut self, field_name: &str, validator: F) -> Self
    where
        F: Fn(&str) -> Result<(), ValidationError> + 'static,
    {
        self.rules.push(ValidationRule {
            field_name: field_name.to_string(),
            validator: Box::new(validator),
        });
        self
    }

    pub fn validate(&self, data: &[(&str, &str)]) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        for rule in &self.rules {
            if let Some((_, value)) = data.iter().find(|(field, _)| *field == rule.field_name) {
                if let Err(err) = (rule.validator)(value) {
                    errors.push(err);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub fn create_email_validator() -> DataValidator {
    DataValidator::new()
        .add_rule("email", |value| {
            if value.contains('@') && value.contains('.') {
                Ok(())
            } else {
                Err(ValidationError {
                    field: "email".to_string(),
                    reason: "Invalid email format".to_string(),
                })
            }
        })
        .add_rule("username", |value| {
            if value.len() >= 3 && value.len() <= 20 && value.chars().all(|c| c.is_alphanumeric()) {
                Ok(())
            } else {
                Err(ValidationError {
                    field: "username".to_string(),
                    reason: "Username must be 3-20 alphanumeric characters".to_string(),
                })
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_data() {
        let validator = create_email_validator();
        let data = vec![
            ("email", "test@example.com"),
            ("username", "user123"),
        ];

        assert!(validator.validate(&data).is_ok());
    }

    #[test]
    fn test_invalid_email() {
        let validator = create_email_validator();
        let data = vec![
            ("email", "invalid-email"),
            ("username", "user123"),
        ];

        let result = validator.validate(&data);
        assert!(result.is_err());
        if let Err(errors) = result {
            assert_eq!(errors.len(), 1);
            assert!(errors[0].to_string().contains("email"));
        }
    }

    #[test]
    fn test_invalid_username() {
        let validator = create_email_validator();
        let data = vec![
            ("email", "test@example.com"),
            ("username", "ab"),
        ];

        let result = validator.validate(&data);
        assert!(result.is_err());
    }
}