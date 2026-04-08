#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Word(String),
    Integer(i64),
    Symbol(char),
    Arrow,
    String(String),
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            c if c.is_whitespace() => {}
            '{' | '}' | '(' | ')' | ';' | ',' | '=' | '+' | '*' | '/' | ':' | '.' | '<' | '>'
            | '?' => tokens.push(Token::Symbol(ch)),
            '-' => {
                if chars.peek().copied() == Some('>') {
                    chars.next();
                    tokens.push(Token::Arrow);
                } else {
                    tokens.push(Token::Symbol('-'));
                }
            }
            '"' => {
                let mut value = String::new();
                let mut escaped = false;
                loop {
                    let Some(next) = chars.next() else {
                        return Err("unterminated string literal".to_owned());
                    };
                    if escaped {
                        let decoded = match next {
                            'n' => '\n',
                            'r' => '\r',
                            't' => '\t',
                            '\\' => '\\',
                            '"' => '"',
                            other => other,
                        };
                        value.push(decoded);
                        escaped = false;
                        continue;
                    }
                    match next {
                        '\\' => escaped = true,
                        '"' => break,
                        other => value.push(other),
                    }
                }
                tokens.push(Token::String(value));
            }
            c if c.is_ascii_digit() => {
                let mut digits = String::from(c);
                while let Some(next) = chars.peek().copied() {
                    if next.is_ascii_digit() {
                        digits.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let value = digits
                    .parse::<i64>()
                    .map_err(|_| format!("integer literal `{digits}` is out of range"))?;
                tokens.push(Token::Integer(value));
            }
            c if is_ident_start(c) => {
                let mut word = String::from(c);
                while let Some(next) = chars.peek().copied() {
                    if is_ident_continue(next) {
                        word.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if word == "wgsl" && next_non_whitespace_char(&chars) == Some('{') {
                    let source = consume_wgsl_block(&mut chars)?;
                    tokens.push(Token::String(source));
                } else {
                    tokens.push(Token::Word(word));
                }
            }
            other => return Err(format!("unexpected character `{other}`")),
        }
    }

    Ok(tokens)
}

pub fn describe_token(token: &Token) -> String {
    match token {
        Token::Word(value) => format!("identifier `{value}`"),
        Token::Integer(value) => format!("integer `{value}`"),
        Token::Symbol(value) => format!("symbol `{value}`"),
        Token::Arrow => "symbol `->`".to_owned(),
        Token::String(value) => format!("string \"{value}\""),
    }
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn next_non_whitespace_char(chars: &std::iter::Peekable<std::str::Chars<'_>>) -> Option<char> {
    let mut clone = chars.clone();
    while let Some(ch) = clone.peek().copied() {
        if ch.is_whitespace() {
            clone.next();
            continue;
        }
        return Some(ch);
    }
    None
}

fn consume_wgsl_block(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> Result<String, String> {
    while let Some(ch) = chars.peek().copied() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }
        break;
    }
    if chars.next() != Some('{') {
        return Err("wgsl block must start with `{`".to_owned());
    }

    let mut depth = 1usize;
    let mut out = String::new();
    let mut in_string = false;
    let mut escape = false;
    while let Some(ch) = chars.next() {
        if in_string {
            out.push(ch);
            if escape {
                escape = false;
                continue;
            }
            match ch {
                '\\' => escape = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match ch {
            '"' => {
                in_string = true;
                out.push(ch);
            }
            '{' => {
                depth += 1;
                out.push(ch);
            }
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Ok(out.trim().to_owned());
                }
                out.push(ch);
            }
            other => out.push(other),
        }
    }

    Err("unterminated wgsl block".to_owned())
}
