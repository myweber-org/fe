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
        write!(f, "Parse error at position {}: {}", self.position, self.message)
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
            _ => Err(self.error("Unexpected character")),
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
        if self.peek_char() == Some('-') {
            self.advance(1);
        }
        
        while let Some(c) = self.peek_char() {
            if !c.is_digit(10) && c != '.' && c != 'e' && c != 'E' && c != '+' && c != '-' {
                break;
            }
            self.advance(1);
        }
        
        let num_str: String = self.input[start..self.position].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(self.error("Invalid number format")),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("\"")?;
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
                        _ => return Err(self.error("Invalid escape sequence")),
                    }
                }
                _ => result.push(c),
            }
        }
        
        Err(self.error("Unterminated string"))
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
            self.skip_whitespace();
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();
            
            match self.peek_char() {
                Some(',') => {
                    self.advance(1);
                    continue;
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
            self.skip_whitespace();
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
                    continue;
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
        self.position = (self.position + n).min(self.input.len());
    }

    fn starts_with(&self, s: &str) -> bool {
        let end = self.position + s.len();
        if end > self.input.len() {
            return false;
        }
        self.input[self.position..end].iter().collect::<String>() == s
    }

    fn expect(&mut self, s: &str) -> Result<(), ParseError> {
        if self.starts_with(s) {
            self.advance(s.len());
            Ok(())
        } else {
            Err(self.error(&format!("Expected '{}'", s)))
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
        let mut parser = JsonParser::new("\"hello\"");
        assert_eq!(parser.parse().unwrap(), JsonValue::String("hello".to_string()));
        
        let mut parser = JsonParser::new("\"hello\\nworld\"");
        assert_eq!(parser.parse().unwrap(), JsonValue::String("hello\nworld".to_string()));
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
        let mut parser = JsonParser::new("{\"key\": \"value\"}");
        let mut expected_map = HashMap::new();
        expected_map.insert("key".to_string(), JsonValue::String("value".to_string()));
        let expected = JsonValue::Object(expected_map);
        assert_eq!(parser.parse().unwrap(), expected);
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
    EmptyKey,
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
        let value = self.parse_value()?;
        self.skip_whitespace();
        
        if self.position < self.input.len() {
            return Err(ParseError::UnexpectedCharacter(
                self.input[self.position],
                self.position,
            ));
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
            Some(c) => Err(ParseError::UnexpectedCharacter(c, self.position)),
            None => Err(ParseError::UnexpectedEndOfInput),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("null")?;
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, ParseError> {
        if self.starts_with("true") {
            self.position += 4;
            Ok(JsonValue::Bool(true))
        } else if self.starts_with("false") {
            self.position += 5;
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
        let mut has_decimal = false;
        let mut has_exponent = false;

        if self.peek_char() == Some('-') {
            self.advance();
        }

        while let Some(c) = self.peek_char() {
            match c {
                '0'..='9' => self.advance(),
                '.' if !has_decimal && !has_exponent => {
                    has_decimal = true;
                    self.advance();
                }
                'e' | 'E' if !has_exponent => {
                    has_exponent = true;
                    self.advance();
                    if self.peek_char() == Some('-') || self.peek_char() == Some('+') {
                        self.advance();
                    }
                }
                _ => break,
            }
        }

        let number_str: String = self.input[start..self.position].iter().collect();
        number_str
            .parse::<f64>()
            .map(JsonValue::Number)
            .map_err(|_| ParseError::InvalidNumber)
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("\"")?;
        let mut result = String::new();

        while let Some(c) = self.peek_char() {
            match c {
                '"' => {
                    self.advance();
                    return Ok(JsonValue::String(result));
                }
                '\\' => {
                    self.advance();
                    let escaped = self.parse_escape_sequence()?;
                    result.push(escaped);
                }
                c if c.is_control() => {
                    return Err(ParseError::UnexpectedCharacter(c, self.position));
                }
                _ => {
                    result.push(c);
                    self.advance();
                }
            }
        }

        Err(ParseError::UnexpectedEndOfInput)
    }

    fn parse_escape_sequence(&mut self) -> Result<char, ParseError> {
        match self.peek_char() {
            Some('"') => {
                self.advance();
                Ok('"')
            }
            Some('\\') => {
                self.advance();
                Ok('\\')
            }
            Some('/') => {
                self.advance();
                Ok('/')
            }
            Some('b') => {
                self.advance();
                Ok('\x08')
            }
            Some('f') => {
                self.advance();
                Ok('\x0C')
            }
            Some('n') => {
                self.advance();
                Ok('\n')
            }
            Some('r') => {
                self.advance();
                Ok('\r')
            }
            Some('t') => {
                self.advance();
                Ok('\t')
            }
            Some('u') => {
                self.advance();
                self.parse_unicode_escape()
            }
            Some(c) => Err(ParseError::InvalidEscapeSequence),
            None => Err(ParseError::UnexpectedEndOfInput),
        }
    }

    fn parse_unicode_escape(&mut self) -> Result<char, ParseError> {
        let mut code_point = 0;
        for _ in 0..4 {
            match self.peek_char() {
                Some(c) if c.is_digit(16) => {
                    code_point = code_point * 16 + c.to_digit(16).unwrap();
                    self.advance();
                }
                _ => return Err(ParseError::InvalidEscapeSequence),
            }
        }
        char::from_u32(code_point).ok_or(ParseError::InvalidEscapeSequence)
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("[")?;
        self.skip_whitespace();

        let mut array = Vec::new();

        if self.peek_char() == Some(']') {
            self.advance();
            return Ok(JsonValue::Array(array));
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();

            match self.peek_char() {
                Some(',') => {
                    self.advance();
                    self.skip_whitespace();
                    if self.peek_char() == Some(']') {
                        return Err(ParseError::TrailingComma);
                    }
                }
                Some(']') => {
                    self.advance();
                    break;
                }
                Some(c) => return Err(ParseError::UnexpectedCharacter(c, self.position)),
                None => return Err(ParseError::UnexpectedEndOfInput),
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        self.expect("{")?;
        self.skip_whitespace();

        let mut object = HashMap::new();

        if self.peek_char() == Some('}') {
            self.advance();
            return Ok(JsonValue::Object(object));
        }

        loop {
            let key = match self.parse_value()? {
                JsonValue::String(s) => {
                    if s.is_empty() {
                        return Err(ParseError::EmptyKey);
                    }
                    s
                }
                _ => return Err(ParseError::UnexpectedCharacter('"', self.position)),
            };

            self.skip_whitespace();
            self.expect(":")?;
            self.skip_whitespace();

            let value = self.parse_value()?;
            object.insert(key, value);
            self.skip_whitespace();

            match self.peek_char() {
                Some(',') => {
                    self.advance();
                    self.skip_whitespace();
                    if self.peek_char() == Some('}') {
                        return Err(ParseError::TrailingComma);
                    }
                }
                Some('}') => {
                    self.advance();
                    break;
                }
                Some(c) => return Err(ParseError::UnexpectedCharacter(c, self.position)),
                None => return Err(ParseError::UnexpectedEndOfInput),
            }
        }

        Ok(JsonValue::Object(object))
    }

    fn expect(&mut self, expected: &str) -> Result<(), ParseError> {
        for ch in expected.chars() {
            match self.peek_char() {
                Some(c) if c == ch => self.advance(),
                Some(c) => return Err(ParseError::UnexpectedCharacter(c, self.position)),
                None => return Err(ParseError::UnexpectedEndOfInput),
            }
        }
        Ok(())
    }

    fn starts_with(&self, s: &str) -> bool {
        s.chars()
            .enumerate()
            .all(|(i, c)| self.input.get(self.position + i) == Some(&c))
    }

    fn peek_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
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

        let mut parser = JsonParser::new("1.23e-4");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(1.23e-4)));
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello""#);
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::String("hello".to_string()))
        );

        let mut parser = JsonParser::new(r#""escape\"test""#);
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::String("escape\"test".to_string()))
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
        let mut expected = HashMap::new();
        expected.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(expected)));
    }

    #[test]
    fn test_error_handling() {
        let mut parser = JsonParser::new("invalid");
        assert!(parser.parse().is_err());

        let mut parser = JsonParser::new("[1, 2, ]");
        assert!(matches!(
            parser.parse(),
            Err(ParseError::TrailingComma)
        ));

        let mut parser = JsonParser::new(r#"{"": "value"}"#);
        assert!(matches!(
            parser.parse(),
            Err(ParseError::EmptyKey)
        ));
    }
}