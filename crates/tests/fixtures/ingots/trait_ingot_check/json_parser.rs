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
                        'b' => result.push('\u{0008}'),
                        'f' => result.push('\u{000C}'),
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
        let mut has_dot = false;
        
        while self.position < self.input.len() {
            let ch = self.input[self.position];
            if ch.is_ascii_digit() {
                self.position += 1;
            } else if ch == '.' && !has_dot {
                has_dot = true;
                self.position += 1;
            } else {
                break;
            }
        }
        
        let number_str: String = self.input[start..self.position].iter().collect();
        number_str.parse().map_err(|e| format!("Invalid number: {}", e))
    }

    fn parse_array(&mut self) -> Result<Vec<JsonValue>, String> {
        self.consume('[')?;
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
        if self.peek() == Some(']') {
            self.consume(']')?;
            return Ok(array);
        }
        
        loop {
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            if self.peek() == Some(']') {
                self.consume(']')?;
                break;
            }
            
            self.consume(',')?;
        }
        
        Ok(array)
    }

    fn parse_object(&mut self) -> Result<HashMap<String, JsonValue>, String> {
        self.consume('{')?;
        self.skip_whitespace();
        
        let mut object = HashMap::new();
        
        if self.peek() == Some('}') {
            self.consume('}')?;
            return Ok(object);
        }
        
        loop {
            let key = self.parse_string()?;
            self.consume(':')?;
            let value = self.parse_value()?;
            object.insert(key, value);
            
            self.skip_whitespace();
            if self.peek() == Some('}') {
                self.consume('}')?;
                break;
            }
            
            self.consume(',')?;
        }
        
        Ok(object)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        
        match self.peek() {
            Some('n') => {
                if self.position + 3 < self.input.len() 
                    && self.input[self.position..self.position + 4].iter().collect::<String>() == "null" {
                    self.position += 4;
                    Ok(JsonValue::Null)
                } else {
                    Err("Expected 'null'".to_string())
                }
            }
            Some('t') => {
                if self.position + 3 < self.input.len() 
                    && self.input[self.position..self.position + 4].iter().collect::<String>() == "true" {
                    self.position += 4;
                    Ok(JsonValue::Bool(true))
                } else {
                    Err("Expected 'true'".to_string())
                }
            }
            Some('f') => {
                if self.position + 4 < self.input.len() 
                    && self.input[self.position..self.position + 5].iter().collect::<String>() == "false" {
                    self.position += 5;
                    Ok(JsonValue::Bool(false))
                } else {
                    Err("Expected 'false'".to_string())
                }
            }
            Some('"') => {
                let s = self.parse_string()?;
                Ok(JsonValue::String(s))
            }
            Some('[') => {
                let arr = self.parse_array()?;
                Ok(JsonValue::Array(arr))
            }
            Some('{') => {
                let obj = self.parse_object()?;
                Ok(JsonValue::Object(obj))
            }
            Some(ch) if ch.is_ascii_digit() || ch == '-' || ch == '.' => {
                let num = self.parse_number()?;
                Ok(JsonValue::Number(num))
            }
            Some(ch) => Err(format!("Unexpected character: {}", ch)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
            Err("Trailing characters after JSON value".to_string())
        } else {
            Ok(result)
        }
    }
}

pub fn parse_json(input: &str) -> Result<JsonValue, String> {
    let mut parser = JsonParser::new(input);
    parser.parse()
}