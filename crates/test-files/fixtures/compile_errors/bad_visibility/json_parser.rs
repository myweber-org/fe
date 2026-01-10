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
    UnexpectedEndOfInput,
    InvalidToken,
    ExpectedValue,
    ExpectedColon,
    ExpectedComma,
    TrailingCharacters,
}

pub struct JsonParser {
    input: String,
    pos: usize,
}

impl JsonParser {
    pub fn new(input: String) -> Self {
        JsonParser { input, pos: 0 }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if c.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        
        if self.pos >= self.input.len() {
            return Err(ParseError::UnexpectedEndOfInput);
        }

        let c = self.input.chars().nth(self.pos).unwrap();
        match c {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_bool(),
            '"' => self.parse_string(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            '-' | '0'..='9' => self.parse_number(),
            _ => Err(ParseError::InvalidToken),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        if self.input[self.pos..].starts_with("null") {
            self.pos += 4;
            Ok(JsonValue::Null)
        } else {
            Err(ParseError::InvalidToken)
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, ParseError> {
        if self.input[self.pos..].starts_with("true") {
            self.pos += 4;
            Ok(JsonValue::Bool(true))
        } else if self.input[self.pos..].starts_with("false") {
            self.pos += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err(ParseError::InvalidToken)
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        self.pos += 1; // Skip opening quote
        let start = self.pos;
        
        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if c == '"' {
                let s = self.input[start..self.pos].to_string();
                self.pos += 1; // Skip closing quote
                return Ok(JsonValue::String(s));
            }
            self.pos += 1;
        }
        
        Err(ParseError::UnexpectedEndOfInput)
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.pos;
        
        if self.input.chars().nth(self.pos) == Some('-') {
            self.pos += 1;
        }
        
        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if !c.is_ascii_digit() && c != '.' && c != 'e' && c != 'E' && c != '+' && c != '-' {
                break;
            }
            self.pos += 1;
        }
        
        let num_str = &self.input[start..self.pos];
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(ParseError::InvalidToken),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        self.pos += 1; // Skip '['
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
        if self.pos < self.input.len() && self.input.chars().nth(self.pos) == Some(']') {
            self.pos += 1;
            return Ok(JsonValue::Array(array));
        }
        
        loop {
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            if self.pos >= self.input.len() {
                return Err(ParseError::UnexpectedEndOfInput);
            }
            
            let c = self.input.chars().nth(self.pos).unwrap();
            if c == ']' {
                self.pos += 1;
                break;
            } else if c == ',' {
                self.pos += 1;
                self.skip_whitespace();
                continue;
            } else {
                return Err(ParseError::ExpectedComma);
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        self.pos += 1; // Skip '{'
        self.skip_whitespace();
        
        let mut object = HashMap::new();
        
        if self.pos < self.input.len() && self.input.chars().nth(self.pos) == Some('}') {
            self.pos += 1;
            return Ok(JsonValue::Object(object));
        }
        
        loop {
            self.skip_whitespace();
            let key = match self.parse_value()? {
                JsonValue::String(s) => s,
                _ => return Err(ParseError::InvalidToken),
            };
            
            self.skip_whitespace();
            if self.pos >= self.input.len() {
                return Err(ParseError::UnexpectedEndOfInput);
            }
            
            if self.input.chars().nth(self.pos) != Some(':') {
                return Err(ParseError::ExpectedColon);
            }
            self.pos += 1;
            
            let value = self.parse_value()?;
            object.insert(key, value);
            
            self.skip_whitespace();
            if self.pos >= self.input.len() {
                return Err(ParseError::UnexpectedEndOfInput);
            }
            
            let c = self.input.chars().nth(self.pos).unwrap();
            if c == '}' {
                self.pos += 1;
                break;
            } else if c == ',' {
                self.pos += 1;
                self.skip_whitespace();
                continue;
            } else {
                return Err(ParseError::ExpectedComma);
            }
        }
        
        Ok(JsonValue::Object(object))
    }

    pub fn parse(&mut self) -> Result<JsonValue, ParseError> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        
        if self.pos < self.input.len() {
            return Err(ParseError::TrailingCharacters);
        }
        
        Ok(result)
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
    fn test_parse_string() {
        let mut parser = JsonParser::new("\"hello world\"".to_string());
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello world".to_string())));
    }

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42".to_string());
        assert_eq!(parser.parse(), Ok(JsonValue::Number(42.0)));
        
        let mut parser = JsonParser::new("-3.14".to_string());
        assert_eq!(parser.parse(), Ok(JsonValue::Number(-3.14)));
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
        let mut parser = JsonParser::new(r#"{"key": "value", "num": 42}"#.to_string());
        let mut expected_map = HashMap::new();
        expected_map.insert("key".to_string(), JsonValue::String("value".to_string()));
        expected_map.insert("num".to_string(), JsonValue::Number(42.0));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(expected_map)));
    }
}