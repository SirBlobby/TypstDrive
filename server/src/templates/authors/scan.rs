pub struct Field {
    pub value_start: usize,
    pub value_end: usize,
}

const FIELD_NAMES: &[&str] = &["authors", "author"];

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

fn string_end(bytes: &[u8], start: usize) -> usize {
    let mut index = start + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b'"' => return index + 1,
            _ => index += 1,
        }
    }
    bytes.len()
}

fn comment_end(bytes: &[u8], start: usize) -> Option<usize> {
    match bytes.get(start..start + 2)? {
        b"//" => Some(bytes[start..].iter().position(|byte| *byte == b'\n').map_or(bytes.len(), |offset| start + offset)),
        b"/*" => Some(bytes[start + 2..].windows(2).position(|pair| pair == b"*/").map_or(bytes.len(), |offset| start + 2 + offset + 2)),
        _ => None,
    }
}

fn closing_for(open: u8) -> u8 {
    match open {
        b'(' => b')',
        b'[' => b']',
        _ => b'}',
    }
}

pub fn matching_close(text: &str, open_index: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut stack = vec![closing_for(bytes[open_index])];
    let mut index = open_index + 1;
    while index < bytes.len() {
        let in_code = stack.last() != Some(&b']');
        let byte = bytes[index];
        if let Some(end) = comment_end(bytes, index) {
            index = end;
            continue;
        }
        match byte {
            b'"' if in_code => {
                index = string_end(bytes, index);
                continue;
            }
            b'\\' if !in_code => index += 1,
            b'(' | b'[' | b'{' => stack.push(closing_for(byte)),
            b')' | b']' | b'}' => {
                if stack.pop() != Some(byte) {
                    return None;
                }
                if stack.is_empty() {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn skip_spaces(bytes: &[u8], mut index: usize) -> usize {
    while index < bytes.len() && (bytes[index] == b' ' || bytes[index] == b'\t' || bytes[index] == b'\n' || bytes[index] == b'\r') {
        index += 1;
    }
    index
}

pub fn value_end(text: &str, start: usize) -> Option<usize> {
    match text.as_bytes().get(start)? {
        b'(' | b'[' => matching_close(text, start).map(|close| close + 1),
        b'"' => Some(string_end(text.as_bytes(), start)),
        _ => None,
    }
}

fn field_value_start(text: &str, name_end: usize, inside_parentheses: bool, after_let: bool) -> Option<usize> {
    let bytes = text.as_bytes();
    let separator = skip_spaces(bytes, name_end);
    let is_named_argument = inside_parentheses && bytes.get(separator) == Some(&b':');
    let is_binding = after_let && bytes.get(separator) == Some(&b'=') && bytes.get(separator + 1) != Some(&b'=');
    (is_named_argument || is_binding).then(|| skip_spaces(bytes, separator + 1))
}

fn follows_let(text: &str, name_start: usize) -> bool {
    text[..name_start].trim_end_matches([' ', '\t']).ends_with("let")
}

pub fn find_field(text: &str) -> Option<Field> {
    let bytes = text.as_bytes();
    let mut stack: Vec<u8> = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let in_code = matches!(stack.last(), Some(b')') | Some(b'}'));
        if let Some(end) = comment_end(bytes, index) {
            index = end;
            continue;
        }
        let byte = bytes[index];
        match byte {
            b'"' if in_code => {
                index = string_end(bytes, index);
                continue;
            }
            b'(' | b'[' | b'{' => stack.push(closing_for(byte)),
            b')' | b']' | b'}' => {
                if stack.last() == Some(&byte) {
                    stack.pop();
                }
            }
            _ if index == 0 || !is_identifier_byte(bytes[index - 1]) => {
                for name in FIELD_NAMES {
                    let name_end = index + name.len();
                    let matches_name = bytes[index..].starts_with(name.as_bytes()) && bytes.get(name_end).map_or(true, |next| !is_identifier_byte(*next));
                    if !matches_name {
                        continue;
                    }
                    let inside_parentheses = stack.last() == Some(&b')');
                    let Some(value_start) = field_value_start(text, name_end, inside_parentheses, follows_let(text, index)) else {
                        continue;
                    };
                    if let Some(value_end) = value_end(text, value_start) {
                        return Some(Field { value_start, value_end });
                    }
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

pub fn split_top_level(inner: &str) -> Vec<&str> {
    let bytes = inner.as_bytes();
    let mut parts = Vec::new();
    let mut part_start = 0;
    let mut index = 0;
    while index < bytes.len() {
        if let Some(end) = comment_end(bytes, index) {
            index = end;
            continue;
        }
        match bytes[index] {
            b'(' | b'[' | b'{' => {
                index = matching_close(inner, index).map_or(bytes.len(), |close| close + 1);
                continue;
            }
            b'"' => {
                index = string_end(bytes, index);
                continue;
            }
            b',' => {
                parts.push(&inner[part_start..index]);
                part_start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }
    parts.push(&inner[part_start..]);
    parts.into_iter().map(without_leading_comments).filter(|part| !part.is_empty()).collect()
}

pub fn without_leading_comments(part: &str) -> &str {
    let mut rest = part.trim();
    while let Some(end) = comment_end(rest.as_bytes(), 0) {
        rest = rest[end..].trim();
    }
    rest
}

pub fn without_comments(code: &str) -> String {
    let bytes = code.as_bytes();
    let mut kept = String::new();
    let mut copied_until = 0;
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'"' {
            index = string_end(bytes, index);
            continue;
        }
        if let Some(end) = comment_end(bytes, index) {
            kept.push_str(&code[copied_until..index]);
            copied_until = end;
            index = end;
            continue;
        }
        index += 1;
    }
    kept.push_str(&code[copied_until..]);
    kept.trim().to_string()
}
