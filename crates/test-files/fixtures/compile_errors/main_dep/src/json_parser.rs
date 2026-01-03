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
}