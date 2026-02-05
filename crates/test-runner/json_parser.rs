use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum JsonValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
}

struct JsonParser {
    input: Vec<char>,
    position: usize,
}

impl JsonParser {
    fn new(input: &str) -> Self {
        JsonParser {
            input: input.chars().collect(),
            position: 0,
        }
    }

    fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        self.parse_value()
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        match self.peek() {
            Some('"') => self.parse_string(),
            Some('t') | Some('f') => self.parse_boolean(),
            Some('n') => self.parse_null(),
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            _ => Err(format!("Unexpected character at position {}", self.position)),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.consume('"')?;
        let mut result = String::new();
        while let Some(c) = self.next_char() {
            match c {
                '"' => break,
                '\\' => {
                    if let Some(escaped) = self.next_char() {
                        result.push(match escaped {
                            'n' => '\n',
                            't' => '\t',
                            'r' => '\r',
                            '\\' => '\\',
                            '"' => '"',
                            _ => return Err(format!("Invalid escape sequence: \\{}", escaped)),
                        });
                    }
                }
                _ => result.push(c),
            }
        }
        Ok(JsonValue::String(result))
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.position;
        while let Some(c) = self.peek() {
            if c.is_digit(10) || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-' {
                self.position += 1;
            } else {
                break;
            }
        }
        let num_str: String = self.input[start..self.position].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(format!("Invalid number: {}", num_str)),
        }
    }

    fn parse_boolean(&mut self) -> Result<JsonValue, String> {
        if self.consume_str("true") {
            Ok(JsonValue::Boolean(true))
        } else if self.consume_str("false") {
            Ok(JsonValue::Boolean(false))
        } else {
            Err("Expected boolean value".to_string())
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.consume_str("null") {
            Ok(JsonValue::Null)
        } else {
            Err("Expected null value".to_string())
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume('{')?;
        self.skip_whitespace();
        let mut map = HashMap::new();
        if self.peek() == Some('}') {
            self.position += 1;
            return Ok(JsonValue::Object(map));
        }
        loop {
            self.skip_whitespace();
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err("Expected string key".to_string()),
            };
            self.skip_whitespace();
            self.consume(':')?;
            self.skip_whitespace();
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            if self.peek() == Some('}') {
                self.position += 1;
                break;
            }
            self.consume(',')?;
        }
        Ok(JsonValue::Object(map))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.consume('[')?;
        self.skip_whitespace();
        let mut arr = Vec::new();
        if self.peek() == Some(']') {
            self.position += 1;
            return Ok(JsonValue::Array(arr));
        }
        loop {
            self.skip_whitespace();
            arr.push(self.parse_value()?);
            self.skip_whitespace();
            if self.peek() == Some(']') {
                self.position += 1;
                break;
            }
            self.consume(',')?;
        }
        Ok(JsonValue::Array(arr))
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.position += 1;
        }
        c
    }

    fn consume(&mut self, expected: char) -> Result<(), String> {
        if self.peek() == Some(expected) {
            self.position += 1;
            Ok(())
        } else {
            Err(format!("Expected '{}' at position {}", expected, self.position))
        }
    }

    fn consume_str(&mut self, expected: &str) -> bool {
        let expected_chars: Vec<char> = expected.chars().collect();
        if self.position + expected_chars.len() <= self.input.len() {
            let slice = &self.input[self.position..self.position + expected_chars.len()];
            if slice == expected_chars {
                self.position += expected_chars.len();
                return true;
            }
        }
        false
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.position += 1;
            } else {
                break;
            }
        }
    }
}

fn extract_values(json: &str) -> Result<Vec<String>, String> {
    let mut parser = JsonParser::new(json);
    let parsed = parser.parse()?;
    let mut values = Vec::new();
    collect_strings(&parsed, &mut values);
    Ok(values)
}

fn collect_strings(value: &JsonValue, result: &mut Vec<String>) {
    match value {
        JsonValue::String(s) => result.push(s.clone()),
        JsonValue::Object(map) => {
            for v in map.values() {
                collect_strings(v, result);
            }
        }
        JsonValue::Array(arr) => {
            for v in arr {
                collect_strings(v, result);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let json = r#"{"name": "Alice", "age": 30, "active": true}"#;
        let mut parser = JsonParser::new(json);
        let result = parser.parse();
        assert!(result.is_ok());
        if let Ok(JsonValue::Object(map)) = result {
            assert_eq!(map.get("name"), Some(&JsonValue::String("Alice".to_string())));
            assert_eq!(map.get("age"), Some(&JsonValue::Number(30.0)));
            assert_eq!(map.get("active"), Some(&JsonValue::Boolean(true)));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_value_extraction() {
        let json = r#"{"user": {"name": "Bob", "tags": ["rust", "parser"]}}"#;
        let values = extract_values(json).unwrap();
        assert_eq!(values, vec!["Bob", "rust", "parser"]);
    }
}
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
            Err("Expected boolean value".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // Skip opening quote
        let start = self.pos;
        let mut escaped = false;
        let mut result = String::new();

        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            
            if escaped {
                match c {
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    '/' => result.push('/'),
                    'b' => result.push('\x08'),
                    'f' => result.push('\x0c'),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    _ => return Err(format!("Invalid escape sequence: \\{}", c)),
                }
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                self.pos += 1;
                return Ok(JsonValue::String(result));
            } else {
                result.push(c);
            }
            
            self.pos += 1;
        }

        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        let mut has_decimal = false;
        let mut has_exponent = false;

        while self.pos < self.input.len() {
            let c = self.input.chars().nth(self.pos).unwrap();
            
            match c {
                '-' | '+' => {
                    if self.pos != start && !has_exponent {
                        break;
                    }
                }
                'e' | 'E' => {
                    if has_exponent {
                        break;
                    }
                    has_exponent = true;
                }
                '.' => {
                    if has_decimal || has_exponent {
                        break;
                    }
                    has_decimal = true;
                }
                '0'..='9' => {}
                _ => break,
            }
            
            self.pos += 1;
        }

        let num_str = &self.input[start..self.pos];
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(format!("Invalid number: {}", num_str)),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // Skip '['
        let mut array = Vec::new();
        
        self.skip_whitespace();
        
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
        let mut object = HashMap::new();
        
        self.skip_whitespace();
        
        if self.pos < self.input.len() && self.input.chars().nth(self.pos).unwrap() == '}' {
            self.pos += 1;
            return Ok(JsonValue::Object(object));
        }

        loop {
            self.skip_whitespace();
            
            if self.input.chars().nth(self.pos).unwrap() != '"' {
                return Err("Expected string key".to_string());
            }
            
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => unreachable!(),
            };
            
            self.skip_whitespace();
            
            if self.pos >= self.input.len() || self.input.chars().nth(self.pos).unwrap() != ':' {
                return Err("Expected ':' after object key".to_string());
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
            return Err("Extra characters after JSON value".to_string());
        }
        
        Ok(result)
    }
}

pub fn parse_json(json_str: &str) -> Result<JsonValue, String> {
    let mut parser = JsonParser::new(json_str);
    parser.parse()
}