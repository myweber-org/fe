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

#[derive(Debug, PartialEq)]
enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

struct JsonParser {
    chars: Vec<char>,
    position: usize,
}

impl JsonParser {
    fn new(input: &str) -> Self {
        JsonParser {
            chars: input.chars().collect(),
            position: 0,
        }
    }

    fn parse(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.chars.len() {
            return Err("Unexpected characters after JSON value".to_string());
        }
        Ok(result)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.peek_char() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string_value(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('n') => self.parse_null(),
            _ => Err("Invalid JSON value".to_string()),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.expect_char('{')?;
        self.skip_whitespace();

        let mut map = HashMap::new();

        if self.peek_char() == Some('}') {
            self.advance();
            return Ok(JsonValue::Object(map));
        }

        loop {
            self.skip_whitespace();
            let key = self.parse_json_string()?;
            self.skip_whitespace();
            self.expect_char(':')?;
            let value = self.parse_value()?;
            map.insert(key, value);

            self.skip_whitespace();
            match self.peek_char() {
                Some(',') => {
                    self.advance();
                    continue;
                }
                Some('}') => {
                    self.advance();
                    break;
                }
                _ => return Err("Expected ',' or '}' in object".to_string()),
            }
        }

        Ok(JsonValue::Object(map))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.expect_char('[')?;
        self.skip_whitespace();

        let mut array = Vec::new();

        if self.peek_char() == Some(']') {
            self.advance();
            return Ok(JsonValue::Array(array));
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);

            self.skip_whitespace();
            match self.peek_char() {
                Some(',') => {
                    self.advance();
                    continue;
                }
                Some(']') => {
                    self.advance();
                    break;
                }
                _ => return Err("Expected ',' or ']' in array".to_string()),
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn parse_string_value(&mut self) -> Result<JsonValue, String> {
        let s = self.parse_json_string()?;
        Ok(JsonValue::String(s))
    }

    fn parse_json_string(&mut self) -> Result<String, String> {
        self.expect_char('"')?;
        let mut result = String::new();

        while let Some(c) = self.peek_char() {
            match c {
                '"' => {
                    self.advance();
                    return Ok(result);
                }
                '\\' => {
                    self.advance();
                    let escaped = self.parse_escape_sequence()?;
                    result.push(escaped);
                }
                _ => {
                    result.push(c);
                    self.advance();
                }
            }
        }

        Err("Unterminated string".to_string())
    }

    fn parse_escape_sequence(&mut self) -> Result<char, String> {
        match self.peek_char() {
            Some('"') => {
                self.advance();
                Ok('"')
            }
            Some('\\') => {
                self.advance();
                Ok('\\')
            }
            Some('/') => {
                self.advance();
                Ok('/')
            }
            Some('b') => {
                self.advance();
                Ok('\x08')
            }
            Some('f') => {
                self.advance();
                Ok('\x0c')
            }
            Some('n') => {
                self.advance();
                Ok('\n')
            }
            Some('r') => {
                self.advance();
                Ok('\r')
            }
            Some('t') => {
                self.advance();
                Ok('\t')
            }
            Some('u') => {
                self.advance();
                self.parse_unicode_escape()
            }
            _ => Err("Invalid escape sequence".to_string()),
        }
    }

    fn parse_unicode_escape(&mut self) -> Result<char, String> {
        let hex_digits: String = (0..4)
            .map(|_| {
                self.peek_char()
                    .filter(|c| c.is_digit(16))
                    .map(|c| {
                        self.advance();
                        c
                    })
                    .ok_or("Invalid Unicode escape sequence".to_string())
            })
            .collect::<Result<String, String>>()?;

        let code_point = u32::from_str_radix(&hex_digits, 16)
            .map_err(|_| "Invalid Unicode code point".to_string())?;

        char::from_u32(code_point).ok_or("Invalid Unicode code point".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.position;
        let mut has_decimal = false;
        let mut has_exponent = false;

        if self.peek_char() == Some('-') {
            self.advance();
        }

        while let Some(c) = self.peek_char() {
            match c {
                '0'..='9' => {
                    self.advance();
                }
                '.' => {
                    if has_decimal || has_exponent {
                        return Err("Invalid number format".to_string());
                    }
                    has_decimal = true;
                    self.advance();
                }
                'e' | 'E' => {
                    if has_exponent {
                        return Err("Invalid number format".to_string());
                    }
                    has_exponent = true;
                    self.advance();

                    if self.peek_char() == Some('+') || self.peek_char() == Some('-') {
                        self.advance();
                    }
                }
                _ => break,
            }
        }

        let number_str: String = self.chars[start..self.position].iter().collect();
        number_str
            .parse::<f64>()
            .map(JsonValue::Number)
            .map_err(|_| "Invalid number".to_string())
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.consume_str("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume_str("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err("Invalid boolean value".to_string())
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.consume_str("null") {
            Ok(JsonValue::Null)
        } else {
            Err("Invalid null value".to_string())
        }
    }

    fn consume_str(&mut self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        if self.position + chars.len() <= self.chars.len() {
            for (i, &c) in chars.iter().enumerate() {
                if self.chars[self.position + i] != c {
                    return false;
                }
            }
            self.position += chars.len();
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.position).copied()
    }

    fn advance(&mut self) {
        if self.position < self.chars.len() {
            self.position += 1;
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<(), String> {
        match self.peek_char() {
            Some(c) if c == expected => {
                self.advance();
                Ok(())
            }
            _ => Err(format!("Expected '{}'", expected)),
        }
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
    fn test_parse_bool() {
        assert_eq!(parse_json("true").unwrap(), JsonValue::Bool(true));
        assert_eq!(parse_json("false").unwrap(), JsonValue::Bool(false));
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_json("42").unwrap(), JsonValue::Number(42.0));
        assert_eq!(parse_json("-3.14").unwrap(), JsonValue::Number(-3.14));
        assert_eq!(parse_json("1.23e4").unwrap(), JsonValue::Number(12300.0));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse_json(r#""hello""#).unwrap(),
            JsonValue::String("hello".to_string())
        );
        assert_eq!(
            parse_json(r#""escape\"test""#).unwrap(),
            JsonValue::String("escape\"test".to_string())
        );
    }

    #[test]
    fn test_parse_array() {
        let result = parse_json(r#"[1, true, "test"]"#).unwrap();
        match result {
            JsonValue::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], JsonValue::Number(1.0));
                assert_eq!(arr[1], JsonValue::Bool(true));
                assert_eq!(arr[2], JsonValue::String("test".to_string()));
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_parse_object() {
        let result = parse_json(r#"{"key": "value", "num": 42}"#).unwrap();
        match result {
            JsonValue::Object(map) => {
                assert_eq!(map.len(), 2);
                assert_eq!(
                    map.get("key"),
                    Some(&JsonValue::String("value".to_string()))
                );
                assert_eq!(map.get("num"), Some(&JsonValue::Number(42.0)));
            }
            _ => panic!("Expected object"),
        }
    }
}