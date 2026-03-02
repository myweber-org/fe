use std::collections::HashMap;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
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
        self.input.get(self.pos).copied()
    }

    fn consume(&mut self, expected: char) -> Result<(), String> {
        self.skip_whitespace();
        if let Some(ch) = self.peek() {
            if ch == expected {
                self.pos += 1;
                return Ok(());
            }
        }
        Err(format!("Expected '{}' at position {}", expected, self.pos))
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.consume('"')?;
        let mut result = String::new();
        
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            self.pos += 1;
            
            match ch {
                '"' => break,
                '\\' => {
                    if self.pos >= self.input.len() {
                        return Err("Unterminated escape sequence".to_string());
                    }
                    let next = self.input[self.pos];
                    self.pos += 1;
                    match next {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        '/' => result.push('/'),
                        'b' => result.push('\x08'),
                        'f' => result.push('\x0c'),
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        _ => return Err(format!("Invalid escape sequence: \\{}", next)),
                    }
                }
                _ => result.push(ch),
            }
        }
        
        Ok(JsonValue::String(result))
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        let mut has_dot = false;
        
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            if ch.is_ascii_digit() {
                self.pos += 1;
            } else if ch == '.' && !has_dot {
                has_dot = true;
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
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
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
                if ch == ']' {
                    self.pos += 1;
                    break;
                } else if ch == ',' {
                    self.pos += 1;
                    continue;
                } else {
                    return Err(format!("Expected ',' or ']' at position {}", self.pos));
                }
            } else {
                return Err("Unexpected end of input in array".to_string());
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume('{')?;
        self.skip_whitespace();
        
        let mut object = HashMap::new();
        
        if let Some(ch) = self.peek() {
            if ch == '}' {
                self.pos += 1;
                return Ok(JsonValue::Object(object));
            }
        }
        
        loop {
            let key = match self.parse_value()? {
                JsonValue::String(s) => s,
                _ => return Err("Object key must be a string".to_string()),
            };
            
            self.consume(':')?;
            let value = self.parse_value()?;
            object.insert(key, value);
            
            self.skip_whitespace();
            if let Some(ch) = self.peek() {
                if ch == '}' {
                    self.pos += 1;
                    break;
                } else if ch == ',' {
                    self.pos += 1;
                    continue;
                } else {
                    return Err(format!("Expected ',' or '}}' at position {}", self.pos));
                }
            } else {
                return Err("Unexpected end of input in object".to_string());
            }
        }
        
        Ok(JsonValue::Object(object))
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        
        if let Some(ch) = self.peek() {
            match ch {
                '"' => self.parse_string(),
                '[' => self.parse_array(),
                '{' => self.parse_object(),
                't' => {
                    if self.pos + 3 < self.input.len() 
                        && self.input[self.pos..self.pos + 4] == ['t', 'r', 'u', 'e'] {
                        self.pos += 4;
                        Ok(JsonValue::Bool(true))
                    } else {
                        Err("Expected 'true'".to_string())
                    }
                }
                'f' => {
                    if self.pos + 4 < self.input.len() 
                        && self.input[self.pos..self.pos + 5] == ['f', 'a', 'l', 's', 'e'] {
                        self.pos += 5;
                        Ok(JsonValue::Bool(false))
                    } else {
                        Err("Expected 'false'".to_string())
                    }
                }
                'n' => {
                    if self.pos + 3 < self.input.len() 
                        && self.input[self.pos..self.pos + 4] == ['n', 'u', 'l', 'l'] {
                        self.pos += 4;
                        Ok(JsonValue::Null)
                    } else {
                        Err("Expected 'null'".to_string())
                    }
                }
                '-' | '0'..='9' => self.parse_number(),
                _ => Err(format!("Unexpected character '{}' at position {}", ch, self.pos)),
            }
        } else {
            Err("Unexpected end of input".to_string())
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
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            JsonValue::Object(obj) => {
                write!(f, "{{")?;
                for (i, (key, value)) in obj.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "\"{}\": {}", key.escape_default(), value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

pub fn pretty_print_json(value: &JsonValue, indent: usize) -> String {
    fn pretty_print(value: &JsonValue, indent: usize, current_indent: usize) -> String {
        let indent_str = " ".repeat(current_indent);
        let next_indent = current_indent + indent;
        let next_indent_str = " ".repeat(next_indent);
        
        match value {
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
                    if i > 0 {
                        result.push_str(",\n");
                    }
                    result.push_str(&format!("{}{}", next_indent_str, pretty_print(item, indent, next_indent)));
                }
                result.push_str(&format!("\n{}]", indent_str));
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
                    if i > 0 {
                        result.push_str(",\n");
                    }
                    result.push_str(&format!("{}\"{}\": {}", next_indent_str, key.escape_default(), 
                        pretty_print(value, indent, next_indent)));
                }
                result.push_str(&format!("\n{}}}", indent_str));
                result
            }
        }
    }
    
    pretty_print(value, indent, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_json() {
        let json_str = r#"{"name": "John", "age": 30, "active": true}"#;
        let mut parser = JsonParser::new(json_str);
        let result = parser.parse();
        assert!(result.is_ok());
        
        if let Ok(JsonValue::Object(obj)) = result {
            assert_eq!(obj.get("name"), Some(&JsonValue::String("John".to_string())));
            assert_eq!(obj.get("age"), Some(&JsonValue::Number(30.0)));
            assert_eq!(obj.get("active"), Some(&JsonValue::Bool(true)));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_pretty_print() {
        let json_str = r#"{"name":"John","age":30}"#;
        let mut parser = JsonParser::new(json_str);
        let result = parser.parse().unwrap();
        let pretty = pretty_print_json(&result, 2);
        assert!(pretty.contains("\n"));
        assert!(pretty.contains("  "));
    }
}