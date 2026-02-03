use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    name: String,
    age: u8,
    email: String,
}

pub fn parse_user_json(json_str: &str) -> Result<User> {
    let user: User = serde_json::from_str(json_str)?;
    Ok(user)
}

pub fn extract_field(json_str: &str, field: &str) -> Result<String> {
    let v: Value = serde_json::from_str(json_str)?;
    
    match v.get(field) {
        Some(value) => {
            if value.is_string() {
                Ok(value.as_str().unwrap().to_string())
            } else {
                Ok(value.to_string())
            }
        }
        None => Err(serde_json::Error::custom(format!("Field '{}' not found", field))),
    }
}

pub fn validate_json_schema(json_str: &str) -> bool {
    serde_json::from_str::<Value>(json_str).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_user() {
        let json = r#"{"name":"Alice","age":30,"email":"alice@example.com"}"#;
        let result = parse_user_json(json);
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.name, "Alice");
        assert_eq!(user.age, 30);
    }

    #[test]
    fn test_extract_existing_field() {
        let json = r#"{"name":"Bob","age":25}"#;
        let result = extract_field(json, "name");
        assert_eq!(result.unwrap(), "Bob");
    }

    #[test]
    fn test_validate_correct_json() {
        let json = r#"{"key":"value"}"#;
        assert!(validate_json_schema(json));
    }
}use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

#[derive(Debug)]
struct ParseError {
    message: String,
    position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at position {}: {}", self.position, self.message)
    }
}

impl Error for ParseError {}

struct JsonParser {
    input: Vec<char>,
    pos: usize,
}

impl JsonParser {
    fn new(input: &str) -> Self {
        JsonParser {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn parse(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        let value = self.parse_value()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err(self.error("Unexpected trailing characters"));
        }
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        match self.peek_char() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            _ => Err(self.error("Invalid JSON value")),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("null")?;
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, ParseError> {
        if self.consume("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err(self.error("Expected boolean"))
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.pos;
        if self.consume_char('-') {
            // consume minus sign
        }
        while let Some(c) = self.peek_char() {
            if c.is_digit(10) {
                self.consume_char(c);
            } else {
                break;
            }
        }
        if self.consume_char('.') {
            while let Some(c) = self.peek_char() {
                if c.is_digit(10) {
                    self.consume_char(c);
                } else {
                    break;
                }
            }
        }
        let num_str: String = self.input[start..self.pos].iter().collect();
        match num_str.parse() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(self.error("Invalid number")),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        self.expect_char('"')?;
        let mut result = String::new();
        while let Some(c) = self.next_char() {
            if c == '"' {
                break;
            } else if c == '\\' {
                let escaped = self.next_char().ok_or_else(|| self.error("Unterminated string"))?;
                match escaped {
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    '/' => result.push('/'),
                    'b' => result.push('\x08'),
                    'f' => result.push('\x0c'),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    _ => return Err(self.error("Invalid escape sequence")),
                }
            } else {
                result.push(c);
            }
        }
        Ok(JsonValue::String(result))
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.expect_char('[')?;
        self.skip_whitespace();
        let mut array = Vec::new();
        if self.consume_char(']') {
            return Ok(JsonValue::Array(array));
        }
        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();
            if self.consume_char(']') {
                break;
            }
            self.expect_char(',')?;
            self.skip_whitespace();
        }
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        self.expect_char('{')?;
        self.skip_whitespace();
        let mut map = HashMap::new();
        if self.consume_char('}') {
            return Ok(JsonValue::Object(map));
        }
        loop {
            let key = match self.parse_value()? {
                JsonValue::String(s) => s,
                _ => return Err(self.error("Object key must be a string")),
            };
            self.skip_whitespace();
            self.expect_char(':')?;
            self.skip_whitespace();
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            if self.consume_char('}') {
                break;
            }
            self.expect_char(',')?;
            self.skip_whitespace();
        }
        Ok(JsonValue::Object(map))
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.consume_char(c);
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.peek_char();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<(), ParseError> {
        if self.consume_char(expected) {
            Ok(())
        } else {
            Err(self.error(&format!("Expected '{}'", expected)))
        }
    }

    fn consume(&mut self, expected: &str) -> bool {
        let expected_chars: Vec<char> = expected.chars().collect();
        if self.pos + expected_chars.len() <= self.input.len() {
            for (i, &c) in expected_chars.iter().enumerate() {
                if self.input[self.pos + i] != c {
                    return false;
                }
            }
            self.pos += expected_chars.len();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: &str) -> Result<(), ParseError> {
        if self.consume(expected) {
            Ok(())
        } else {
            Err(self.error(&format!("Expected '{}'", expected)))
        }
    }

    fn error(&self, msg: &str) -> ParseError {
        ParseError {
            message: msg.to_string(),
            position: self.pos,
        }
    }
}

fn parse_json(json_str: &str) -> Result<JsonValue, ParseError> {
    let mut parser = JsonParser::new(json_str);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        assert_eq!(parse_json("null").unwrap(), JsonValue::Null);
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse_json("true").unwrap(), JsonValue::Bool(true));
        assert_eq!(parse_json("false").unwrap(), JsonValue::Bool(false));
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_json("42").unwrap(), JsonValue::Number(42.0));
        assert_eq!(parse_json("-3.14").unwrap(), JsonValue::Number(-3.14));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse_json(r#""hello""#).unwrap(),
            JsonValue::String("hello".to_string())
        );
        assert_eq!(
            parse_json(r#""escape\"test""#).unwrap(),
            JsonValue::String("escape\"test".to_string())
        );
    }

    #[test]
    fn test_parse_array() {
        let json = r#"[1, true, "hello"]"#;
        let result = parse_json(json).unwrap();
        if let JsonValue::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], JsonValue::Number(1.0));
            assert_eq!(arr[1], JsonValue::Bool(true));
            assert_eq!(arr[2], JsonValue::String("hello".to_string()));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_parse_object() {
        let json = r#"{"key": "value", "num": 42}"#;
        let result = parse_json(json).unwrap();
        if let JsonValue::Object(map) = result {
            assert_eq!(map.len(), 2);
            assert_eq!(map.get("key"), Some(&JsonValue::String("value".to_string())));
            assert_eq!(map.get("num"), Some(&JsonValue::Number(42.0)));
        } else {
            panic!("Expected object");
        }
    }
}