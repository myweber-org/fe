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

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        
        if self.position >= self.input.len() {
            return Err("Unexpected end of input".to_string());
        }

        match self.input[self.position] {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_bool(),
            '"' => self.parse_string(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            '-' | '0'..='9' => self.parse_number(),
            _ => Err(format!("Unexpected character: {}", self.input[self.position])),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.position + 3 < self.input.len() 
            && self.input[self.position..self.position+4].iter().collect::<String>() == "null" {
            self.position += 4;
            Ok(JsonValue::Null)
        } else {
            Err("Expected 'null'".to_string())
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.position + 3 < self.input.len() 
            && self.input[self.position..self.position+4].iter().collect::<String>() == "true" {
            self.position += 4;
            Ok(JsonValue::Bool(true))
        } else if self.position + 4 < self.input.len() 
            && self.input[self.position..self.position+5].iter().collect::<String>() == "false" {
            self.position += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err("Expected boolean value".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.position += 1; // Skip opening quote
        let mut result = String::new();
        
        while self.position < self.input.len() && self.input[self.position] != '"' {
            result.push(self.input[self.position]);
            self.position += 1;
        }
        
        if self.position < self.input.len() && self.input[self.position] == '"' {
            self.position += 1;
            Ok(JsonValue::String(result))
        } else {
            Err("Unterminated string".to_string())
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.position;
        
        if self.input[self.position] == '-' {
            self.position += 1;
        }
        
        while self.position < self.input.len() && self.input[self.position].is_ascii_digit() {
            self.position += 1;
        }
        
        if self.position < self.input.len() && self.input[self.position] == '.' {
            self.position += 1;
            while self.position < self.input.len() && self.input[self.position].is_ascii_digit() {
                self.position += 1;
            }
        }
        
        let number_str: String = self.input[start..self.position].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number format".to_string()),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.position += 1; // Skip '['
        let mut array = Vec::new();
        
        self.skip_whitespace();
        
        if self.position < self.input.len() && self.input[self.position] == ']' {
            self.position += 1;
            return Ok(JsonValue::Array(array));
        }
        
        loop {
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            
            if self.position >= self.input.len() {
                return Err("Unterminated array".to_string());
            }
            
            if self.input[self.position] == ']' {
                self.position += 1;
                break;
            } else if self.input[self.position] == ',' {
                self.position += 1;
                self.skip_whitespace();
            } else {
                return Err("Expected ',' or ']' in array".to_string());
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.position += 1; // Skip '{'
        let mut object = HashMap::new();
        
        self.skip_whitespace();
        
        if self.position < self.input.len() && self.input[self.position] == '}' {
            self.position += 1;
            return Ok(JsonValue::Object(object));
        }
        
        loop {
            self.skip_whitespace();
            
            if self.position >= self.input.len() || self.input[self.position] != '"' {
                return Err("Expected string key in object".to_string());
            }
            
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err("Expected string key".to_string()),
            };
            
            self.skip_whitespace();
            
            if self.position >= self.input.len() || self.input[self.position] != ':' {
                return Err("Expected ':' after object key".to_string());
            }
            
            self.position += 1;
            let value = self.parse_value()?;
            object.insert(key, value);
            
            self.skip_whitespace();
            
            if self.position >= self.input.len() {
                return Err("Unterminated object".to_string());
            }
            
            if self.input[self.position] == '}' {
                self.position += 1;
                break;
            } else if self.input[self.position] == ',' {
                self.position += 1;
                self.skip_whitespace();
            } else {
                return Err("Expected ',' or '}' in object".to_string());
            }
        }
        
        Ok(JsonValue::Object(object))
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

pub fn parse_json(json_str: &str) -> Result<JsonValue, String> {
    let mut parser = JsonParser::new(json_str);
    parser.parse()
}