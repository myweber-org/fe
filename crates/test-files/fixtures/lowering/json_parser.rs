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

#[derive(Debug)]
struct ParseError {
    message: String,
    position: usize,
}

fn parse_json(input: &str) -> Result<JsonValue, ParseError> {
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
        'n' => parse_null(chars, index),
        't' | 'f' => parse_bool(chars, index),
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

fn parse_null(chars: &[char], index: &mut usize) -> Result<JsonValue, ParseError> {
    if *index + 3 < chars.len() && chars[*index..*index + 4].iter().collect::<String>() == "null" {
        *index += 4;
        Ok(JsonValue::Null)
    } else {
        Err(ParseError {
            message: "Expected 'null'".to_string(),
            position: *index,
        })
    }
}

fn parse_bool(chars: &[char], index: &mut usize) -> Result<JsonValue, ParseError> {
    if *index + 3 < chars.len() && chars[*index..*index + 4].iter().collect::<String>() == "true" {
        *index += 4;
        Ok(JsonValue::Bool(true))
    } else if *index + 4 < chars.len() && chars[*index..*index + 5].iter().collect::<String>() == "false" {
        *index += 5;
        Ok(JsonValue::Bool(false))
    } else {
        Err(ParseError {
            message: "Expected boolean value".to_string(),
            position: *index,
        })
    }
}

fn parse_string(chars: &[char], index: &mut usize) -> Result<JsonValue, ParseError> {
    *index += 1;
    let start = *index;
    let mut result = String::new();

    while *index < chars.len() && chars[*index] != '"' {
        if chars[*index] == '\\' {
            *index += 1;
            if *index >= chars.len() {
                return Err(ParseError {
                    message: "Unterminated escape sequence".to_string(),
                    position: *index,
                });
            }
            match chars[*index] {
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                '/' => result.push('/'),
                'b' => result.push('\x08'),
                'f' => result.push('\x0c'),
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

    if *index >= chars.len() || chars[*index] != '"' {
        return Err(ParseError {
            message: "Unterminated string".to_string(),
            position: start,
        });
    }

    *index += 1;
    Ok(JsonValue::String(result))
}

fn parse_number(chars: &[char], index: &mut usize) -> Result<JsonValue, ParseError> {
    let start = *index;
    let mut has_dot = false;
    let mut has_exponent = false;

    if chars[*index] == '-' {
        *index += 1;
    }

    while *index < chars.len() {
        match chars[*index] {
            '0'..='9' => *index += 1,
            '.' => {
                if has_dot || has_exponent {
                    return Err(ParseError {
                        message: "Invalid number format".to_string(),
                        position: *index,
                    });
                }
                has_dot = true;
                *index += 1;
            }
            'e' | 'E' => {
                if has_exponent {
                    return Err(ParseError {
                        message: "Invalid number format".to_string(),
                        position: *index,
                    });
                }
                has_exponent = true;
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
            message: "Invalid number format".to_string(),
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

        if chars[*index] == ']' {
            *index += 1;
            break;
        } else if chars[*index] == ',' {
            *index += 1;
            skip_whitespace(chars, index);
        } else {
            return Err(ParseError {
                message: format!("Expected ',' or ']', found: {}", chars[*index]),
                position: *index,
            });
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
                message: "Expected ':'".to_string(),
                position: *index,
            });
        }
        *index += 1;

        let value = parse_value(chars, index)?;
        object.insert(key, value);
        skip_whitespace(chars, index);

        if *index >= chars.len() {
            return Err(ParseError {
                message: "Unterminated object".to_string(),
                position: *index,
            });
        }

        if chars[*index] == '}' {
            *index += 1;
            break;
        } else if chars[*index] == ',' {
            *index += 1;
            skip_whitespace(chars, index);
        } else {
            return Err(ParseError {
                message: format!("Expected ',' or '}}', found: {}", chars[*index]),
                position: *index,
            });
        }
    }

    Ok(JsonValue::Object(object))
}

fn skip_whitespace(chars: &[char], index: &mut usize) {
    while *index < chars.len() && chars[*index].is_whitespace() {
        *index += 1;
    }
}