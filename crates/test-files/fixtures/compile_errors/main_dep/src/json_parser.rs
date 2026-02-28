use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

struct JsonParser {
    input: String,
    pos: usize,
}

impl JsonParser {
    fn new(input: &str) -> Self {
        JsonParser {
            input: input.to_string(),
            pos: 0,
        }
    }

    fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        let value = self.parse_value()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err("Unexpected trailing characters".to_string());
        }
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
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
        if self.consume("null") {
            Ok(JsonValue::Null)
        } else {
            Err("Expected 'null'".to_string())
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.consume("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err("Expected boolean".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.consume_char(); // consume opening quote
        let mut result = String::new();
        while let Some(c) = self.peek_char() {
            if c == '"' {
                self.consume_char(); // consume closing quote
                return Ok(JsonValue::String(result));
            }
            result.push(c);
            self.consume_char();
        }
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if c.is_digit(10) || c == '.' || c == '-' || c == 'e' || c == 'E' {
                self.consume_char();
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
        self.consume_char(); // consume '['
        let mut array = Vec::new();
        self.skip_whitespace();

        if self.peek_char() == Some(']') {
            self.consume_char();
            return Ok(JsonValue::Array(array));
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();

            match self.peek_char() {
                Some(',') => {
                    self.consume_char();
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.consume_char();
                    break;
                }
                _ => return Err("Expected ',' or ']'".to_string()),
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume_char(); // consume '{'
        let mut map = HashMap::new();
        self.skip_whitespace();

        if self.peek_char() == Some('}') {
            self.consume_char();
            return Ok(JsonValue::Object(map));
        }

        loop {
            self.skip_whitespace();
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err("Expected string key".to_string()),
            };
            self.skip_whitespace();

            if self.peek_char() != Some(':') {
                return Err("Expected ':'".to_string());
            }
            self.consume_char();
            self.skip_whitespace();

            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();

            match self.peek_char() {
                Some(',') => {
                    self.consume_char();
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.consume_char();
                    break;
                }
                _ => return Err("Expected ',' or '}'".to_string()),
            }
        }

        Ok(JsonValue::Object(map))
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.consume_char();
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }

    fn consume_char(&mut self) -> Option<char> {
        let c = self.peek_char();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn consume(&mut self, expected: &str) -> bool {
        if self.input[self.pos..].starts_with(expected) {
            self.pos += expected.len();
            true
        } else {
            false
        }
    }
}

fn main() {
    let json_str = r#"{"name": "test", "value": 42, "active": true}"#;
    let mut parser = JsonParser::new(json_str);
    match parser.parse() {
        Ok(value) => println!("Parsed: {:?}", value),
        Err(e) => println!("Error: {}", e),
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
            Err(format!("Expected '{}', found EOF", expected))
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
                        'u' => return Err("Unicode escape not implemented".to_string()),
                        _ => return Err(format!("Invalid escape character: {}", escaped)),
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
        num_str.parse().map_err(|e| format!("Invalid number: {}", e))
    }

    fn parse_array(&mut self) -> Result<Vec<JsonValue>, String> {
        self.consume('[')?;
        self.skip_whitespace();
        
        if let Some(ch) = self.peek() {
            if ch == ']' {
                self.position += 1;
                return Ok(Vec::new());
            }
        }
        
        let mut array = Vec::new();
        loop {
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                if ch == ']' {
                    self.position += 1;
                    break;
                } else if ch == ',' {
                    self.position += 1;
                    continue;
                } else {
                    return Err(format!("Expected ',' or ']', found '{}'", ch));
                }
            } else {
                return Err("Unexpected EOF in array".to_string());
            }
        }
        
        Ok(array)
    }

    fn parse_object(&mut self) -> Result<HashMap<String, JsonValue>, String> {
        self.consume('{')?;
        self.skip_whitespace();
        
        if let Some(ch) = self.peek() {
            if ch == '}' {
                self.position += 1;
                return Ok(HashMap::new());
            }
        }
        
        let mut object = HashMap::new();
        loop {
            let key = self.parse_string()?;
            self.consume(':')?;
            let value = self.parse_value()?;
            object.insert(key, value);
            
            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                if ch == '}' {
                    self.position += 1;
                    break;
                } else if ch == ',' {
                    self.position += 1;
                    continue;
                } else {
                    return Err(format!("Expected ',' or '}}', found '{}'", ch));
                }
            } else {
                return Err("Unexpected EOF in object".to_string());
            }
        }
        
        Ok(object)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        
        if let Some(ch) = self.peek() {
            match ch {
                'n' => {
                    if self.position + 3 < self.input.len() 
                        && self.input[self.position..self.position + 4].iter().collect::<String>() == "null" {
                        self.position += 4;
                        Ok(JsonValue::Null)
                    } else {
                        Err("Invalid null value".to_string())
                    }
                }
                't' => {
                    if self.position + 3 < self.input.len() 
                        && self.input[self.position..self.position + 4].iter().collect::<String>() == "true" {
                        self.position += 4;
                        Ok(JsonValue::Bool(true))
                    } else {
                        Err("Invalid boolean value".to_string())
                    }
                }
                'f' => {
                    if self.position + 4 < self.input.len() 
                        && self.input[self.position..self.position + 5].iter().collect::<String>() == "false" {
                        self.position += 5;
                        Ok(JsonValue::Bool(false))
                    } else {
                        Err("Invalid boolean value".to_string())
                    }
                }
                '"' => {
                    let s = self.parse_string()?;
                    Ok(JsonValue::String(s))
                }
                '[' => {
                    let arr = self.parse_array()?;
                    Ok(JsonValue::Array(arr))
                }
                '{' => {
                    let obj = self.parse_object()?;
                    Ok(JsonValue::Object(obj))
                }
                '-' | '0'..='9' => {
                    let num = self.parse_number()?;
                    Ok(JsonValue::Number(num))
                }
                _ => Err(format!("Unexpected character: {}", ch)),
            }
        } else {
            Err("Unexpected EOF".to_string())
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

pub fn parse_json(json_str: &str) -> Result<JsonValue, String> {
    let mut parser = JsonParser::new(json_str);
    parser.parse()
}