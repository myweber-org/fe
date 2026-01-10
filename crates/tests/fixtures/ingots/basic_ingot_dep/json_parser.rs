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

#[derive(Debug)]
pub struct ParseError {
    message: String,
    position: usize,
}

pub fn parse_json(input: &str) -> Result<JsonValue, ParseError> {
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;
    parse_value(&chars, &mut index)
}

fn parse_value(chars: &[char], index: &mut usize) -> Result<JsonValue, ParseError> {
    skip_whitespace(chars, index);
    
    if *index >= chars.len() {
        return Err(ParseError {
            message: "Unexpected end of input".to_string(),
            position: *index,
        });
    }
    
    match chars[*index] {
        'n' => parse_literal(chars, index, "null", JsonValue::Null),
        't' => parse_literal(chars, index, "true", JsonValue::Bool(true)),
        'f' => parse_literal(chars, index, "false", JsonValue::Bool(false)),
        '"' => parse_string(chars, index),
        '[' => parse_array(chars, index),
        '{' => parse_object(chars, index),
        '-' | '0'..='9' => parse_number(chars, index),
        _ => Err(ParseError {
            message: format!("Unexpected character: {}", chars[*index]),
            position: *index,
        }),
    }
}

fn parse_literal(
    chars: &[char],
    index: &mut usize,
    literal: &str,
    value: JsonValue,
) -> Result<JsonValue, ParseError> {
    for (i, ch) in literal.chars().enumerate() {
        if *index + i >= chars.len() || chars[*index + i] != ch {
            return Err(ParseError {
                message: format!("Expected '{}'", literal),
                position: *index,
            });
        }
    }
    *index += literal.len();
    Ok(value)
}

fn parse_string(chars: &[char], index: &mut usize) -> Result<JsonValue, ParseError> {
    *index += 1;
    let mut result = String::new();
    
    while *index < chars.len() && chars[*index] != '"' {
        if chars[*index] == '\\' {
            *index += 1;
            if *index >= chars.len() {
                return Err(ParseError {
                    message: "Unterminated escape sequence".to_string(),
                    position: *index - 1,
                });
            }
            match chars[*index] {
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                '/' => result.push('/'),
                'b' => result.push('\u{0008}'),
                'f' => result.push('\u{000C}'),
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                _ => return Err(ParseError {
                    message: format!("Invalid escape character: {}", chars[*index]),
                    position: *index,
                }),
            }
        } else {
            result.push(chars[*index]);
        }
        *index += 1;
    }
    
    if *index >= chars.len() {
        return Err(ParseError {
            message: "Unterminated string".to_string(),
            position: *index,
        });
    }
    
    *index += 1;
    Ok(JsonValue::String(result))
}

fn parse_number(chars: &[char], index: &mut usize) -> Result<JsonValue, ParseError> {
    let start = *index;
    let mut has_dot = false;
    let mut has_exp = false;
    
    if chars[*index] == '-' {
        *index += 1;
    }
    
    while *index < chars.len() {
        match chars[*index] {
            '0'..='9' => *index += 1,
            '.' => {
                if has_dot || has_exp {
                    break;
                }
                has_dot = true;
                *index += 1;
            }
            'e' | 'E' => {
                if has_exp {
                    break;
                }
                has_exp = true;
                *index += 1;
                if *index < chars.len() && (chars[*index] == '+' || chars[*index] == '-') {
                    *index += 1;
                }
            }
            _ => break,
        }
    }
    
    let num_str: String = chars[start..*index].iter().collect();
    match num_str.parse::<f64>() {
        Ok(num) => Ok(JsonValue::Number(num)),
        Err(_) => Err(ParseError {
            message: format!("Invalid number: {}", num_str),
            position: start,
        }),
    }
}

fn parse_array(chars: &[char], index: &mut usize) -> Result<JsonValue, ParseError> {
    *index += 1;
    skip_whitespace(chars, index);
    
    let mut array = Vec::new();
    
    if *index < chars.len() && chars[*index] == ']' {
        *index += 1;
        return Ok(JsonValue::Array(array));
    }
    
    loop {
        let value = parse_value(chars, index)?;
        array.push(value);
        
        skip_whitespace(chars, index);
        
        if *index >= chars.len() {
            return Err(ParseError {
                message: "Unterminated array".to_string(),
                position: *index,
            });
        }
        
        match chars[*index] {
            ',' => {
                *index += 1;
                skip_whitespace(chars, index);
                continue;
            }
            ']' => {
                *index += 1;
                break;
            }
            _ => {
                return Err(ParseError {
                    message: format!("Expected ',' or ']', found: {}", chars[*index]),
                    position: *index,
                });
            }
        }
    }
    
    Ok(JsonValue::Array(array))
}

fn parse_object(chars: &[char], index: &mut usize) -> Result<JsonValue, ParseError> {
    *index += 1;
    skip_whitespace(chars, index);
    
    let mut object = HashMap::new();
    
    if *index < chars.len() && chars[*index] == '}' {
        *index += 1;
        return Ok(JsonValue::Object(object));
    }
    
    loop {
        skip_whitespace(chars, index);
        
        if *index >= chars.len() || chars[*index] != '"' {
            return Err(ParseError {
                message: "Expected string key".to_string(),
                position: *index,
            });
        }
        
        let key = match parse_string(chars, index)? {
            JsonValue::String(s) => s,
            _ => unreachable!(),
        };
        
        skip_whitespace(chars, index);
        
        if *index >= chars.len() || chars[*index] != ':' {
            return Err(ParseError {
                message: "Expected ':' after key".to_string(),
                position: *index,
            });
        }
        
        *index += 1;
        skip_whitespace(chars, index);
        
        let value = parse_value(chars, index)?;
        object.insert(key, value);
        
        skip_whitespace(chars, index);
        
        if *index >= chars.len() {
            return Err(ParseError {
                message: "Unterminated object".to_string(),
                position: *index,
            });
        }
        
        match chars[*index] {
            ',' => {
                *index += 1;
                skip_whitespace(chars, index);
                continue;
            }
            '}' => {
                *index += 1;
                break;
            }
            _ => {
                return Err(ParseError {
                    message: format!("Expected ',' or '}}', found: {}", chars[*index]),
                    position: *index,
                });
            }
        }
    }
    
    Ok(JsonValue::Object(object))
}

fn skip_whitespace(chars: &[char], index: &mut usize) {
    while *index < chars.len() && chars[*index].is_whitespace() {
        *index += 1;
    }
}use std::collections::HashMap;

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
pub enum ParseError {
    UnexpectedCharacter(char, usize),
    UnexpectedEndOfInput,
    InvalidNumber,
    InvalidEscapeSequence,
    KeyMustBeString,
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

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn consume(&mut self, expected: char) -> Result<(), ParseError> {
        self.skip_whitespace();
        match self.peek() {
            Some(ch) if ch == expected => {
                self.position += 1;
                Ok(())
            }
            Some(ch) => Err(ParseError::UnexpectedCharacter(ch, self.position)),
            None => Err(ParseError::UnexpectedEndOfInput),
        }
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        self.consume('"')?;
        let mut result = String::new();
        let mut escape = false;

        while let Some(ch) = self.peek() {
            self.position += 1;

            if escape {
                match ch {
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    '/' => result.push('/'),
                    'b' => result.push('\x08'),
                    'f' => result.push('\x0c'),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    _ => return Err(ParseError::InvalidEscapeSequence),
                }
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                return Ok(result);
            } else {
                result.push(ch);
            }
        }

        Err(ParseError::UnexpectedEndOfInput)
    }

    fn parse_number(&mut self) -> Result<f64, ParseError> {
        let start = self.position;
        let mut has_dot = false;
        let mut has_exp = false;

        while let Some(ch) = self.peek() {
            match ch {
                '0'..='9' => {
                    self.position += 1;
                }
                '.' => {
                    if has_dot || has_exp {
                        return Err(ParseError::InvalidNumber);
                    }
                    has_dot = true;
                    self.position += 1;
                }
                'e' | 'E' => {
                    if has_exp {
                        return Err(ParseError::InvalidNumber);
                    }
                    has_exp = true;
                    self.position += 1;
                    if let Some(next) = self.peek() {
                        if next == '+' || next == '-' {
                            self.position += 1;
                        }
                    }
                }
                '+' | '-' if self.position == start => {
                    self.position += 1;
                }
                _ => break,
            }
        }

        let num_str: String = self.input[start..self.position].iter().collect();
        num_str
            .parse()
            .map_err(|_| ParseError::InvalidNumber)
    }

    fn parse_array(&mut self) -> Result<Vec<JsonValue>, ParseError> {
        self.consume('[')?;
        self.skip_whitespace();

        let mut array = Vec::new();

        if let Some(']') = self.peek() {
            self.position += 1;
            return Ok(array);
        }

        loop {
            let value = self.parse_value()?;
            array.push(value);

            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.position += 1;
                    self.skip_whitespace();
                }
                Some(']') => {
                    self.position += 1;
                    break;
                }
                Some(ch) => return Err(ParseError::UnexpectedCharacter(ch, self.position)),
                None => return Err(ParseError::UnexpectedEndOfInput),
            }
        }

        Ok(array)
    }

    fn parse_object(&mut self) -> Result<HashMap<String, JsonValue>, ParseError> {
        self.consume('{')?;
        self.skip_whitespace();

        let mut object = HashMap::new();

        if let Some('}') = self.peek() {
            self.position += 1;
            return Ok(object);
        }

        loop {
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.consume(':')?;
            let value = self.parse_value()?;
            object.insert(key, value);

            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.position += 1;
                    self.skip_whitespace();
                }
                Some('}') => {
                    self.position += 1;
                    break;
                }
                Some(ch) => return Err(ParseError::UnexpectedCharacter(ch, self.position)),
                None => return Err(ParseError::UnexpectedEndOfInput),
            }
        }

        Ok(object)
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
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
                if self.position + 3 < self.input.len()
                    && self.input[self.position..self.position + 4] == ['t', 'r', 'u', 'e']
                {
                    self.position += 4;
                    Ok(JsonValue::Bool(true))
                } else {
                    Err(ParseError::UnexpectedCharacter(self.input[self.position], self.position))
                }
            }
            Some('f') => {
                if self.position + 4 < self.input.len()
                    && self.input[self.position..self.position + 5] == ['f', 'a', 'l', 's', 'e']
                {
                    self.position += 5;
                    Ok(JsonValue::Bool(false))
                } else {
                    Err(ParseError::UnexpectedCharacter(self.input[self.position], self.position))
                }
            }
            Some('n') => {
                if self.position + 3 < self.input.len()
                    && self.input[self.position..self.position + 4] == ['n', 'u', 'l', 'l']
                {
                    self.position += 4;
                    Ok(JsonValue::Null)
                } else {
                    Err(ParseError::UnexpectedCharacter(self.input[self.position], self.position))
                }
            }
            Some(ch) if ch.is_ascii_digit() || ch == '-' => {
                let num = self.parse_number()?;
                Ok(JsonValue::Number(num))
            }
            Some(ch) => Err(ParseError::UnexpectedCharacter(ch, self.position)),
            None => Err(ParseError::UnexpectedEndOfInput),
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, ParseError> {
        let result = self.parse_value()?;
        self.skip_whitespace();
        if self.position < self.input.len() {
            Err(ParseError::UnexpectedCharacter(self.input[self.position], self.position))
        } else {
            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_json() {
        let mut parser = JsonParser::new(r#"{"name": "test", "value": 42.5}"#);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_array() {
        let mut parser = JsonParser::new(r#"[1, 2, 3, true, false, null]"#);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_nested() {
        let mut parser = JsonParser::new(r#"{"data": {"items": [1, 2, 3], "active": true}}"#);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}