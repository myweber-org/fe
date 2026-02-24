
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
}use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
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

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn consume(&mut self, expected: char) -> Result<(), String> {
        match self.peek() {
            Some(ch) if ch == expected => {
                self.pos += 1;
                Ok(())
            }
            Some(ch) => Err(format!("Expected '{}', found '{}'", expected, ch)),
            None => Err(format!("Expected '{}', found EOF", expected)),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.consume('"')?;
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    self.pos += 1;
                    return Ok(result);
                }
                '\\' => {
                    self.pos += 1;
                    let escaped = self.peek().ok_or("Unexpected EOF after escape")?;
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
                    self.pos += 1;
                }
                _ => {
                    result.push(ch);
                    self.pos += 1;
                }
            }
        }
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<f64, String> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '+' || ch == 'e' || ch == 'E' {
                self.pos += 1;
            } else {
                break;
            }
        }
        let num_str: String = self.input[start..self.pos].iter().collect();
        num_str.parse().map_err(|e| format!("Invalid number: {}", e))
    }

    fn parse_array(&mut self) -> Result<Vec<JsonValue>, String> {
        self.consume('[')?;
        self.skip_whitespace();
        let mut array = Vec::new();
        if let Some(']') = self.peek() {
            self.pos += 1;
            return Ok(array);
        }
        loop {
            self.skip_whitespace();
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.pos += 1;
                    continue;
                }
                Some(']') => {
                    self.pos += 1;
                    break;
                }
                _ => return Err("Expected ',' or ']' in array".to_string()),
            }
        }
        Ok(array)
    }

    fn parse_object(&mut self) -> Result<HashMap<String, JsonValue>, String> {
        self.consume('{')?;
        self.skip_whitespace();
        let mut map = HashMap::new();
        if let Some('}') = self.peek() {
            self.pos += 1;
            return Ok(map);
        }
        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.consume(':')?;
            self.skip_whitespace();
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.pos += 1;
                    continue;
                }
                Some('}') => {
                    self.pos += 1;
                    break;
                }
                _ => return Err("Expected ',' or '}' in object".to_string()),
            }
        }
        Ok(map)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.peek() {
            Some('n') => {
                self.pos += 1;
                self.consume('u')?;
                self.consume('l')?;
                self.consume('l')?;
                Ok(JsonValue::Null)
            }
            Some('t') => {
                self.pos += 1;
                self.consume('r')?;
                self.consume('u')?;
                self.consume('e')?;
                Ok(JsonValue::Bool(true))
            }
            Some('f') => {
                self.pos += 1;
                self.consume('a')?;
                self.consume('l')?;
                self.consume('s')?;
                self.consume('e')?;
                Ok(JsonValue::Bool(false))
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
            Some(ch) if ch.is_ascii_digit() || ch == '-' => {
                let num = self.parse_number()?;
                Ok(JsonValue::Number(num))
            }
            Some(ch) => Err(format!("Unexpected character: {}", ch)),
            None => Err("Unexpected EOF".to_string()),
        }
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

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Bool(b) => write!(f, "{}", b),
            JsonValue::Number(n) => write!(f, "{}", n),
            JsonValue::String(s) => write!(f, "\"{}\"", s.escape_default()),
            JsonValue::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            JsonValue::Object(obj) => {
                write!(f, "{{")?;
                for (i, (key, value)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", key.escape_default(), value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

pub fn pretty_print_json(json: &JsonValue, indent: usize) -> String {
    fn pretty_print_helper(json: &JsonValue, indent_level: usize, spaces: usize) -> String {
        let indent_str = " ".repeat(indent_level * spaces);
        match json {
            JsonValue::Null => "null".to_string(),
            JsonValue::Bool(b) => b.to_string(),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::String(s) => format!("\"{}\"", s.escape_default()),
            JsonValue::Array(arr) => {
                if arr.is_empty() {
                    return "[]".to_string();
                }
                let mut result = "[\n".to_string();
                for (i, item) in arr.iter().enumerate() {
                    result.push_str(&format!(
                        "{}{}",
                        " ".repeat((indent_level + 1) * spaces),
                        pretty_print_helper(item, indent_level + 1, spaces)
                    ));
                    if i < arr.len() - 1 {
                        result.push_str(",\n");
                    } else {
                        result.push('\n');
                    }
                }
                result.push_str(&format!("{}{}", indent_str, "]"));
                result
            }
            JsonValue::Object(obj) => {
                if obj.is_empty() {
                    return "{}".to_string();
                }
                let mut result = "{\n".to_string();
                let mut entries: Vec<_> = obj.iter().collect();
                entries.sort_by(|a, b| a.0.cmp(b.0));
                for (i, (key, value)) in entries.iter().enumerate() {
                    result.push_str(&format!(
                        "{}\"{}\": {}",
                        " ".repeat((indent_level + 1) * spaces),
                        key.escape_default(),
                        pretty_print_helper(value, indent_level + 1, spaces)
                    ));
                    if i < entries.len() - 1 {
                        result.push_str(",\n");
                    } else {
                        result.push('\n');
                    }
                }
                result.push_str(&format!("{}{}", indent_str, "}"));
                result
            }
        }
    }
    pretty_print_helper(json, 0, indent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_json() {
        let json_str = r#"{"name": "test", "value": 42, "active": true}"#;
        let mut parser = JsonParser::new(json_str);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_array() {
        let json_str = r#"[1, 2, 3, "four", false]"#;
        let mut parser = JsonParser::new(json_str);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_pretty_print() {
        let json_str = r#"{"b": 2, "a": 1}"#;
        let mut parser = JsonParser::new(json_str);
        let json = parser.parse().unwrap();
        let pretty = pretty_print_json(&json, 2);
        assert!(pretty.contains("\"a\": 1"));
        assert!(pretty.contains("\"b\": 2"));
    }
}