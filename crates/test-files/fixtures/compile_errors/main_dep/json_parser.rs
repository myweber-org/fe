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

#[derive(Debug)]
pub enum ParseError {
    UnexpectedCharacter(char, usize),
    UnexpectedEndOfInput,
    InvalidNumber,
    InvalidEscapeSequence,
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
            Some('"') => self.parse_string(),
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
                        return Err(ParseError::UnexpectedEndOfInput);
                    }
                }
                Ok(JsonValue::Bool(true))
            }
            Some('f') => {
                let expected = "false";
                for expected_ch in expected.chars() {
                    if self.consume() != Some(expected_ch) {
                        return Err(ParseError::UnexpectedEndOfInput);
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

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        self.consume(); // Consume opening quote
        let mut result = String::new();

        while let Some(ch) = self.consume() {
            match ch {
                '"' => return Ok(JsonValue::String(result)),
                '\\' => {
                    let escaped = self.consume().ok_or(ParseError::UnexpectedEndOfInput)?;
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
                }
                _ => result.push(ch),
            }
        }

        Err(ParseError::UnexpectedEndOfInput)
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.position;
        let mut has_decimal = false;
        let mut has_exponent = false;

        if self.peek() == Some('-') {
            self.consume();
        }

        while let Some(ch) = self.peek() {
            if ch.is_digit(10) {
                self.consume();
            } else if ch == '.' && !has_decimal && !has_exponent {
                has_decimal = true;
                self.consume();
            } else if (ch == 'e' || ch == 'E') && !has_exponent {
                has_exponent = true;
                self.consume();
                if self.peek() == Some('-') || self.peek() == Some('+') {
                    self.consume();
                }
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

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.consume(); // Consume '['
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
            match self.consume() {
                Some(',') => {
                    self.skip_whitespace();
                    continue;
                }
                Some(']') => break,
                Some(ch) => return Err(ParseError::UnexpectedCharacter(ch, self.position - 1)),
                None => return Err(ParseError::UnexpectedEndOfInput),
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        self.consume(); // Consume '{'
        let mut object = HashMap::new();

        self.skip_whitespace();
        if self.peek() == Some('}') {
            self.consume();
            return Ok(JsonValue::Object(object));
        }

        loop {
            self.skip_whitespace();
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => unreachable!(),
            };

            self.skip_whitespace();
            match self.consume() {
                Some(':') => (),
                Some(ch) => return Err(ParseError::UnexpectedCharacter(ch, self.position - 1)),
                None => return Err(ParseError::UnexpectedEndOfInput),
            }

            self.skip_whitespace();
            let value = self.parse_value()?;
            object.insert(key, value);

            self.skip_whitespace();
            match self.consume() {
                Some(',') => {
                    self.skip_whitespace();
                    continue;
                }
                Some('}') => break,
                Some(ch) => return Err(ParseError::UnexpectedCharacter(ch, self.position - 1)),
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

        let mut parser = JsonParser::new("1.23e-4");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(1.23e-4)));
    }

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello world""#);
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::String("hello world".to_string()))
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

        let mut parser = JsonParser::new("[]");
        assert_eq!(parser.parse(), Ok(JsonValue::Array(vec![])));
    }

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new(r#"{"key": "value"}"#);
        let mut expected = HashMap::new();
        expected.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(expected)));

        let mut parser = JsonParser::new("{}");
        assert_eq!(parser.parse(), Ok(JsonValue::Object(HashMap::new())));
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

    fn consume(&mut self, expected: char) -> Result<(), String> {
        match self.peek() {
            Some(ch) if ch == expected => {
                self.pos += 1;
                Ok(())
            }
            Some(ch) => Err(format!("Expected '{}', found '{}'", expected, ch)),
            None => Err(format!("Expected '{}', found EOF", expected)),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        let result = match self.peek() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(ch) if ch.is_digit(10) || ch == '-' => self.parse_number(),
            _ => Err("Invalid JSON token".to_string()),
        }?;
        self.skip_whitespace();
        Ok(result)
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        let expected = "null";
        for (i, ch) in expected.chars().enumerate() {
            if self.peek() != Some(ch) {
                return Err(format!("Expected null at position {}", self.pos + i));
            }
            self.pos += 1;
        }
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.input[self.pos..].starts_with(&['t', 'r', 'u', 'e']) {
            self.pos += 4;
            Ok(JsonValue::Bool(true))
        } else if self.input[self.pos..].starts_with(&['f', 'a', 'l', 's', 'e']) {
            self.pos += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err("Invalid boolean value".to_string())
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        let mut has_dot = false;

        if self.peek() == Some('-') {
            self.pos += 1;
        }

        while let Some(ch) = self.peek() {
            if ch.is_digit(10) {
                self.pos += 1;
            } else if ch == '.' && !has_dot {
                has_dot = true;
                self.pos += 1;
            } else {
                break;
            }
        }

        let num_str: String = self.input[start..self.pos].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number format".to_string()),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.consume('"')?;
        let mut result = String::new();

        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.pos += 1;
                break;
            } else if ch == '\\' {
                self.pos += 1;
                match self.peek() {
                    Some('"') => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some('/') => result.push('/'),
                    Some('b') => result.push('\x08'),
                    Some('f') => result.push('\x0c'),
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some(_) => return Err("Invalid escape sequence".to_string()),
                    None => return Err("Unterminated string".to_string()),
                }
                self.pos += 1;
            } else {
                result.push(ch);
                self.pos += 1;
            }
        }

        Ok(JsonValue::String(result))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume('[')?;
        self.skip_whitespace();
        let mut array = Vec::new();

        if self.peek() == Some(']') {
            self.pos += 1;
            return Ok(JsonValue::Array(array));
        }

        loop {
            let value = self.parse()?;
            array.push(value);
            self.skip_whitespace();

            match self.peek() {
                Some(',') => {
                    self.pos += 1;
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.pos += 1;
                    break;
                }
                _ => return Err("Expected ',' or ']' in array".to_string()),
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume('{')?;
        self.skip_whitespace();
        let mut map = HashMap::new();

        if self.peek() == Some('}') {
            self.pos += 1;
            return Ok(JsonValue::Object(map));
        }

        loop {
            let key = match self.parse()? {
                JsonValue::String(s) => s,
                _ => return Err("Object keys must be strings".to_string()),
            };

            self.skip_whitespace();
            self.consume(':')?;
            self.skip_whitespace();

            let value = self.parse()?;
            map.insert(key, value);

            self.skip_whitespace();

            match self.peek() {
                Some(',') => {
                    self.pos += 1;
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.pos += 1;
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
        let mut map = HashMap::new();
        map.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(map)));
    }
}