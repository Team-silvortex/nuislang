#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Word(String),
    Integer(i64),
    Float(String),
    Symbol(char),
    Arrow,
    String(String),
    DocComment(String),
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            c if c.is_whitespace() => {}
            '/' if chars.peek().copied() == Some('/') => {
                chars.next();
                if chars.peek().copied() == Some('/') {
                    chars.next();
                    tokens.push(Token::DocComment(consume_doc_comment(&mut chars)));
                } else {
                    skip_line_comment(&mut chars);
                }
            }
            '/' if chars.peek().copied() == Some('*') => {
                chars.next();
                skip_block_comment(&mut chars)?;
            }
            '{' | '}' | '(' | ')' | '[' | ']' | ';' | ',' | '=' | '!' | '+' | '*' | '/' | '%'
            | ':' | '.' | '<' | '>' | '?' | '@' | '|' | '&' => tokens.push(Token::Symbol(ch)),
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
                    } else if next == '.' {
                        let mut lookahead = chars.clone();
                        lookahead.next();
                        if lookahead
                            .peek()
                            .copied()
                            .is_some_and(|ch| ch.is_ascii_digit())
                        {
                            digits.push(next);
                            chars.next();
                            while let Some(fraction) = chars.peek().copied() {
                                if fraction.is_ascii_digit() {
                                    digits.push(fraction);
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            tokens.push(Token::Float(digits.clone()));
                            digits.clear();
                        }
                        break;
                    } else {
                        break;
                    }
                }
                if digits.is_empty() {
                    continue;
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

const STAGE_NEUTRAL_TOKEN_STREAM_PROTOCOL: &str = "nuis-token-stream-v1";

pub(super) fn render_stage_neutral_token_stream(input: &str) -> Result<String, String> {
    let tokens = tokenize(input)?;
    let mut out = format!("{STAGE_NEUTRAL_TOKEN_STREAM_PROTOCOL}\n");
    for token in tokens {
        match token {
            Token::Word(value) => push_hex_token(&mut out, "word", value.as_bytes()),
            Token::Integer(value) => out.push_str(&format!("integer\t{value}\n")),
            Token::Float(value) => push_hex_token(&mut out, "float", value.as_bytes()),
            Token::Symbol(value) => out.push_str(&format!("symbol\t{}\n", value as u32)),
            Token::Arrow => out.push_str("arrow\n"),
            Token::String(value) => push_hex_token(&mut out, "string", value.as_bytes()),
            Token::DocComment(value) => push_hex_token(&mut out, "doc-comment", value.as_bytes()),
        }
    }
    Ok(out)
}

pub(super) fn verify_stage_neutral_token_stream(input: &str, rendered: &str) -> Result<(), String> {
    let expected = tokenize(input)?;
    let decoded = parse_stage_neutral_token_stream(rendered)?;
    if decoded != expected {
        return Err("stage-neutral token stream does not match source tokenization".to_owned());
    }
    if render_stage_neutral_token_stream(input)? != rendered {
        return Err("stage-neutral token stream is not canonically rendered".to_owned());
    }
    Ok(())
}

fn parse_stage_neutral_token_stream(source: &str) -> Result<Vec<Token>, String> {
    if !source.ends_with('\n') || source.contains('\r') {
        return Err("stage-neutral token stream must use canonical LF lines".to_owned());
    }
    let mut lines = source.split_terminator('\n');
    if lines.next() != Some(STAGE_NEUTRAL_TOKEN_STREAM_PROTOCOL) {
        return Err(format!(
            "stage-neutral token stream must begin with `{STAGE_NEUTRAL_TOKEN_STREAM_PROTOCOL}`"
        ));
    }
    let mut tokens = Vec::new();
    for line in lines {
        if line == "arrow" {
            tokens.push(Token::Arrow);
            continue;
        }
        let (kind, payload) = line
            .split_once('\t')
            .ok_or_else(|| format!("malformed stage-neutral token record `{line}`"))?;
        let token = match kind {
            "word" => Token::Word(decode_hex_utf8(payload, kind)?),
            "integer" => Token::Integer(
                payload
                    .parse::<i64>()
                    .map_err(|_| format!("invalid integer token payload `{payload}`"))?,
            ),
            "float" => Token::Float(decode_hex_utf8(payload, kind)?),
            "symbol" => {
                let scalar = payload
                    .parse::<u32>()
                    .map_err(|_| format!("invalid symbol token payload `{payload}`"))?;
                Token::Symbol(char::from_u32(scalar).ok_or_else(|| {
                    format!("symbol token payload `{payload}` is not a Unicode scalar")
                })?)
            }
            "string" => Token::String(decode_hex_utf8(payload, kind)?),
            "doc-comment" => Token::DocComment(decode_hex_utf8(payload, kind)?),
            _ => return Err(format!("unsupported stage-neutral token kind `{kind}`")),
        };
        tokens.push(token);
    }
    Ok(tokens)
}

fn push_hex_token(out: &mut String, kind: &str, bytes: &[u8]) {
    out.push_str(kind);
    out.push('\t');
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out.push('\n');
}

fn decode_hex_utf8(value: &str, kind: &str) -> Result<String, String> {
    if !value.len().is_multiple_of(2) {
        return Err(format!("{kind} token contains odd-length hexadecimal data"));
    }
    let mut bytes = Vec::with_capacity(value.len() / 2);
    for pair in value.as_bytes().chunks_exact(2) {
        let high = hex_nibble(pair[0])?;
        let low = hex_nibble(pair[1])?;
        bytes.push((high << 4) | low);
    }
    String::from_utf8(bytes).map_err(|error| format!("{kind} token is not UTF-8: {error}"))
}

fn hex_nibble(value: u8) -> Result<u8, String> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => Err(format!(
            "stage-neutral token contains non-canonical hexadecimal byte `{}`",
            value as char
        )),
    }
}

pub fn describe_token(token: &Token) -> String {
    match token {
        Token::Word(value) => format!("identifier `{value}`"),
        Token::Integer(value) => format!("integer `{value}`"),
        Token::Float(value) => format!("float `{value}`"),
        Token::Symbol(value) => format!("symbol `{value}`"),
        Token::Arrow => "symbol `->`".to_owned(),
        Token::String(value) => format!("string \"{value}\""),
        Token::DocComment(value) => format!("doc comment \"{value}\""),
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
        if ch == '/' {
            let mut lookahead = clone.clone();
            lookahead.next();
            match lookahead.peek().copied() {
                Some('/') => {
                    clone.next();
                    clone.next();
                    skip_line_comment(&mut clone);
                    continue;
                }
                Some('*') => {
                    clone.next();
                    clone.next();
                    if skip_block_comment(&mut clone).is_err() {
                        return None;
                    }
                    continue;
                }
                _ => {}
            }
        }
        return Some(ch);
    }
    None
}

fn skip_line_comment(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    for ch in chars.by_ref() {
        if ch == '\n' {
            break;
        }
    }
}

fn consume_doc_comment(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut text = String::new();
    for ch in chars.by_ref() {
        if ch == '\n' {
            break;
        }
        text.push(ch);
    }
    if let Some(stripped) = text.strip_prefix(' ') {
        stripped.to_owned()
    } else {
        text
    }
}

fn skip_block_comment(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Result<(), String> {
    let mut depth = 1usize;
    while let Some(ch) = chars.next() {
        match ch {
            '/' if chars.peek().copied() == Some('*') => {
                chars.next();
                depth += 1;
            }
            '*' if chars.peek().copied() == Some('/') => {
                chars.next();
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Ok(());
                }
            }
            _ => {}
        }
    }
    Err("unterminated block comment".to_owned())
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
    let mut in_line_comment = false;
    let mut block_comment_depth = 0usize;
    while let Some(ch) = chars.next() {
        if in_line_comment {
            out.push(ch);
            if ch == '\n' {
                in_line_comment = false;
            }
            continue;
        }

        if block_comment_depth > 0 {
            out.push(ch);
            match ch {
                '/' if chars.peek().copied() == Some('*') => {
                    out.push('*');
                    chars.next();
                    block_comment_depth += 1;
                }
                '*' if chars.peek().copied() == Some('/') => {
                    out.push('/');
                    chars.next();
                    block_comment_depth = block_comment_depth.saturating_sub(1);
                }
                _ => {}
            }
            continue;
        }

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
            '/' if chars.peek().copied() == Some('/') => {
                out.push(ch);
                out.push('/');
                chars.next();
                in_line_comment = true;
            }
            '/' if chars.peek().copied() == Some('*') => {
                out.push(ch);
                out.push('*');
                chars.next();
                block_comment_depth = 1;
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

#[cfg(test)]
mod tests {
    use super::{
        parse_stage_neutral_token_stream, render_stage_neutral_token_stream, tokenize,
        verify_stage_neutral_token_stream, Token,
    };

    #[test]
    fn tokenizes_wgsl_block_with_comments_without_breaking_brace_depth() {
        let tokens = tokenize(
            r#"
shader_inline_wgsl("demo", wgsl {
  // this comment mentions { braces } that should be ignored
  /* block comment { also } ignored */
  struct VsOut {
    @builtin(position) pos: vec4<f32>,
  };

  @vertex
  fn vs_main() -> VsOut {
    var out: VsOut;
    return out;
  }
})
"#,
        )
        .expect("wgsl block tokenizes");

        assert!(tokens
            .iter()
            .any(|token| matches!(token, Token::String(source) if source.contains("@vertex"))));
        assert_eq!(
            tokens
                .iter()
                .filter(|token| matches!(token, Token::String(_)))
                .count(),
            2
        );
    }

    #[test]
    fn tokenizes_nuis_source_with_line_and_block_comments() {
        let tokens = tokenize(
            r#"
// module header
mod cpu main {
  /* comment before function */
  fn add(a: i32, b: i32) -> i32 {
    let sum = a + b; // trailing comment
    sum
  }
}
"#,
        )
        .expect("nuis source tokenizes");

        assert!(tokens.contains(&Token::Word("mod".to_owned())));
        assert!(tokens.contains(&Token::Word("add".to_owned())));
        assert!(tokens.contains(&Token::Word("sum".to_owned())));
        assert!(
            !tokens.contains(&Token::Word("comment".to_owned())),
            "comment text should not become normal tokens"
        );
    }

    #[test]
    fn rejects_unterminated_block_comment() {
        let error = tokenize("mod cpu main { /* missing end ").expect_err("comment should fail");
        assert!(error.contains("unterminated block comment"));
    }

    #[test]
    fn tokenizes_doc_comments_as_dedicated_tokens() {
        let tokens = tokenize(
            r#"
/// adds two values
mod cpu main {}
"#,
        )
        .expect("doc comments tokenize");

        assert!(tokens.iter().any(|token| matches!(
            token,
            Token::DocComment(text) if text == "adds two values"
        )));
    }

    #[test]
    fn stage_neutral_token_stream_round_trips_without_rust_layout_identity() {
        let source =
            "/// label\nmod cpu Main { fn main() -> i64 { let value = \"\"; return 7; } }\n";
        let rendered = render_stage_neutral_token_stream(source).expect("render token stream");
        let parsed = parse_stage_neutral_token_stream(&rendered).expect("parse token stream");

        assert_eq!(parsed, tokenize(source).expect("tokenize source"));
        assert!(rendered.contains("string\t\n"));
        verify_stage_neutral_token_stream(source, &rendered).expect("verify token stream");
    }
}
