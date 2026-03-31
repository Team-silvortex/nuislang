use nuis_semantics::model::{NirFunction, NirModule, NirStmt, NirValue};

pub fn frontend_name() -> &'static str {
    "nuisc-parser-minimal"
}

pub fn parse_nuis_module(input: &str) -> Result<NirModule, String> {
    let tokens = tokenize(input)?;
    let mut parser = Parser::new(tokens);
    parser.parse_module()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word(String),
    Integer(i64),
    Symbol(char),
    String(String),
}

struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, cursor: 0 }
    }

    fn parse_module(&mut self) -> Result<NirModule, String> {
        self.expect_word("mod")?;
        let domain = self.expect_ident()?;
        let name = self.expect_ident()?;
        self.expect_symbol('{')?;

        let mut functions = Vec::new();
        while !self.peek_symbol('}') {
            functions.push(self.parse_function()?);
        }

        self.expect_symbol('}')?;
        self.expect_eof()?;

        Ok(NirModule {
            domain,
            name,
            functions,
        })
    }

    fn parse_function(&mut self) -> Result<NirFunction, String> {
        self.expect_word("fn")?;
        let name = self.expect_ident()?;
        self.expect_symbol('(')?;
        self.expect_symbol(')')?;
        self.expect_symbol('{')?;

        let mut body = Vec::new();
        while !self.peek_symbol('}') {
            body.push(self.parse_stmt()?);
        }

        self.expect_symbol('}')?;

        Ok(NirFunction { name, body })
    }

    fn parse_stmt(&mut self) -> Result<NirStmt, String> {
        if self.peek_word("let") {
            return self.parse_let_stmt();
        }

        let callee = self.expect_ident()?;
        if callee != "print" {
            return Err(format!(
                "minimal nuisc frontend currently only supports `let ... = ...;` and `print(...)`, found `{callee}`"
            ));
        }

        self.expect_symbol('(')?;
        let value = self.parse_value()?;
        self.expect_symbol(')')?;
        self.expect_symbol(';')?;
        Ok(NirStmt::Print(value))
    }

    fn parse_let_stmt(&mut self) -> Result<NirStmt, String> {
        self.expect_word("let")?;
        let name = self.expect_ident()?;
        self.expect_symbol('=')?;
        let value = self.parse_value()?;
        self.expect_symbol(';')?;
        Ok(NirStmt::Let { name, value })
    }

    fn parse_value(&mut self) -> Result<NirValue, String> {
        match self.next() {
            Some(Token::String(text)) => Ok(NirValue::Text(text)),
            Some(Token::Integer(value)) => Ok(NirValue::Int(value)),
            Some(Token::Word(name)) => Ok(NirValue::Var(name)),
            Some(other) => Err(format!(
                "minimal nuisc frontend expected string, integer, or identifier, found {}",
                describe_token(&other)
            )),
            None => Err("minimal nuisc frontend expected value, found end of input".to_owned()),
        }
    }

    fn expect_word(&mut self, expected: &str) -> Result<(), String> {
        match self.next() {
            Some(Token::Word(actual)) if actual == expected => Ok(()),
            Some(other) => Err(format!("expected `{expected}`, found {}", describe_token(&other))),
            None => Err(format!("expected `{expected}`, found end of input")),
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.next() {
            Some(Token::Word(actual)) => Ok(actual),
            Some(other) => Err(format!("expected identifier, found {}", describe_token(&other))),
            None => Err("expected identifier, found end of input".to_owned()),
        }
    }

    fn expect_symbol(&mut self, expected: char) -> Result<(), String> {
        match self.next() {
            Some(Token::Symbol(actual)) if actual == expected => Ok(()),
            Some(other) => Err(format!(
                "expected `{expected}`, found {}",
                describe_token(&other)
            )),
            None => Err(format!("expected `{expected}`, found end of input")),
        }
    }

    fn expect_eof(&self) -> Result<(), String> {
        if self.cursor == self.tokens.len() {
            Ok(())
        } else {
            Err("unexpected trailing tokens after module".to_owned())
        }
    }

    fn peek_symbol(&self, expected: char) -> bool {
        matches!(self.tokens.get(self.cursor), Some(Token::Symbol(actual)) if *actual == expected)
    }

    fn peek_word(&self, expected: &str) -> bool {
        matches!(self.tokens.get(self.cursor), Some(Token::Word(actual)) if actual == expected)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.cursor).cloned();
        if token.is_some() {
            self.cursor += 1;
        }
        token
    }
}

fn describe_token(token: &Token) -> String {
    match token {
        Token::Word(value) => format!("identifier `{value}`"),
        Token::Integer(value) => format!("integer `{value}`"),
        Token::Symbol(value) => format!("symbol `{value}`"),
        Token::String(value) => format!("string \"{value}\""),
    }
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
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

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}
