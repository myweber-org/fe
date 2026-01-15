use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
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
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.pos < self.input.len() {
            return Err("Unexpected trailing characters".to_string());
        }
        Ok(result)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
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
            Err("Expected 'true' or 'false'".to_string())
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        if self.peek_char() == Some('-') {
            self.advance();
        }
        while let Some(c) = self.peek_char() {
            if c.is_digit(10) || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-' {
                self.advance();
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

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.expect('"')?;
        let mut result = String::new();
        while let Some(c) = self.next_char() {
            match c {
                '"' => break,
                '\\' => {
                    let escaped = self.next_char().ok_or("Unterminated escape sequence")?;
                    result.push(match escaped {
                        '"' => '"',
                        '\\' => '\\',
                        '/' => '/',
                        'b' => '\u{0008}',
                        'f' => '\u{000c}',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        'u' => {
                            let hex: String = (0..4)
                                .filter_map(|_| self.next_char())
                                .collect();
                            u32::from_str_radix(&hex, 16)
                                .ok()
                                .and_then(|cp| char::from_u32(cp))
                                .ok_or("Invalid Unicode escape")?
                        }
                        _ => return Err("Invalid escape sequence".to_string()),
                    });
                }
                _ => result.push(c),
            }
        }
        Ok(JsonValue::String(result))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.expect('[')?;
        self.skip_whitespace();
        let mut items = Vec::new();
        if self.peek_char() != Some(']') {
            loop {
                items.push(self.parse_value()?);
                self.skip_whitespace();
                if self.peek_char() == Some(']') {
                    break;
                }
                self.expect(',')?;
                self.skip_whitespace();
            }
        }
        self.expect(']')?;
        Ok(JsonValue::Array(items))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.expect('{')?;
        self.skip_whitespace();
        let mut map = HashMap::new();
        if self.peek_char() != Some('}') {
            loop {
                let key = match self.parse_value()? {
                    JsonValue::String(s) => s,
                    _ => return Err("Object key must be string".to_string()),
                };
                self.skip_whitespace();
                self.expect(':')?;
                self.skip_whitespace();
                let value = self.parse_value()?;
                map.insert(key, value);
                self.skip_whitespace();
                if self.peek_char() == Some('}') {
                    break;
                }
                self.expect(',')?;
                self.skip_whitespace();
            }
        }
        self.expect('}')?;
        Ok(JsonValue::Object(map))
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
        self.input.chars().nth(self.pos)
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.peek_char();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn advance(&mut self) {
        if self.pos < self.input.len() {
            self.pos += 1;
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), String> {
        match self.next_char() {
            Some(c) if c == expected => Ok(()),
            Some(c) => Err(format!("Expected '{}', found '{}'", expected, c)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn consume(&mut self, target: &str) -> bool {
        if self.input[self.pos..].starts_with(target) {
            self.pos += target.len();
            true
        } else {
            false
        }
    }
}

impl JsonValue {
    fn to_pretty_string(&self, indent: usize) -> String {
        match self {
            JsonValue::Null => "null".to_string(),
            JsonValue::Bool(b) => b.to_string(),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::String(s) => format!("\"{}\"", s.escape_default()),
            JsonValue::Array(arr) => {
                if arr.is_empty() {
                    "[]".to_string()
                } else {
                    let items: Vec<String> = arr.iter()
                        .map(|item| format!("{}{}", " ".repeat(indent + 2), item.to_pretty_string(indent + 2)))
                        .collect();
                    format!("[\n{}\n{}]", items.join(",\n"), " ".repeat(indent))
                }
            }
            JsonValue::Object(obj) => {
                if obj.is_empty() {
                    "{}".to_string()
                } else {
                    let items: Vec<String> = obj.iter()
                        .map(|(k, v)| format!("{}{}: {}", " ".repeat(indent + 2), JsonValue::String(k.clone()).to_pretty_string(0), v.to_pretty_string(indent + 2)))
                        .collect();
                    format!("{{\n{}\n{}}}", items.join(",\n"), " ".repeat(indent))
                }
            }
        }
    }
}

fn main() {
    let json_str = r#"
    {
        "name": "John Doe",
        "age": 30,
        "is_student": false,
        "courses": ["Math", "Science"],
        "address": {
            "street": "123 Main St",
            "city": "Anytown"
        }
    }
    "#;

    let mut parser = JsonParser::new(json_str);
    match parser.parse() {
        Ok(json) => {
            println!("Parsed successfully!");
            println!("Pretty printed:\n{}", json.to_pretty_string(0));
        }
        Err(e) => println!("Parse error: {}", e),
    }
}