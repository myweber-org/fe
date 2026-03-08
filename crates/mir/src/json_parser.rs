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
                if ch == ',' {
                    self.pos += 1;
                    continue;
                } else if ch == ']' {
                    self.pos += 1;
                    break;
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
                if ch == ',' {
                    self.pos += 1;
                    continue;
                } else if ch == '}' {
                    self.pos += 1;
                    break;
                } else {
                    return Err(format!("Expected ',' or '}}', found '{}'", ch));
                }
            } else {
                return Err("Unexpected end of input".to_string());
            }
        }
        Ok(JsonValue::Object(map))
    }

    fn parse_keyword(&mut self, keyword: &str, value: JsonValue) -> Result<JsonValue, String> {
        for expected_char in keyword.chars() {
            if let Some(ch) = self.peek() {
                if ch == expected_char {
                    self.pos += 1;
                } else {
                    return Err(format!("Expected keyword '{}'", keyword));
                }
            } else {
                return Err("Unexpected end of input".to_string());
            }
        }
        Ok(value)
    }

    pub fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        if let Some(ch) = self.peek() {
            match ch {
                '"' => self.parse_string(),
                '[' => self.parse_array(),
                '{' => self.parse_object(),
                't' => self.parse_keyword("true", JsonValue::Bool(true)),
                'f' => self.parse_keyword("false", JsonValue::Bool(false)),
                'n' => self.parse_keyword("null", JsonValue::Null),
                '-' | '0'..='9' => self.parse_number(),
                _ => Err(format!("Unexpected character: '{}'", ch)),
            }
        } else {
            Err("Unexpected end of input".to_string())
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            Err("Trailing characters after JSON value".to_string())
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

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new(r#"{"key": "value"}"#);
        let mut map = HashMap::new();
        map.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(map)));
    }
}