#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Word(String),
    Integer(i64),
    Symbol(char),
    String(String),
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            c if c.is_whitespace() => {}
            '{' | '}' | '(' | ')' | ';' | ',' | '=' => tokens.push(Token::Symbol(ch)),
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
                tokens.push(Token::Word(word));
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
        Token::String(value) => format!("string \"{value}\""),
    }
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}
