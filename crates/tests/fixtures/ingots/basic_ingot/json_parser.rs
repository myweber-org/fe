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
        match self.peek() {
            Some(ch) if ch == expected => {
                self.pos += 1;
                Ok(())
            }
            Some(ch) => Err(format!("Expected '{}', found '{}'", expected, ch)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.consume('"')?;
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.pos += 1;
                return Ok(result);
            }
            if ch == '\\' {
                self.pos += 1;
                let escaped = self.peek().ok_or("Unexpected end after escape")?;
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
                self.pos += 1;
            } else {
                result.push(ch);
                self.pos += 1;
            }
        }
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<f64, String> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == 'e' || ch == 'E' || ch == '+' {
                self.pos += 1;
            } else {
                break;
            }
        }
        let num_str: String = self.input[start..self.pos].iter().collect();
        num_str
            .parse()
            .map_err(|_| format!("Invalid number: {}", num_str))
    }

    fn parse_array(&mut self) -> Result<Vec<JsonValue>, String> {
        self.consume('[')?;
        self.skip_whitespace();
        if let Some(']') = self.peek() {
            self.pos += 1;
            return Ok(Vec::new());
        }

        let mut array = Vec::new();
        loop {
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
                Some(ch) => return Err(format!("Expected ',' or ']', found '{}'", ch)),
                None => return Err("Unexpected end of array".to_string()),
            }
        }
        Ok(array)
    }

    fn parse_object(&mut self) -> Result<HashMap<String, JsonValue>, String> {
        self.consume('{')?;
        self.skip_whitespace();
        if let Some('}') = self.peek() {
            self.pos += 1;
            return Ok(HashMap::new());
        }

        let mut map = HashMap::new();
        loop {
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.consume(':')?;
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
                Some(ch) => return Err(format!("Expected ',' or '}}', found '{}'", ch)),
                None => return Err("Unexpected end of object".to_string()),
            }
        }
        Ok(map)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.peek() {
            Some('"') => {
                let s = self.parse_string()?;
                Ok(JsonValue::String(s))
            }
            Some('{') => {
                let obj = self.parse_object()?;
                Ok(JsonValue::Object(obj))
            }
            Some('[') => {
                let arr = self.parse_array()?;
                Ok(JsonValue::Array(arr))
            }
            Some('t') => {
                if self.input[self.pos..].starts_with(&['t', 'r', 'u', 'e']) {
                    self.pos += 4;
                    Ok(JsonValue::Bool(true))
                } else {
                    Err("Expected 'true'".to_string())
                }
            }
            Some('f') => {
                if self.input[self.pos..].starts_with(&['f', 'a', 'l', 's', 'e']) {
                    self.pos += 5;
                    Ok(JsonValue::Bool(false))
                } else {
                    Err("Expected 'false'".to_string())
                }
            }
            Some('n') => {
                if self.input[self.pos..].starts_with(&['n', 'u', 'l', 'l']) {
                    self.pos += 4;
                    Ok(JsonValue::Null)
                } else {
                    Err("Expected 'null'".to_string())
                }
            }
            Some(ch) if ch.is_ascii_digit() || ch == '-' => {
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
        if self.pos < self.input.len() {
            return Err("Trailing characters after JSON value".to_string());
        }
        Ok(result)
    }
}

pub fn parse_json(input: &str) -> Result<JsonValue, String> {
    let mut parser = JsonParser::new(input);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_object() {
        let json = r#"{"name": "test", "value": 42}"#;
        let result = parse_json(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_nested_structure() {
        let json = r#"{"data": [1, 2, {"nested": true}]}"#;
        let result = parse_json(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_json() {
        let json = r#"{"unclosed": "#;
        let result = parse_json(json);
        assert!(result.is_err());
    }
}