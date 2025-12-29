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

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.input[self.position].is_whitespace() {
            self.position += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn consume(&mut self, expected: char) -> Result<(), ParseError> {
        self.skip_whitespace();
        match self.peek() {
            Some(ch) if ch == expected => {
                self.position += 1;
                Ok(())
            }
            Some(ch) => Err(ParseError::UnexpectedCharacter(ch, self.position)),
            None => Err(ParseError::UnexpectedEndOfInput),
        }
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        self.consume('"')?;
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    self.position += 1;
                    return Ok(result);
                }
                '\\' => {
                    self.position += 1;
                    let escaped = self.peek().ok_or(ParseError::UnexpectedEndOfInput)?;
                    match escaped {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        '/' => result.push('/'),
                        'b' => result.push('\x08'),
                        'f' => result.push('\x0c'),
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        _ => return Err(ParseError::InvalidEscapeSequence),
                    }
                    self.position += 1;
                }
                _ => {
                    result.push(ch);
                    self.position += 1;
                }
            }
        }
        Err(ParseError::UnexpectedEndOfInput)
    }

    fn parse_number(&mut self) -> Result<f64, ParseError> {
        let start = self.position;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '+' || ch == 'e' || ch == 'E' {
                self.position += 1;
            } else {
                break;
            }
        }
        let num_str: String = self.input[start..self.position].iter().collect();
        num_str.parse().map_err(|_| ParseError::InvalidNumber)
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        match self.peek() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => Ok(JsonValue::String(self.parse_string()?)),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(ch) if ch.is_ascii_digit() || ch == '-' => Ok(JsonValue::Number(self.parse_number()?)),
            _ => Err(ParseError::UnexpectedCharacter(
                self.peek().unwrap_or('\0'),
                self.position,
            )),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        let expected = ['n', 'u', 'l', 'l'];
        for &ch in &expected {
            self.consume(ch)?;
        }
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, ParseError> {
        match self.peek() {
            Some('t') => {
                let expected = ['t', 'r', 'u', 'e'];
                for &ch in &expected {
                    self.consume(ch)?;
                }
                Ok(JsonValue::Bool(true))
            }
            Some('f') => {
                let expected = ['f', 'a', 'l', 's', 'e'];
                for &ch in &expected {
                    self.consume(ch)?;
                }
                Ok(JsonValue::Bool(false))
            }
            _ => Err(ParseError::UnexpectedCharacter(
                self.peek().unwrap_or('\0'),
                self.position,
            )),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.consume('[')?;
        let mut array = Vec::new();
        self.skip_whitespace();
        if let Some(']') = self.peek() {
            self.position += 1;
            return Ok(JsonValue::Array(array));
        }
        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.position += 1;
                    continue;
                }
                Some(']') => {
                    self.position += 1;
                    break;
                }
                Some(ch) => return Err(ParseError::UnexpectedCharacter(ch, self.position)),
                None => return Err(ParseError::UnexpectedEndOfInput),
            }
        }
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        self.consume('{')?;
        let mut map = HashMap::new();
        self.skip_whitespace();
        if let Some('}') = self.peek() {
            self.position += 1;
            return Ok(JsonValue::Object(map));
        }
        loop {
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.consume(':')?;
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.position += 1;
                    continue;
                }
                Some('}') => {
                    self.position += 1;
                    break;
                }
                Some(ch) => return Err(ParseError::UnexpectedCharacter(ch, self.position)),
                None => return Err(ParseError::UnexpectedEndOfInput),
            }
        }
        Ok(JsonValue::Object(map))
    }

    pub fn parse(&mut self) -> Result<JsonValue, ParseError> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
            Err(ParseError::UnexpectedCharacter(
                self.input[self.position],
                self.position,
            ))
        } else {
            Ok(result)
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