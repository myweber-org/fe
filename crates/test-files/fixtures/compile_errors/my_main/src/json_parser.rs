use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    username: String,
    email: String,
    active: bool,
}

pub fn parse_user_json(json_str: &str) -> Result<User> {
    let user: User = serde_json::from_str(json_str)?;
    Ok(user)
}

pub fn create_user_json(user: &User) -> Result<String> {
    let json = serde_json::to_string(user)?;
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_parsing() {
        let json_data = r#"
        {
            "id": 42,
            "username": "rustacean",
            "email": "user@example.com",
            "active": true
        }
        "#;

        let result = parse_user_json(json_data);
        assert!(result.is_ok());
        
        let user = result.unwrap();
        assert_eq!(user.id, 42);
        assert_eq!(user.username, "rustacean");
        assert_eq!(user.email, "user@example.com");
        assert!(user.active);
    }

    #[test]
    fn test_json_creation() {
        let user = User {
            id: 100,
            username: String::from("testuser"),
            email: String::from("test@example.com"),
            active: false,
        };

        let result = create_user_json(&user);
        assert!(result.is_ok());
        
        let json_str = result.unwrap();
        assert!(json_str.contains("\"id\":100"));
        assert!(json_str.contains("\"username\":\"testuser\""));
    }
}use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
enum JsonValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
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
    position: usize,
}

impl JsonParser {
    fn new(input: &str) -> Self {
        JsonParser {
            input: input.chars().collect(),
            position: 0,
        }
    }

    fn parse(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        let value = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
            return Err(self.error("Unexpected trailing characters"));
        }
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        match self.peek_char() {
            Some('"') => self.parse_string(),
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('t') | Some('f') => self.parse_boolean(),
            Some('n') => self.parse_null(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            _ => Err(self.error("Invalid JSON value")),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        self.expect_char('"')?;
        let mut result = String::new();
        
        while let Some(c) = self.next_char() {
            match c {
                '"' => return Ok(JsonValue::String(result)),
                '\\' => {
                    let escaped = self.next_char().ok_or_else(|| self.error("Unterminated escape sequence"))?;
                    match escaped {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        '/' => result.push('/'),
                        'b' => result.push('\x08'),
                        'f' => result.push('\x0c'),
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        'u' => {
                            let hex_code: String = (0..4)
                                .map(|_| self.next_char().ok_or_else(|| self.error("Invalid Unicode escape")))
                                .collect::<Result<_, _>>()?;
                            let code_point = u32::from_str_radix(&hex_code, 16)
                                .map_err(|_| self.error("Invalid hex code"))?;
                            result.push(char::from_u32(code_point).ok_or_else(|| self.error("Invalid Unicode code point"))?);
                        }
                        _ => return Err(self.error("Invalid escape character")),
                    }
                }
                _ => result.push(c),
            }
        }
        
        Err(self.error("Unterminated string"))
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.position;
        let mut has_decimal = false;
        let mut has_exponent = false;
        
        if self.peek_char() == Some('-') {
            self.next_char();
        }
        
        if self.peek_char() == Some('0') {
            self.next_char();
            if self.peek_char().map_or(false, |c| c.is_digit(10)) {
                return Err(self.error("Leading zeros are not allowed"));
            }
        } else {
            while self.peek_char().map_or(false, |c| c.is_digit(10)) {
                self.next_char();
            }
        }
        
        if self.peek_char() == Some('.') {
            has_decimal = true;
            self.next_char();
            if !self.peek_char().map_or(false, |c| c.is_digit(10)) {
                return Err(self.error("Expected digit after decimal point"));
            }
            while self.peek_char().map_or(false, |c| c.is_digit(10)) {
                self.next_char();
            }
        }
        
        if self.peek_char() == Some('e') || self.peek_char() == Some('E') {
            has_exponent = true;
            self.next_char();
            if self.peek_char() == Some('+') || self.peek_char() == Some('-') {
                self.next_char();
            }
            if !self.peek_char().map_or(false, |c| c.is_digit(10)) {
                return Err(self.error("Expected digit in exponent"));
            }
            while self.peek_char().map_or(false, |c| c.is_digit(10)) {
                self.next_char();
            }
        }
        
        let number_str: String = self.input[start..self.position].iter().collect();
        let number = number_str.parse::<f64>()
            .map_err(|_| self.error("Invalid number format"))?;
        
        Ok(JsonValue::Number(number))
    }

    fn parse_boolean(&mut self) -> Result<JsonValue, ParseError> {
        if self.consume_str("true") {
            Ok(JsonValue::Boolean(true))
        } else if self.consume_str("false") {
            Ok(JsonValue::Boolean(false))
        } else {
            Err(self.error("Invalid boolean value"))
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        if self.consume_str("null") {
            Ok(JsonValue::Null)
        } else {
            Err(self.error("Invalid null value"))
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        self.expect_char('{')?;
        self.skip_whitespace();
        
        let mut map = HashMap::new();
        
        if self.peek_char() == Some('}') {
            self.next_char();
            return Ok(JsonValue::Object(map));
        }
        
        loop {
            self.skip_whitespace();
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => unreachable!(),
            };
            
            self.skip_whitespace();
            self.expect_char(':')?;
            
            let value = self.parse_value()?;
            map.insert(key, value);
            
            self.skip_whitespace();
            match self.peek_char() {
                Some(',') => {
                    self.next_char();
                    self.skip_whitespace();
                    if self.peek_char() == Some('}') {
                        return Err(self.error("Trailing comma in object"));
                    }
                }
                Some('}') => {
                    self.next_char();
                    break;
                }
                _ => return Err(self.error("Expected ',' or '}' in object")),
            }
        }
        
        Ok(JsonValue::Object(map))
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.expect_char('[')?;
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
        if self.peek_char() == Some(']') {
            self.next_char();
            return Ok(JsonValue::Array(array));
        }
        
        loop {
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            match self.peek_char() {
                Some(',') => {
                    self.next_char();
                    self.skip_whitespace();
                    if self.peek_char() == Some(']') {
                        return Err(self.error("Trailing comma in array"));
                    }
                }
                Some(']') => {
                    self.next_char();
                    break;
                }
                _ => return Err(self.error("Expected ',' or ']' in array")),
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.peek_char();
        if c.is_some() {
            self.position += 1;
        }
        c
    }

    fn expect_char(&mut self, expected: char) -> Result<(), ParseError> {
        match self.next_char() {
            Some(c) if c == expected => Ok(()),
            _ => Err(self.error(&format!("Expected '{}'", expected))),
        }
    }

    fn consume_str(&mut self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        if self.position + chars.len() <= self.input.len() {
            for (i, &c) in chars.iter().enumerate() {
                if self.input[self.position + i] != c {
                    return false;
                }
            }
            self.position += chars.len();
            true
        } else {
            false
        }
    }

    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            position: self.position,
        }
    }
}

pub fn parse_json(json_str: &str) -> Result<JsonValue, ParseError> {
    let mut parser = JsonParser::new(json_str);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_object() {
        let json = r#"{"name": "test", "value": 42}"#;
        let result = parse_json(json);
        assert!(result.is_ok());
        
        if let Ok(JsonValue::Object(map)) = result {
            assert_eq!(map.get("name"), Some(&JsonValue::String("test".to_string())));
            assert_eq!(map.get("value"), Some(&JsonValue::Number(42.0)));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_array() {
        let json = r#"[1, 2, 3, true, false, null]"#;
        let result = parse_json(json);
        assert!(result.is_ok());
        
        if let Ok(JsonValue::Array(arr)) = result {
            assert_eq!(arr.len(), 6);
            assert_eq!(arr[0], JsonValue::Number(1.0));
            assert_eq!(arr[3], JsonValue::Boolean(true));
            assert_eq!(arr[5], JsonValue::Null);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_parse_nested_structure() {
        let json = r#"{
            "users": [
                {"id": 1, "active": true},
                {"id": 2, "active": false}
            ],
            "metadata": {"version": "1.0"}
        }"#;
        
        let result = parse_json(json);
        assert!(result.is_ok());
    }
}