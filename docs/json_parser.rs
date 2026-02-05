use serde_json::{Value, json};
use std::fs;

pub fn parse_json_file(file_path: &str) -> Result<Value, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let json_value: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    
    Ok(json_value)
}

pub fn validate_json_structure(value: &Value, expected_keys: &[&str]) -> bool {
    if let Value::Object(map) = value {
        expected_keys.iter().all(|key| map.contains_key(*key))
    } else {
        false
    }
}

pub fn pretty_print_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| String::from("Invalid JSON"))
}

pub fn create_sample_json() -> Value {
    json!({
        "name": "Data Processor",
        "version": "1.0.0",
        "active": true,
        "features": ["parsing", "validation", "formatting"],
        "metadata": {
            "author": "System",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_json_parsing() {
        let sample = create_sample_json();
        let pretty = pretty_print_json(&sample);
        assert!(pretty.contains("Data Processor"));
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), &pretty).unwrap();
        
        let parsed = parse_json_file(temp_file.path().to_str().unwrap()).unwrap();
        assert!(validate_json_structure(&parsed, &["name", "version", "active"]));
    }
}use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedCharacter(char, usize),
    UnexpectedEndOfInput,
    InvalidNumber,
    InvalidEscapeSequence,
    TrailingComma,
    MissingColon,
    KeyMustBeString,
}

pub struct JsonParser {
    input: Vec<char>,
    position: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Self {
        JsonParser {
            input: input.chars().collect(),
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
            return Err(ParseError::UnexpectedCharacter(
                self.input[self.position],
                self.position,
            ));
        }
        Ok(result)
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        match self.peek_char() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            _ => Err(ParseError::UnexpectedEndOfInput),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("null")?;
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, ParseError> {
        if self.starts_with("true") {
            self.advance(4);
            Ok(JsonValue::Bool(true))
        } else if self.starts_with("false") {
            self.advance(5);
            Ok(JsonValue::Bool(false))
        } else {
            Err(ParseError::UnexpectedCharacter(
                self.input[self.position],
                self.position,
            ))
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.position;
        if self.peek_char() == Some('-') {
            self.advance(1);
        }

        while let Some(c) = self.peek_char() {
            if c.is_digit(10) {
                self.advance(1);
            } else {
                break;
            }
        }

        if self.peek_char() == Some('.') {
            self.advance(1);
            while let Some(c) = self.peek_char() {
                if c.is_digit(10) {
                    self.advance(1);
                } else {
                    break;
                }
            }
        }

        let number_str: String = self.input[start..self.position].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(ParseError::InvalidNumber),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        self.advance(1); // Skip opening quote
        let mut result = String::new();

        while let Some(c) = self.next_char() {
            match c {
                '"' => return Ok(JsonValue::String(result)),
                '\\' => {
                    let escaped = self.parse_escape_sequence()?;
                    result.push(escaped);
                }
                _ => result.push(c),
            }
        }

        Err(ParseError::UnexpectedEndOfInput)
    }

    fn parse_escape_sequence(&mut self) -> Result<char, ParseError> {
        match self.next_char() {
            Some('"') => Ok('"'),
            Some('\\') => Ok('\\'),
            Some('/') => Ok('/'),
            Some('b') => Ok('\x08'),
            Some('f') => Ok('\x0c'),
            Some('n') => Ok('\n'),
            Some('r') => Ok('\r'),
            Some('t') => Ok('\t'),
            _ => Err(ParseError::InvalidEscapeSequence),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.advance(1); // Skip '['
        self.skip_whitespace();
        let mut array = Vec::new();

        if self.peek_char() == Some(']') {
            self.advance(1);
            return Ok(JsonValue::Array(array));
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();

            match self.peek_char() {
                Some(',') => {
                    self.advance(1);
                    self.skip_whitespace();
                    if self.peek_char() == Some(']') {
                        return Err(ParseError::TrailingComma);
                    }
                }
                Some(']') => {
                    self.advance(1);
                    break;
                }
                _ => return Err(ParseError::UnexpectedEndOfInput),
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        self.advance(1); // Skip '{'
        self.skip_whitespace();
        let mut map = HashMap::new();

        if self.peek_char() == Some('}') {
            self.advance(1);
            return Ok(JsonValue::Object(map));
        }

        loop {
            self.skip_whitespace();
            let key = match self.parse_value()? {
                JsonValue::String(s) => s,
                _ => return Err(ParseError::KeyMustBeString),
            };

            self.skip_whitespace();
            if self.peek_char() != Some(':') {
                return Err(ParseError::MissingColon);
            }
            self.advance(1);

            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();

            match self.peek_char() {
                Some(',') => {
                    self.advance(1);
                    self.skip_whitespace();
                    if self.peek_char() == Some('}') {
                        return Err(ParseError::TrailingComma);
                    }
                }
                Some('}') => {
                    self.advance(1);
                    break;
                }
                _ => return Err(ParseError::UnexpectedEndOfInput),
            }
        }

        Ok(JsonValue::Object(map))
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.advance(1);
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn next_char(&mut self) -> Option<char> {
        if self.position < self.input.len() {
            let c = self.input[self.position];
            self.position += 1;
            Some(c)
        } else {
            None
        }
    }

    fn advance(&mut self, n: usize) {
        self.position += n;
    }

    fn starts_with(&self, s: &str) -> bool {
        let end = self.position + s.len();
        if end > self.input.len() {
            return false;
        }
        self.input[self.position..end]
            .iter()
            .collect::<String>()
            == s
    }

    fn expect(&mut self, expected: &str) -> Result<(), ParseError> {
        if self.starts_with(expected) {
            self.advance(expected.len());
            Ok(())
        } else {
            Err(ParseError::UnexpectedCharacter(
                self.input[self.position],
                self.position,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        let mut parser = JsonParser::new("null");
        assert_eq!(parser.parse(), Ok(JsonValue::Null));
    }

    #[test]
    fn test_parse_bool() {
        let mut parser = JsonParser::new("true");
        assert_eq!(parser.parse(), Ok(JsonValue::Bool(true)));

        let mut parser = JsonParser::new("false");
        assert_eq!(parser.parse(), Ok(JsonValue::Bool(false)));
    }

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(42.0)));

        let mut parser = JsonParser::new("-3.14");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(-3.14)));
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello world""#);
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::String("hello world".to_string()))
        );

        let mut parser = JsonParser::new(r#""escape\nsequence""#);
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::String("escape\nsequence".to_string()))
        );
    }

    #[test]
    fn test_parse_array() {
        let mut parser = JsonParser::new("[1, 2, 3]");
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Number(2.0),
                JsonValue::Number(3.0),
            ]))
        );
    }

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new(r#"{"key": "value"}"#);
        let mut map = HashMap::new();
        map.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(map)));
    }
}