use std::collections::HashMap;
use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

#[derive(Debug)]
pub struct JsonParser<'a> {
    chars: Chars<'a>,
    current: Option<char>,
}

impl<'a> JsonParser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut chars = input.chars();
        let current = chars.next();
        JsonParser { chars, current }
    }

    fn advance(&mut self) {
        self.current = self.chars.next();
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.current {
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
        let expected = "null";
        for ch in expected.chars() {
            match self.current {
                Some(c) if c == ch => self.advance(),
                _ => return Err("Invalid null value".to_string()),
            }
        }
        Ok(JsonValue::Null)
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if let Some('t') = self.current {
            let expected = "true";
            for ch in expected.chars() {
                match self.current {
                    Some(c) if c == ch => self.advance(),
                    _ => return Err("Invalid boolean value".to_string()),
                }
            }
            Ok(JsonValue::Bool(true))
        } else {
            let expected = "false";
            for ch in expected.chars() {
                match self.current {
                    Some(c) if c == ch => self.advance(),
                    _ => return Err("Invalid boolean value".to_string()),
                }
            }
            Ok(JsonValue::Bool(false))
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.advance(); // Skip opening quote
        let mut result = String::new();

        while let Some(c) = self.current {
            if c == '"' {
                self.advance();
                return Ok(JsonValue::String(result));
            } else if c == '\\' {
                self.advance();
                if let Some(escaped) = self.current {
                    match escaped {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        '/' => result.push('/'),
                        'b' => result.push('\x08'),
                        'f' => result.push('\x0c'),
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        _ => return Err("Invalid escape sequence".to_string()),
                    }
                    self.advance();
                } else {
                    return Err("Unterminated string".to_string());
                }
            } else {
                result.push(c);
                self.advance();
            }
        }

        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let mut num_str = String::new();
        let mut has_decimal = false;
        let mut has_exponent = false;

        if let Some('-') = self.current {
            num_str.push('-');
            self.advance();
        }

        while let Some(c) = self.current {
            if c.is_digit(10) {
                num_str.push(c);
                self.advance();
            } else if c == '.' && !has_decimal && !has_exponent {
                num_str.push(c);
                has_decimal = true;
                self.advance();
            } else if (c == 'e' || c == 'E') && !has_exponent {
                num_str.push(c);
                has_exponent = true;
                self.advance();

                if let Some(sign) = self.current {
                    if sign == '+' || sign == '-' {
                        num_str.push(sign);
                        self.advance();
                    }
                }
            } else {
                break;
            }
        }

        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number format".to_string()),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.advance(); // Skip '['
        self.skip_whitespace();
        let mut array = Vec::new();

        if let Some(']') = self.current {
            self.advance();
            return Ok(JsonValue::Array(array));
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);
            self.skip_whitespace();

            match self.current {
                Some(',') => {
                    self.advance();
                    self.skip_whitespace();
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

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.advance(); // Skip '{'
        self.skip_whitespace();
        let mut object = HashMap::new();

        if let Some('}') = self.current {
            self.advance();
            return Ok(JsonValue::Object(object));
        }

        loop {
            self.skip_whitespace();
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err("Object key must be a string".to_string()),
            };

            self.skip_whitespace();
            match self.current {
                Some(':') => self.advance(),
                _ => return Err("Expected ':' after object key".to_string()),
            }

            let value = self.parse_value()?;
            object.insert(key, value);
            self.skip_whitespace();

            match self.current {
                Some(',') => {
                    self.advance();
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.advance();
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
        if self.current.is_some() {
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
        let result = parse_json("[1, 2, 3]").unwrap();
        if let JsonValue::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], JsonValue::Number(1.0));
            assert_eq!(arr[1], JsonValue::Number(2.0));
            assert_eq!(arr[2], JsonValue::Number(3.0));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_parse_object() {
        let result = parse_json(r#"{"key": "value", "num": 42}"#).unwrap();
        if let JsonValue::Object(obj) = result {
            assert_eq!(obj.len(), 2);
            assert_eq!(
                obj.get("key"),
                Some(&JsonValue::String("value".to_string()))
            );
            assert_eq!(obj.get("num"), Some(&JsonValue::Number(42.0)));
        } else {
            panic!("Expected object");
        }
    }
}