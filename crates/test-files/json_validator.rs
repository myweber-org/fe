use std::collections::HashMap;
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
        write!(f, "Error at position {}: {}", self.position, self.message)
    }
}

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
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err(ParseError {
                message: "Unexpected trailing characters".to_string(),
                position: self.pos,
            });
        }
        Ok(result)
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        match self.peek_char() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            _ => Err(ParseError {
                message: "Unexpected character".to_string(),
                position: self.pos,
            }),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        if self.consume_str("null") {
            Ok(JsonValue::Null)
        } else {
            Err(ParseError {
                message: "Expected 'null'".to_string(),
                position: self.pos,
            })
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, ParseError> {
        if self.consume_str("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume_str("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err(ParseError {
                message: "Expected boolean value".to_string(),
                position: self.pos,
            })
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, ParseError> {
        let start = self.pos;
        if self.consume_char('-') {
            // Optional minus sign
        }
        
        while let Some(c) = self.peek_char() {
            if c.is_digit(10) {
                self.consume_char();
            } else {
                break;
            }
        }
        
        if self.peek_char() == Some('.') {
            self.consume_char();
            while let Some(c) = self.peek_char() {
                if c.is_digit(10) {
                    self.consume_char();
                } else {
                    break;
                }
            }
        }
        
        let number_str: String = self.input[start..self.pos].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(ParseError {
                message: "Invalid number format".to_string(),
                position: start,
            }),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, ParseError> {
        if !self.consume_char('"') {
            return Err(ParseError {
                message: "Expected opening quote".to_string(),
                position: self.pos,
            });
        }
        
        let mut result = String::new();
        while let Some(c) = self.peek_char() {
            if c == '"' {
                self.consume_char();
                return Ok(JsonValue::String(result));
            } else if c == '\\' {
                self.consume_char();
                match self.peek_char() {
                    Some('"') => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some('/') => result.push('/'),
                    Some('b') => result.push('\x08'),
                    Some('f') => result.push('\x0c'),
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some(_) => return Err(ParseError {
                        message: "Invalid escape sequence".to_string(),
                        position: self.pos,
                    }),
                    None => return Err(ParseError {
                        message: "Unexpected end of string".to_string(),
                        position: self.pos,
                    }),
                }
                self.consume_char();
            } else {
                result.push(c);
                self.consume_char();
            }
        }
        
        Err(ParseError {
            message: "Unterminated string".to_string(),
            position: self.pos,
        })
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        if !self.consume_char('[') {
            return Err(ParseError {
                message: "Expected '['".to_string(),
                position: self.pos,
            });
        }
        
        self.skip_whitespace();
        let mut array = Vec::new();
        
        if self.peek_char() == Some(']') {
            self.consume_char();
            return Ok(JsonValue::Array(array));
        }
        
        loop {
            self.skip_whitespace();
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();
            
            match self.peek_char() {
                Some(',') => {
                    self.consume_char();
                    continue;
                }
                Some(']') => {
                    self.consume_char();
                    break;
                }
                _ => return Err(ParseError {
                    message: "Expected ',' or ']'".to_string(),
                    position: self.pos,
                }),
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        if !self.consume_char('{') {
            return Err(ParseError {
                message: "Expected '{'".to_string(),
                position: self.pos,
            });
        }
        
        self.skip_whitespace();
        let mut object = HashMap::new();
        
        if self.peek_char() == Some('}') {
            self.consume_char();
            return Ok(JsonValue::Object(object));
        }
        
        loop {
            self.skip_whitespace();
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => unreachable!(),
            };
            
            self.skip_whitespace();
            if !self.consume_char(':') {
                return Err(ParseError {
                    message: "Expected ':'".to_string(),
                    position: self.pos,
                });
            }
            
            self.skip_whitespace();
            let value = self.parse_value()?;
            object.insert(key, value);
            
            self.skip_whitespace();
            match self.peek_char() {
                Some(',') => {
                    self.consume_char();
                    continue;
                }
                Some('}') => {
                    self.consume_char();
                    break;
                }
                _ => return Err(ParseError {
                    message: "Expected ',' or '}'".to_string(),
                    position: self.pos,
                }),
            }
        }
        
        Ok(JsonValue::Object(object))
    }

    fn peek_char(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn consume_str(&mut self, s: &str) -> bool {
        let end = self.pos + s.len();
        if end <= self.input.len() {
            let slice: String = self.input[self.pos..end].iter().collect();
            if slice == s {
                self.pos = end;
                return true;
            }
        }
        false
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
}

pub fn validate_json(input: &str) -> Result<JsonValue, ParseError> {
    let mut parser = JsonParser::new(input);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let json = r#"{"name": "test", "value": 42, "active": true, "tags": ["rust", "json"]}"#;
        let result = validate_json(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_json() {
        let json = r#"{"name": "test", "value": 42,}"#;
        let result = validate_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_object() {
        let json = r#"{}"#;
        let result = validate_json(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_array() {
        let json = r#"[]"#;
        let result = validate_json(json);
        assert!(result.is_ok());
    }
}