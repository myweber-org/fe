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
pub enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
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
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }

    fn consume(&mut self, expected: char) -> Result<(), String> {
        self.skip_whitespace();
        if let Some(ch) = self.peek() {
            if ch == expected {
                self.position += 1;
                Ok(())
            } else {
                Err(format!("Expected '{}', found '{}'", expected, ch))
            }
        } else {
            Err("Unexpected end of input".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.consume('"')?;
        let mut result = String::new();
        
        while self.position < self.input.len() {
            let ch = self.input[self.position];
            self.position += 1;
            
            match ch {
                '"' => return Ok(result),
                '\\' => {
                    if self.position >= self.input.len() {
                        return Err("Unterminated escape sequence".to_string());
                    }
                    let escaped = self.input[self.position];
                    self.position += 1;
                    match escaped {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        '/' => result.push('/'),
                        'b' => result.push('\x08'),
                        'f' => result.push('\x0c'),
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        _ => return Err(format!("Invalid escape sequence: \\{}", escaped)),
                    }
                }
                _ => result.push(ch),
            }
        }
        
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<f64, String> {
        let start = self.position;
        while self.position < self.input.len() {
            let ch = self.input[self.position];
            if ch.is_digit(10) || ch == '.' || ch == '-' || ch == '+' || ch == 'e' || ch == 'E' {
                self.position += 1;
            } else {
                break;
            }
        }
        
        let num_str: String = self.input[start..self.position].iter().collect();
        num_str.parse::<f64>()
            .map_err(|e| format!("Invalid number '{}': {}", num_str, e))
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => {
                let s = self.parse_string()?;
                Ok(JsonValue::String(s))
            }
            Some(ch) if ch.is_digit(10) || ch == '-' => {
                let n = self.parse_number()?;
                Ok(JsonValue::Number(n))
            }
            Some('t') => {
                if self.position + 3 < self.input.len() 
                    && self.input[self.position..self.position+4].iter().collect::<String>() == "true" {
                    self.position += 4;
                    Ok(JsonValue::Bool(true))
                } else {
                    Err("Expected 'true'".to_string())
                }
            }
            Some('f') => {
                if self.position + 4 < self.input.len() 
                    && self.input[self.position..self.position+5].iter().collect::<String>() == "false" {
                    self.position += 5;
                    Ok(JsonValue::Bool(false))
                } else {
                    Err("Expected 'false'".to_string())
                }
            }
            Some('n') => {
                if self.position + 3 < self.input.len() 
                    && self.input[self.position..self.position+4].iter().collect::<String>() == "null" {
                    self.position += 4;
                    Ok(JsonValue::Null)
                } else {
                    Err("Expected 'null'".to_string())
                }
            }
            Some(ch) => Err(format!("Unexpected character: {}", ch)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume('{')?;
        self.skip_whitespace();
        
        let mut map = HashMap::new();
        
        if let Some('}') = self.peek() {
            self.consume('}')?;
            return Ok(JsonValue::Object(map));
        }
        
        loop {
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.consume(':')?;
            let value = self.parse_value()?;
            map.insert(key, value);
            
            self.skip_whitespace();
            if let Some(',') = self.peek() {
                self.consume(',')?;
                self.skip_whitespace();
            } else {
                break;
            }
        }
        
        self.consume('}')?;
        Ok(JsonValue::Object(map))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume('[')?;
        self.skip_whitespace();
        
        let mut arr = Vec::new();
        
        if let Some(']') = self.peek() {
            self.consume(']')?;
            return Ok(JsonValue::Array(arr));
        }
        
        loop {
            let value = self.parse_value()?;
            arr.push(value);
            
            self.skip_whitespace();
            if let Some(',') = self.peek() {
                self.consume(',')?;
                self.skip_whitespace();
            } else {
                break;
            }
        }
        
        self.consume(']')?;
        Ok(JsonValue::Array(arr))
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
            return Err("Trailing characters after JSON value".to_string());
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_object() {
        let json = r#"{"name": "John", "age": 30, "active": true}"#;
        let mut parser = JsonParser::new(json);
        let result = parser.parse().unwrap();
        
        if let JsonValue::Object(map) = result {
            assert_eq!(map.get("name"), Some(&JsonValue::String("John".to_string())));
            assert_eq!(map.get("age"), Some(&JsonValue::Number(30.0)));
            assert_eq!(map.get("active"), Some(&JsonValue::Bool(true)));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_parse_array() {
        let json = r#"[1, 2, 3, "four", false]"#;
        let mut parser = JsonParser::new(json);
        let result = parser.parse().unwrap();
        
        if let JsonValue::Array(arr) = result {
            assert_eq!(arr.len(), 5);
            assert_eq!(arr[0], JsonValue::Number(1.0));
            assert_eq!(arr[3], JsonValue::String("four".to_string()));
            assert_eq!(arr[4], JsonValue::Bool(false));
        } else {
            panic!("Expected array");
        }
    }
}