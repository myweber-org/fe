use std::collections::HashMap;
use std::error::Error;
use std::fmt;

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
pub struct ParseError {
    message: String,
    position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON parse error at position {}: {}", self.position, self.message)
    }
}

impl Error for ParseError {}

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
        let value = self.parse_value()?;
        self.skip_whitespace();
        
        if self.position < self.input.len() {
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
            _ => Err(self.error("Expected JSON value")),
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
            Err(self.error("Expected boolean value"))
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.position;
        let mut has_decimal = false;
        
        if self.peek_char() == Some('-') {
            self.advance(1);
        }
        
        while let Some(c) = self.peek_char() {
            if c.is_digit(10) {
                self.advance(1);
            } else if c == '.' && !has_decimal {
                has_decimal = true;
                self.advance(1);
            } else {
                break;
            }
        }
        
        let number_str: String = self.input[start..self.position].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(self.error("Invalid number format")),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("\"")?;
        let mut result = String::new();
        
        while let Some(c) = self.next_char() {
            match c {
                '"' => break,
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
                        _ => return Err(self.error("Invalid escape sequence")),
                    }
                }
                _ => result.push(c),
            }
        }
        
        Ok(JsonValue::String(result))
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("[")?;
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
                }
                Some(']') => {
                    self.advance(1);
                    break;
                }
                _ => return Err(self.error("Expected ',' or ']' in array")),
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("{")?;
        self.skip_whitespace();
        
        let mut object = HashMap::new();
        
        if self.peek_char() == Some('}') {
            self.advance(1);
            return Ok(JsonValue::Object(object));
        }
        
        loop {
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => unreachable!(),
            };
            
            self.skip_whitespace();
            self.expect(":")?;
            self.skip_whitespace();
            
            let value = self.parse_value()?;
            object.insert(key, value);
            
            self.skip_whitespace();
            match self.peek_char() {
                Some(',') => {
                    self.advance(1);
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.advance(1);
                    break;
                }
                _ => return Err(self.error("Expected ',' or '}' in object")),
            }
        }
        
        Ok(JsonValue::Object(object))
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
        let c = self.peek_char();
        if c.is_some() {
            self.position += 1;
        }
        c
    }

    fn advance(&mut self, n: usize) {
        self.position += n;
    }

    fn starts_with(&self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        self.position + chars.len() <= self.input.len() 
            && self.input[self.position..self.position + chars.len()] == chars
    }

    fn expect(&mut self, expected: &str) -> Result<(), ParseError> {
        if self.starts_with(expected) {
            self.advance(expected.len());
            Ok(())
        } else {
            Err(self.error(&format!("Expected '{}'", expected)))
        }
    }

    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            position: self.position,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        let mut parser = JsonParser::new("null");
        assert_eq!(parser.parse().unwrap(), JsonValue::Null);
    }

    #[test]
    fn test_parse_bool() {
        let mut parser = JsonParser::new("true");
        assert_eq!(parser.parse().unwrap(), JsonValue::Bool(true));
        
        let mut parser = JsonParser::new("false");
        assert_eq!(parser.parse().unwrap(), JsonValue::Bool(false));
    }

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42");
        assert_eq!(parser.parse().unwrap(), JsonValue::Number(42.0));
        
        let mut parser = JsonParser::new("-3.14");
        assert_eq!(parser.parse().unwrap(), JsonValue::Number(-3.14));
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new("\"hello world\"");
        assert_eq!(parser.parse().unwrap(), JsonValue::String("hello world".to_string()));
        
        let mut parser = JsonParser::new("\"escaped \\\"quote\\\"\"");
        assert_eq!(parser.parse().unwrap(), JsonValue::String("escaped \"quote\"".to_string()));
    }

    #[test]
    fn test_parse_array() {
        let mut parser = JsonParser::new("[1, 2, 3]");
        let expected = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);
        assert_eq!(parser.parse().unwrap(), expected);
    }

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new("{\"key\": \"value\", \"number\": 42}");
        let mut expected_map = HashMap::new();
        expected_map.insert("key".to_string(), JsonValue::String("value".to_string()));
        expected_map.insert("number".to_string(), JsonValue::Number(42.0));
        let expected = JsonValue::Object(expected_map);
        assert_eq!(parser.parse().unwrap(), expected);
    }

    #[test]
    fn test_parse_error() {
        let mut parser = JsonParser::new("invalid");
        assert!(parser.parse().is_err());
        
        let mut parser = JsonParser::new("[1, 2");
        assert!(parser.parse().is_err());
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

pub struct JsonParser {
    input: Vec<char>,
    pos: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Self {
        JsonParser {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn consume(&mut self, expected: char) -> Result<(), String> {
        self.skip_whitespace();
        if let Some(ch) = self.peek() {
            if ch == expected {
                self.pos += 1;
                Ok(())
            } else {
                Err(format!("Expected '{}', found '{}'", expected, ch))
            }
        } else {
            Err("Unexpected end of input".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.consume('"')?;
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                break;
            }
            result.push(ch);
            self.pos += 1;
        }
        self.consume('"')?;
        Ok(JsonValue::String(result))
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_digit(10) || ch == '.' || ch == '-' || ch == 'e' || ch == 'E' {
                self.pos += 1;
            } else {
                break;
            }
        }
        let num_str: String = self.input[start..self.pos].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(format!("Invalid number: {}", num_str)),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume('[')?;
        let mut array = Vec::new();
        self.skip_whitespace();
        if let Some(ch) = self.peek() {
            if ch == ']' {
                self.pos += 1;
                return Ok(JsonValue::Array(array));
            }
        }
        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                if ch == ']' {
                    self.pos += 1;
                    break;
                } else if ch == ',' {
                    self.pos += 1;
                    self.skip_whitespace();
                } else {
                    return Err(format!("Expected ',' or ']', found '{}'", ch));
                }
            } else {
                return Err("Unexpected end of input".to_string());
            }
        }
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume('{')?;
        let mut map = HashMap::new();
        self.skip_whitespace();
        if let Some(ch) = self.peek() {
            if ch == '}' {
                self.pos += 1;
                return Ok(JsonValue::Object(map));
            }
        }
        loop {
            let key = match self.parse_value()? {
                JsonValue::String(s) => s,
                _ => return Err("Object key must be a string".to_string()),
            };
            self.skip_whitespace();
            self.consume(':')?;
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                if ch == '}' {
                    self.pos += 1;
                    break;
                } else if ch == ',' {
                    self.pos += 1;
                    self.skip_whitespace();
                } else {
                    return Err(format!("Expected ',' or '}}', found '{}'", ch));
                }
            } else {
                return Err("Unexpected end of input".to_string());
            }
        }
        Ok(JsonValue::Object(map))
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        if let Some(ch) = self.peek() {
            match ch {
                '"' => self.parse_string(),
                '[' => self.parse_array(),
                '{' => self.parse_object(),
                '0'..='9' | '-' => self.parse_number(),
                't' => {
                    if self.input[self.pos..].starts_with(&['t', 'r', 'u', 'e']) {
                        self.pos += 4;
                        Ok(JsonValue::Bool(true))
                    } else {
                        Err("Expected 'true'".to_string())
                    }
                }
                'f' => {
                    if self.input[self.pos..].starts_with(&['f', 'a', 'l', 's', 'e']) {
                        self.pos += 5;
                        Ok(JsonValue::Bool(false))
                    } else {
                        Err("Expected 'false'".to_string())
                    }
                }
                'n' => {
                    if self.input[self.pos..].starts_with(&['n', 'u', 'l', 'l']) {
                        self.pos += 4;
                        Ok(JsonValue::Null)
                    } else {
                        Err("Expected 'null'".to_string())
                    }
                }
                _ => Err(format!("Unexpected character: {}", ch)),
            }
        } else {
            Err("Unexpected end of input".to_string())
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            Err("Extra characters after JSON value".to_string())
        } else {
            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello world""#);
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello world".to_string())));
    }

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42.5");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(42.5)));
    }

    #[test]
    fn test_parse_array() {
        let mut parser = JsonParser::new("[1, 2, 3]");
        let expected = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);
        assert_eq!(parser.parse(), Ok(expected));
    }
}