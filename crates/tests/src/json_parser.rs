use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedCharacter(char, usize),
    UnexpectedEndOfInput,
    InvalidNumber,
    InvalidEscapeSequence,
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

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn consume(&mut self) -> Option<char> {
        let ch = self.peek();
        if ch.is_some() {
            self.position += 1;
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.consume();
            } else {
                break;
            }
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        self.parse_value()
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        match self.peek() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string().map(JsonValue::String),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(ch) if ch.is_digit(10) || ch == '-' => self.parse_number(),
            _ => Err(ParseError::UnexpectedCharacter(
                self.peek().unwrap_or('\0'),
                self.position,
            )),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        let expected = "null";
        for (i, expected_ch) in expected.chars().enumerate() {
            match self.consume() {
                Some(ch) if ch == expected_ch => continue,
                Some(ch) => return Err(ParseError::UnexpectedCharacter(ch, self.position - 1)),
                None => return Err(ParseError::UnexpectedEndOfInput),
            }
        }
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, ParseError> {
        let next = self.peek();
        match next {
            Some('t') => {
                let expected = "true";
                for expected_ch in expected.chars() {
                    if self.consume() != Some(expected_ch) {
                        return Err(ParseError::UnexpectedCharacter(
                            self.input[self.position - 1],
                            self.position - 1,
                        ));
                    }
                }
                Ok(JsonValue::Bool(true))
            }
            Some('f') => {
                let expected = "false";
                for expected_ch in expected.chars() {
                    if self.consume() != Some(expected_ch) {
                        return Err(ParseError::UnexpectedCharacter(
                            self.input[self.position - 1],
                            self.position - 1,
                        ));
                    }
                }
                Ok(JsonValue::Bool(false))
            }
            _ => Err(ParseError::UnexpectedCharacter(
                next.unwrap_or('\0'),
                self.position,
            )),
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.position;
        let mut has_dot = false;

        if self.peek() == Some('-') {
            self.consume();
        }

        while let Some(ch) = self.peek() {
            if ch.is_digit(10) {
                self.consume();
            } else if ch == '.' && !has_dot {
                has_dot = true;
                self.consume();
            } else {
                break;
            }
        }

        let number_str: String = self.input[start..self.position].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(ParseError::InvalidNumber),
        }
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        if self.consume() != Some('"') {
            return Err(ParseError::UnexpectedCharacter(
                self.input[self.position - 1],
                self.position - 1,
            ));
        }

        let mut result = String::new();
        while let Some(ch) = self.consume() {
            match ch {
                '"' => return Ok(result),
                '\\' => {
                    let escaped = self.parse_escape_sequence()?;
                    result.push(escaped);
                }
                _ => result.push(ch),
            }
        }

        Err(ParseError::UnexpectedEndOfInput)
    }

    fn parse_escape_sequence(&mut self) -> Result<char, ParseError> {
        match self.consume() {
            Some('"') => Ok('"'),
            Some('\\') => Ok('\\'),
            Some('/') => Ok('/'),
            Some('b') => Ok('\x08'),
            Some('f') => Ok('\x0c'),
            Some('n') => Ok('\n'),
            Some('r') => Ok('\r'),
            Some('t') => Ok('\t'),
            Some(_) => Err(ParseError::InvalidEscapeSequence),
            None => Err(ParseError::UnexpectedEndOfInput),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        if self.consume() != Some('[') {
            return Err(ParseError::UnexpectedCharacter(
                self.input[self.position - 1],
                self.position - 1,
            ));
        }

        let mut array = Vec::new();
        self.skip_whitespace();

        if self.peek() == Some(']') {
            self.consume();
            return Ok(JsonValue::Array(array));
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();

            match self.peek() {
                Some(',') => {
                    self.consume();
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.consume();
                    break;
                }
                Some(ch) => return Err(ParseError::UnexpectedCharacter(ch, self.position)),
                None => return Err(ParseError::UnexpectedEndOfInput),
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        if self.consume() != Some('{') {
            return Err(ParseError::UnexpectedCharacter(
                self.input[self.position - 1],
                self.position - 1,
            ));
        }

        let mut object = HashMap::new();
        self.skip_whitespace();

        if self.peek() == Some('}') {
            self.consume();
            return Ok(JsonValue::Object(object));
        }

        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();

            if self.consume() != Some(':') {
                return Err(ParseError::UnexpectedCharacter(
                    self.input[self.position - 1],
                    self.position - 1,
                ));
            }

            self.skip_whitespace();
            let value = self.parse_value()?;
            object.insert(key, value);
            self.skip_whitespace();

            match self.peek() {
                Some(',') => {
                    self.consume();
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.consume();
                    break;
                }
                Some(ch) => return Err(ParseError::UnexpectedCharacter(ch, self.position)),
                None => return Err(ParseError::UnexpectedEndOfInput),
            }
        }

        Ok(JsonValue::Object(object))
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

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn consume(&mut self) -> Option<char> {
        let ch = self.peek();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.consume();
            } else {
                break;
            }
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.peek().is_some() {
            return Err("Unexpected trailing characters".to_string());
        }
        Ok(result)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.peek() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(ch) if ch.is_digit(10) || ch == '-' => self.parse_number(),
            _ => Err("Invalid JSON value".to_string()),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        let expected = "null";
        for c in expected.chars() {
            if self.consume() != Some(c) {
                return Err(format!("Expected '{}'", expected));
            }
        }
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.starts_with("true") {
            self.pos += 4;
            Ok(JsonValue::Bool(true))
        } else if self.starts_with("false") {
            self.pos += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err("Invalid boolean value".to_string())
        }
    }

    fn starts_with(&self, s: &str) -> bool {
        s.chars()
            .enumerate()
            .all(|(i, c)| self.input.get(self.pos + i) == Some(&c))
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.consume(); // consume opening quote
        let mut result = String::new();
        while let Some(ch) = self.consume() {
            if ch == '"' {
                return Ok(JsonValue::String(result));
            }
            result.push(ch);
        }
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        let mut has_dot = false;
        while let Some(ch) = self.peek() {
            if ch.is_digit(10) {
                self.consume();
            } else if ch == '.' && !has_dot {
                has_dot = true;
                self.consume();
            } else {
                break;
            }
        }
        let num_str: String = self.input[start..self.pos].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number".to_string()),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume(); // consume '['
        self.skip_whitespace();
        let mut array = Vec::new();
        if self.peek() == Some(']') {
            self.consume();
            return Ok(JsonValue::Array(array));
        }
        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.consume();
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.consume();
                    break;
                }
                _ => return Err("Expected ',' or ']' in array".to_string()),
            }
        }
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume(); // consume '{'
        self.skip_whitespace();
        let mut map = HashMap::new();
        if self.peek() == Some('}') {
            self.consume();
            return Ok(JsonValue::Object(map));
        }
        loop {
            self.skip_whitespace();
            if self.peek() != Some('"') {
                return Err("Expected string key in object".to_string());
            }
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => unreachable!(),
            };
            self.skip_whitespace();
            if self.consume() != Some(':') {
                return Err("Expected ':' after object key".to_string());
            }
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.consume();
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.consume();
                    break;
                }
                _ => return Err("Expected ',' or '}' in object".to_string()),
            }
        }
        Ok(JsonValue::Object(map))
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
        let mut parser = JsonParser::new("3.14");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(3.14)));
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello""#);
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello".to_string())));
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
    input: String,
    pos: usize,
}

impl JsonParser {
    pub fn new(input: String) -> Self {
        JsonParser { input, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err("Unexpected trailing characters".to_string());
        }
        Ok(result)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.peek_char() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            _ => Err("Invalid JSON value".to_string()),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.consume_str("null") {
            Ok(JsonValue::Null)
        } else {
            Err("Expected 'null'".to_string())
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.consume_str("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume_str("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err("Expected 'true' or 'false'".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.consume_char('"');
        let mut result = String::new();
        while let Some(c) = self.next_char() {
            if c == '"' {
                break;
            }
            result.push(c);
        }
        Ok(JsonValue::String(result))
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        let mut has_dot = false;
        while let Some(c) = self.peek_char() {
            if c.is_digit(10) || c == '-' || c == '+' || c == '.' || c == 'e' || c == 'E' {
                if c == '.' {
                    has_dot = true;
                }
                self.pos += 1;
            } else {
                break;
            }
        }
        let num_str = &self.input[start..self.pos];
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number".to_string()),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume_char('[');
        self.skip_whitespace();
        let mut array = Vec::new();
        if self.peek_char() == Some(']') {
            self.consume_char(']');
            return Ok(JsonValue::Array(array));
        }
        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                self.consume_char(']');
                break;
            }
            if self.peek_char() != Some(',') {
                return Err("Expected ',' or ']'".to_string());
            }
            self.consume_char(',');
            self.skip_whitespace();
        }
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume_char('{');
        self.skip_whitespace();
        let mut map = HashMap::new();
        if self.peek_char() == Some('}') {
            self.consume_char('}');
            return Ok(JsonValue::Object(map));
        }
        loop {
            self.skip_whitespace();
            if self.peek_char() != Some('"') {
                return Err("Expected string key".to_string());
            }
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err("Key must be a string".to_string()),
            };
            self.skip_whitespace();
            if self.peek_char() != Some(':') {
                return Err("Expected ':'".to_string());
            }
            self.consume_char(':');
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            if self.peek_char() == Some('}') {
                self.consume_char('}');
                break;
            }
            if self.peek_char() != Some(',') {
                return Err("Expected ',' or '}'".to_string());
            }
            self.consume_char(',');
            self.skip_whitespace();
        }
        Ok(JsonValue::Object(map))
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.peek_char();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn consume_char(&mut self, expected: char) {
        if let Some(c) = self.next_char() {
            if c != expected {
                panic!("Expected '{}', found '{}'", expected, c);
            }
        } else {
            panic!("Unexpected end of input");
        }
    }

    fn consume_str(&mut self, expected: &str) -> bool {
        if self.input[self.pos..].starts_with(expected) {
            self.pos += expected.len();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        let mut parser = JsonParser::new("null".to_string());
        assert_eq!(parser.parse(), Ok(JsonValue::Null));
    }

    #[test]
    fn test_parse_bool() {
        let mut parser = JsonParser::new("true".to_string());
        assert_eq!(parser.parse(), Ok(JsonValue::Bool(true)));
        let mut parser = JsonParser::new("false".to_string());
        assert_eq!(parser.parse(), Ok(JsonValue::Bool(false)));
    }

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42".to_string());
        assert_eq!(parser.parse(), Ok(JsonValue::Number(42.0)));
        let mut parser = JsonParser::new("-3.14".to_string());
        assert_eq!(parser.parse(), Ok(JsonValue::Number(-3.14)));
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new("\"hello\"".to_string());
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello".to_string())));
    }

    #[test]
    fn test_parse_array() {
        let mut parser = JsonParser::new("[1, 2, 3]".to_string());
        let expected = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);
        assert_eq!(parser.parse(), Ok(expected));
    }

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new("{\"key\": \"value\"}".to_string());
        let mut map = HashMap::new();
        map.insert("key".to_string(), JsonValue::String("value".to_string()));
        let expected = JsonValue::Object(map);
        assert_eq!(parser.parse(), Ok(expected));
    }
}