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
    input: String,
    pos: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Self {
        JsonParser {
            input: input.to_string(),
            pos: 0,
        }
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

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        
        if self.pos >= self.input.len() {
            return Err("Unexpected end of input".to_string());
        }

        let c = self.input.chars().nth(self.pos).unwrap();
        
        match c {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_bool(),
            '"' => self.parse_string(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            '-' | '0'..='9' => self.parse_number(),
            _ => Err(format!("Unexpected character: {}", c)),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.input[self.pos..].starts_with("null") {
            self.pos += 4;
            Ok(JsonValue::Null)
        } else {
            Err("Expected 'null'".to_string())
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.input[self.pos..].starts_with("true") {
            self.pos += 4;
            Ok(JsonValue::Bool(true))
        } else if self.input[self.pos..].starts_with("false") {
            self.pos += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err("Expected 'true' or 'false'".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // Skip opening quote
        let start = self.pos;
        let mut escaped = false;
        
        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                let result = self.input[start..self.pos].to_string();
                self.pos += 1; // Skip closing quote
                return Ok(JsonValue::String(result));
            }
            
            self.pos += 1;
        }
        
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        
        if self.input.chars().nth(self.pos) == Some('-') {
            self.pos += 1;
        }
        
        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if c.is_digit(10) || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-' {
                self.pos += 1;
            } else {
                break;
            }
        }
        
        let num_str = &self.input[start..self.pos];
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(format!("Invalid number: {}", num_str)),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
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
                return Err("Unterminated array".to_string());
            }
            
            let c = self.input.chars().nth(self.pos).unwrap();
            if c == ']' {
                self.pos += 1;
                break;
            } else if c == ',' {
                self.pos += 1;
                self.skip_whitespace();
            } else {
                return Err(format!("Expected ',' or ']', found: {}", c));
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // Skip '{'
        self.skip_whitespace();
        
        let mut object = HashMap::new();
        
        if self.pos < self.input.len() && self.input.chars().nth(self.pos) == Some('}') {
            self.pos += 1;
            return Ok(JsonValue::Object(object));
        }
        
        loop {
            self.skip_whitespace();
            
            if self.pos >= self.input.len() || self.input.chars().nth(self.pos) != Some('"') {
                return Err("Expected string key".to_string());
            }
            
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err("Expected string key".to_string()),
            };
            
            self.skip_whitespace();
            
            if self.pos >= self.input.len() || self.input.chars().nth(self.pos) != Some(':') {
                return Err("Expected ':'".to_string());
            }
            
            self.pos += 1; // Skip ':'
            
            let value = self.parse_value()?;
            object.insert(key, value);
            
            self.skip_whitespace();
            if self.pos >= self.input.len() {
                return Err("Unterminated object".to_string());
            }
            
            let c = self.input.chars().nth(self.pos).unwrap();
            if c == '}' {
                self.pos += 1;
                break;
            } else if c == ',' {
                self.pos += 1;
                self.skip_whitespace();
            } else {
                return Err(format!("Expected ',' or '}}', found: {}", c));
            }
        }
        
        Ok(JsonValue::Object(object))
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        
        if self.pos < self.input.len() {
            return Err("Trailing characters after JSON value".to_string());
        }
        
        Ok(result)
    }
}

pub fn parse_json(json_str: &str) -> Result<JsonValue, String> {
    let mut parser = JsonParser::new(json_str);
    parser.parse()
}