use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
}

#[derive(Debug)]
pub struct ParseError {
    message: String,
    position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at position {}: {}", self.position, self.message)
    }
}

impl Error for ParseError {}

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

    fn parse_string(&mut self) -> Result<String, ParseError> {
        if self.input[self.position] != '"' {
            return Err(ParseError {
                message: "Expected string starting with '\"'".to_string(),
                position: self.position,
            });
        }
        self.position += 1;
        let start = self.position;
        while self.position < self.input.len() && self.input[self.position] != '"' {
            if self.input[self.position] == '\\' {
                self.position += 1;
            }
            self.position += 1;
        }
        if self.position >= self.input.len() {
            return Err(ParseError {
                message: "Unterminated string".to_string(),
                position: start,
            });
        }
        let result: String = self.input[start..self.position].iter().collect();
        self.position += 1;
        Ok(result)
    }

    fn parse_number(&mut self) -> Result<f64, ParseError> {
        let start = self.position;
        while self.position < self.input.len() && (self.input[self.position].is_digit(10) || self.input[self.position] == '.' || self.input[self.position] == '-' || self.input[self.position] == 'e' || self.input[self.position] == 'E') {
            self.position += 1;
        }
        let num_str: String = self.input[start..self.position].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(num),
            Err(_) => Err(ParseError {
                message: format!("Invalid number format: {}", num_str),
                position: start,
            }),
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        self.skip_whitespace();
        if self.position >= self.input.len() {
            return Err(ParseError {
                message: "Unexpected end of input".to_string(),
                position: self.position,
            });
        }
        let ch = self.input[self.position];
        match ch {
            '"' => {
                let s = self.parse_string()?;
                Ok(JsonValue::String(s))
            }
            '{' => self.parse_object(),
            '[' => self.parse_array(),
            't' | 'f' => self.parse_boolean(),
            'n' => self.parse_null(),
            '-' | '0'..='9' => {
                let n = self.parse_number()?;
                Ok(JsonValue::Number(n))
            }
            _ => Err(ParseError {
                message: format!("Unexpected character: {}", ch),
                position: self.position,
            }),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, ParseError> {
        if self.input[self.position] != '{' {
            return Err(ParseError {
                message: "Expected object starting with '{'".to_string(),
                position: self.position,
            });
        }
        self.position += 1;
        self.skip_whitespace();
        let mut map = HashMap::new();
        if self.input[self.position] == '}' {
            self.position += 1;
            return Ok(JsonValue::Object(map));
        }
        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            if self.position >= self.input.len() || self.input[self.position] != ':' {
                return Err(ParseError {
                    message: "Expected ':' after key".to_string(),
                    position: self.position,
                });
            }
            self.position += 1;
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            if self.position >= self.input.len() {
                return Err(ParseError {
                    message: "Unexpected end of object".to_string(),
                    position: self.position,
                });
            }
            if self.input[self.position] == '}' {
                self.position += 1;
                break;
            }
            if self.input[self.position] != ',' {
                return Err(ParseError {
                    message: "Expected ',' or '}' in object".to_string(),
                    position: self.position,
                });
            }
            self.position += 1;
        }
        Ok(JsonValue::Object(map))
    }

    fn parse_array(&mut self) -> Result<JsonValue, ParseError> {
        if self.input[self.position] != '[' {
            return Err(ParseError {
                message: "Expected array starting with '['".to_string(),
                position: self.position,
            });
        }
        self.position += 1;
        self.skip_whitespace();
        let mut arr = Vec::new();
        if self.input[self.position] == ']' {
            self.position += 1;
            return Ok(JsonValue::Array(arr));
        }
        loop {
            let value = self.parse_value()?;
            arr.push(value);
            self.skip_whitespace();
            if self.position >= self.input.len() {
                return Err(ParseError {
                    message: "Unexpected end of array".to_string(),
                    position: self.position,
                });
            }
            if self.input[self.position] == ']' {
                self.position += 1;
                break;
            }
            if self.input[self.position] != ',' {
                return Err(ParseError {
                    message: "Expected ',' or ']' in array".to_string(),
                    position: self.position,
                });
            }
            self.position += 1;
            self.skip_whitespace();
        }
        Ok(JsonValue::Array(arr))
    }

    fn parse_boolean(&mut self) -> Result<JsonValue, ParseError> {
        if self.position + 3 < self.input.len() && &self.input[self.position..self.position + 4] == &['t', 'r', 'u', 'e'] {
            self.position += 4;
            return Ok(JsonValue::Boolean(true));
        }
        if self.position + 4 < self.input.len() && &self.input[self.position..self.position + 5] == &['f', 'a', 'l', 's', 'e'] {
            self.position += 5;
            return Ok(JsonValue::Boolean(false));
        }
        Err(ParseError {
            message: "Invalid boolean value".to_string(),
            position: self.position,
        })
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        if self.position + 3 < self.input.len() && &self.input[self.position..self.position + 4] == &['n', 'u', 'l', 'l'] {
            self.position += 4;
            return Ok(JsonValue::Null);
        }
        Err(ParseError {
            message: "Invalid null value".to_string(),
            position: self.position,
        })
    }

    pub fn parse(&mut self) -> Result<JsonValue, ParseError> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
            return Err(ParseError {
                message: "Unexpected trailing characters".to_string(),
                position: self.position,
            });
        }
        Ok(result)
    }
}

pub fn extract_keys(json_str: &str) -> Result<Vec<String>, ParseError> {
    let mut parser = JsonParser::new(json_str);
    let value = parser.parse()?;
    let mut keys = Vec::new();
    extract_keys_from_value(&value, &mut keys);
    Ok(keys)
}

fn extract_keys_from_value(value: &JsonValue, keys: &mut Vec<String>) {
    match value {
        JsonValue::Object(map) => {
            for (key, val) in map {
                keys.push(key.clone());
                extract_keys_from_value(val, keys);
            }
        }
        JsonValue::Array(arr) => {
            for val in arr {
                extract_keys_from_value(val, keys);
            }
        }
        _ => {}
    }
}