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
}