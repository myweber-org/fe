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
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
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
        if self.consume_str("null") {
            Ok(JsonValue::Null)
        } else {
            Err("Expected 'null'".to_string())
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.consume_str("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume_str("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err("Expected boolean".to_string())
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.position;
        if self.peek_char() == Some('-') {
            self.consume_char();
        }
        while let Some(c) = self.peek_char() {
            if c.is_digit(10) || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-' {
                self.consume_char();
            } else {
                break;
            }
        }
        let num_str: String = self.input[start..self.position].iter().collect();
        match num_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err("Invalid number".to_string()),
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, String> {
        self.consume_char(); // consume opening quote
        let mut result = String::new();
        while let Some(c) = self.consume_char() {
            match c {
                '"' => return Ok(JsonValue::String(result)),
                '\\' => {
                    if let Some(escaped) = self.consume_char() {
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
                    } else {
                        return Err("Unterminated string".to_string());
                    }
                }
                _ => result.push(c),
            }
        }
        Err("Unterminated string".to_string())
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
                _ => return Err("Expected ',' or ']' in array".to_string()),
            }
        }
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.consume_char(); // consume '{'
        let mut object = HashMap::new();
        self.skip_whitespace();
        if self.peek_char() == Some('}') {
            self.consume_char();
            return Ok(JsonValue::Object(object));
        }
        loop {
            let key = match self.parse_value()? {
                JsonValue::String(s) => s,
                _ => return Err("Object key must be a string".to_string()),
            };
            self.skip_whitespace();
            if self.peek_char() != Some(':') {
                return Err("Expected ':' after object key".to_string());
            }
            self.consume_char();
            self.skip_whitespace();
            let value = self.parse_value()?;
            object.insert(key, value);
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
                _ => return Err("Expected ',' or '}' in object".to_string()),
            }
        }
        Ok(JsonValue::Object(object))
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
        self.input.get(self.position).copied()
    }

    fn consume_char(&mut self) -> Option<char> {
        if self.position < self.input.len() {
            let c = self.input[self.position];
            self.position += 1;
            Some(c)
        } else {
            None
        }
    }

    fn consume_str(&mut self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        if self.position + chars.len() <= self.input.len() {
            for (i, &expected) in chars.iter().enumerate() {
                if self.input[self.position + i] != expected {
                    return false;
                }
            }
            self.position += chars.len();
            true
        } else {
            false
        }
    }
}

fn pretty_print_json(value: &JsonValue, indent: usize) -> String {
    match value {
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::String(s) => format!("\"{}\"", s),
        JsonValue::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                let items: Vec<String> = arr
                    .iter()
                    .map(|v| format!("{}{}", " ".repeat(indent + 2), pretty_print_json(v, indent + 2)))
                    .collect();
                format!("[\n{}\n{}]", items.join(",\n"), " ".repeat(indent))
            }
        }
        JsonValue::Object(obj) => {
            if obj.is_empty() {
                "{}".to_string()
            } else {
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "{}\"{}\": {}",
                            " ".repeat(indent + 2),
                            k,
                            pretty_print_json(v, indent + 2)
                        )
                    })
                    .collect();
                format!("{{\n{}\n{}}}", items.join(",\n"), " ".repeat(indent))
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
        "courses": ["Math", "Science", "History"],
        "address": {
            "street": "123 Main St",
            "city": "Anytown"
        },
        "scores": [95.5, 87.0, 92.3]
    }
    "#;

    let mut parser = JsonParser::new(json_str);
    match parser.parse() {
        Ok(json_value) => {
            println!("Parsed successfully!");
            println!("Pretty printed JSON:");
            println!("{}", pretty_print_json(&json_value, 0));
        }
        Err(e) => println!("Parse error: {}", e),
    }
}