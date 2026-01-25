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

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn consume(&mut self) -> Option<char> {
        let ch = self.peek();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.consume();
            } else {
                break;
            }
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        if self.consume() != Some('"') {
            return Err("Expected '\"' at start of string".to_string());
        }

        let mut result = String::new();
        while let Some(ch) = self.consume() {
            match ch {
                '"' => return Ok(JsonValue::String(result)),
                '\\' => {
                    if let Some(escaped) = self.consume() {
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
                    } else {
                        return Err("Unexpected end of input in string escape".to_string());
                    }
                }
                _ => result.push(ch),
            }
        }
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos - 1;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' || ch == 'e' || ch == 'E' || ch == '+' || ch == '-' {
                self.consume();
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

    fn parse_keyword(&mut self, keyword: &str, value: JsonValue) -> Result<JsonValue, String> {
        for expected in keyword.chars() {
            match self.consume() {
                Some(ch) if ch == expected => continue,
                _ => return Err(format!("Expected keyword '{}'", keyword)),
            }
        }
        Ok(value)
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        if self.consume() != Some('[') {
            return Err("Expected '[' at start of array".to_string());
        }

        self.skip_whitespace();
        let mut items = Vec::new();

        if self.peek() == Some(']') {
            self.consume();
            return Ok(JsonValue::Array(items));
        }

        loop {
            self.skip_whitespace();
            let item = self.parse_value()?;
            items.push(item);
            self.skip_whitespace();

            match self.peek() {
                Some(',') => {
                    self.consume();
                    continue;
                }
                Some(']') => {
                    self.consume();
                    break;
                }
                _ => return Err("Expected ',' or ']' in array".to_string()),
            }
        }

        Ok(JsonValue::Array(items))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        if self.consume() != Some('{') {
            return Err("Expected '{{' at start of object".to_string());
        }

        self.skip_whitespace();
        let mut map = HashMap::new();

        if self.peek() == Some('}') {
            self.consume();
            return Ok(JsonValue::Object(map));
        }

        loop {
            self.skip_whitespace();
            let key = match self.parse_value()? {
                JsonValue::String(s) => s,
                _ => return Err("Object keys must be strings".to_string()),
            };

            self.skip_whitespace();
            if self.consume() != Some(':') {
                return Err("Expected ':' after object key".to_string());
            }

            self.skip_whitespace();
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();

            match self.peek() {
                Some(',') => {
                    self.consume();
                    continue;
                }
                Some('}') => {
                    self.consume();
                    break;
                }
                _ => return Err("Expected ',' or '}}' in object".to_string()),
            }
        }

        Ok(JsonValue::Object(map))
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.peek() {
            Some('"') => self.parse_string(),
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('t') => self.parse_keyword("true", JsonValue::Bool(true)),
            Some('f') => self.parse_keyword("false", JsonValue::Bool(false)),
            Some('n') => self.parse_keyword("null", JsonValue::Null),
            Some(ch) if ch.is_ascii_digit() || ch == '-' => {
                self.consume();
                self.parse_number()
            }
            _ => Err("Unexpected character".to_string()),
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, String> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.peek().is_some() {
            return Err("Trailing characters after JSON value".to_string());
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        let mut parser = JsonParser::new("\"hello world\"");
        assert_eq!(parser.parse(), Ok(JsonValue::String("hello world".to_string())));
    }

    #[test]
    fn test_parse_number() {
        let mut parser = JsonParser::new("42.5");
        assert_eq!(parser.parse(), Ok(JsonValue::Number(42.5)));
    }

    #[test]
    fn test_parse_array() {
        let mut parser = JsonParser::new("[1, 2, 3]");
        let expected = JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ]);
        assert_eq!(parser.parse(), Ok(expected));
    }

    #[test]
    fn test_parse_object() {
        let mut parser = JsonParser::new(r#"{"key": "value"}"#);
        let mut map = HashMap::new();
        map.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(parser.parse(), Ok(JsonValue::Object(map)));
    }
}