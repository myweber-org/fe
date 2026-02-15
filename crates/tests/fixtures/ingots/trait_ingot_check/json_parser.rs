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
                let s = self.input[start..self.pos].to_string();
                self.pos += 1;
                return Ok(JsonValue::String(s));
            }
            self.pos += 1;
        }

        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        let mut has_dot = false;

        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            if c == '.' {
                if has_dot {
                    return Err("Invalid number format".to_string());
                }
                has_dot = true;
            } else if !c.is_digit(10) && !(c == '-' && self.pos == start) {
                break;
            }
            self.pos += 1;
        }

        let num_str = &self.input[start..self.pos];
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number".to_string()),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // Skip '['
        self.skip_whitespace();
        let mut array = Vec::new();

        if self.pos < self.input.len() && self.input.chars().nth(self.pos).unwrap() == ']' {
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

        if self.pos < self.input.len() && self.input.chars().nth(self.pos).unwrap() == '}' {
            self.pos += 1;
            return Ok(JsonValue::Object(object));
        }

        loop {
            self.skip_whitespace();
            let key = match self.parse_value()? {
                JsonValue::String(s) => s,
                _ => return Err("Object key must be a string".to_string()),
            };

            self.skip_whitespace();
            if self.pos >= self.input.len() || self.input.chars().nth(self.pos).unwrap() != ':' {
                return Err("Expected ':' after object key".to_string());
            }
            self.pos += 1;

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
    fn test_parse_string() {
        let mut parser = JsonParser::new(r#""hello world""#);
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::String("hello world".to_string()))
        );
    }

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(42.0)));

        let mut parser = JsonParser::new("-3.14");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(-3.14)));
    }

    #[test]
    fn test_parse_array() {
        let mut parser = JsonParser::new("[1, 2, 3]");
        assert_eq!(
            parser.parse(),
            Ok(JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Number(2.0),
                JsonValue::Number(3.0)
            ]))
        );
    }

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new(r#"{"key": "value"}"#);
        let mut expected = HashMap::new();
        expected.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(expected)));
    }
}