
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Boolean(bool),
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
            't' | 'f' => self.parse_boolean(),
            '"' => self.parse_string(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            '-' | '0'..='9' => self.parse_number(),
            _ => Err(format!("Unexpected character: {}", self.input[self.position])),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.position + 3 < self.input.len() 
            && self.input[self.position..self.position + 4].iter().collect::<String>() == "null" {
            self.position += 4;
            Ok(JsonValue::Null)
        } else {
            Err("Expected 'null'".to_string())
        }
    }

    fn parse_boolean(&mut self) -> Result<JsonValue, String> {
        if self.position + 3 < self.input.len() 
            && self.input[self.position..self.position + 4].iter().collect::<String>() == "true" {
            self.position += 4;
            Ok(JsonValue::Boolean(true))
        } else if self.position + 4 < self.input.len() 
            && self.input[self.position..self.position + 5].iter().collect::<String>() == "false" {
            self.position += 5;
            Ok(JsonValue::Boolean(false))
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
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
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
            
            match self.input[self.position] {
                ',' => {
                    self.position += 1;
                    self.skip_whitespace();
                }
                ']' => {
                    self.position += 1;
                    break;
                }
                _ => return Err("Expected ',' or ']' in array".to_string()),
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.position += 1; // Skip '{'
        self.skip_whitespace();
        
        let mut object = HashMap::new();
        
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
            
            match self.input[self.position] {
                ',' => {
                    self.position += 1;
                    self.skip_whitespace();
                }
                '}' => {
                    self.position += 1;
                    break;
                }
                _ => return Err("Expected ',' or '}' in object".to_string()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        assert_eq!(parse_json("null").unwrap(), JsonValue::Null);
    }

    #[test]
    fn test_parse_boolean() {
        assert_eq!(parse_json("true").unwrap(), JsonValue::Boolean(true));
        assert_eq!(parse_json("false").unwrap(), JsonValue::Boolean(false));
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_json("42").unwrap(), JsonValue::Number(42.0));
        assert_eq!(parse_json("-3.14").unwrap(), JsonValue::Number(-3.14));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_json("\"hello\"").unwrap(), JsonValue::String("hello".to_string()));
    }

    #[test]
    fn test_parse_array() {
        let result = parse_json("[1, 2, 3]").unwrap();
        match result {
            JsonValue::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], JsonValue::Number(1.0));
                assert_eq!(arr[1], JsonValue::Number(2.0));
                assert_eq!(arr[2], JsonValue::Number(3.0));
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_parse_object() {
        let result = parse_json(r#"{"name": "test", "value": 42}"#).unwrap();
        match result {
            JsonValue::Object(obj) => {
                assert_eq!(obj.len(), 2);
                assert_eq!(obj.get("name"), Some(&JsonValue::String("test".to_string())));
                assert_eq!(obj.get("value"), Some(&JsonValue::Number(42.0)));
            }
            _ => panic!("Expected object"),
        }
    }
}