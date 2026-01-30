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
    MissingClosingQuote,
    MissingClosingBracket,
    MissingClosingBrace,
    TrailingCharacters,
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
        let value = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
            return Err(ParseError::TrailingCharacters);
        }
        Ok(value)
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
            self.advance_by(4);
            Ok(JsonValue::Bool(true))
        } else if self.starts_with("false") {
            self.advance_by(5);
            Ok(JsonValue::Bool(false))
        } else {
            Err(ParseError::UnexpectedCharacter(self.input[self.position], self.position))
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.position;
        if self.peek_char() == Some('-') {
            self.advance();
        }

        while let Some(c) = self.peek_char() {
            if !c.is_digit(10) {
                break;
            }
            self.advance();
        }

        if self.peek_char() == Some('.') {
            self.advance();
            while let Some(c) = self.peek_char() {
                if !c.is_digit(10) {
                    break;
                }
                self.advance();
            }
        }

        let number_str: String = self.input[start..self.position].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(ParseError::InvalidNumber),
        }
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
                _ => {
                    result.push(c);
                    self.advance();
                }
            }
        }

        Err(ParseError::MissingClosingQuote)
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
                Ok('\x0c')
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
            Some(c) => Err(ParseError::InvalidEscapeSequence),
            None => Err(ParseError::UnexpectedEndOfInput),
        }
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
                Some(']') => {
                    self.advance();
                    break;
                }
                Some(',') => {
                    self.advance();
                    self.skip_whitespace();
                }
                Some(c) => return Err(ParseError::UnexpectedCharacter(c, self.position)),
                None => return Err(ParseError::MissingClosingBracket),
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
            self.skip_whitespace();
            let key = match self.parse_value()? {
                JsonValue::String(s) => s,
                _ => return Err(ParseError::UnexpectedCharacter(self.input[self.position], self.position)),
            };

            self.skip_whitespace();
            self.expect(":")?;

            let value = self.parse_value()?;
            object.insert(key, value);

            self.skip_whitespace();
            match self.peek_char() {
                Some('}') => {
                    self.advance();
                    break;
                }
                Some(',') => {
                    self.advance();
                    self.skip_whitespace();
                }
                Some(c) => return Err(ParseError::UnexpectedCharacter(c, self.position)),
                None => return Err(ParseError::MissingClosingBrace),
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

    fn starts_with(&self, prefix: &str) -> bool {
        self.input[self.position..].iter().collect::<String>().starts_with(prefix)
    }

    fn advance_by(&mut self, n: usize) {
        self.position += n;
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn peek_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
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
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello world""#);
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello world".to_string())));
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

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new(r#"{"key": "value"}"#);
        let mut expected_map = HashMap::new();
        expected_map.insert("key".to_string(), JsonValue::String("value".to_string()));
        let expected = JsonValue::Object(expected_map);
        assert_eq!(parser.parse(), Ok(expected));
    }
}