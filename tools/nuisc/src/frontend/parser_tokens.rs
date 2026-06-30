use super::*;

impl Parser {
    pub(super) fn expect_word(&mut self, expected: &str) -> Result<(), String> {
        match self.next() {
            Some(Token::Word(actual)) if actual == expected => Ok(()),
            Some(other) => Err(format!(
                "expected `{expected}`, found {}",
                describe_token(&other)
            )),
            None => Err(format!("expected `{expected}`, found end of input")),
        }
    }

    pub(super) fn expect_ident(&mut self) -> Result<String, String> {
        match self.next() {
            Some(Token::Word(actual)) if actual != "true" && actual != "false" => Ok(actual),
            Some(other) => Err(format!(
                "expected identifier, found {}",
                describe_token(&other)
            )),
            None => Err("expected identifier, found end of input".to_owned()),
        }
    }

    pub(super) fn expect_arrow(&mut self) -> Result<(), String> {
        match self.next() {
            Some(Token::Arrow) => Ok(()),
            Some(other) => Err(format!("expected `->`, found {}", describe_token(&other))),
            None => Err("expected `->`, found end of input".to_owned()),
        }
    }

    pub(super) fn expect_symbol(&mut self, expected: char) -> Result<(), String> {
        match self.next() {
            Some(Token::Symbol(actual)) if actual == expected => Ok(()),
            Some(other) => Err(format!(
                "expected `{expected}`, found {}",
                describe_token(&other)
            )),
            None => Err(format!("expected `{expected}`, found end of input")),
        }
    }

    pub(super) fn expect_eof(&self) -> Result<(), String> {
        if self.cursor == self.tokens.len() {
            Ok(())
        } else {
            Err("unexpected trailing tokens after module".to_owned())
        }
    }

    pub(super) fn peek_symbol(&self, expected: char) -> bool {
        matches!(self.tokens.get(self.cursor), Some(Token::Symbol(actual)) if *actual == expected)
    }

    pub(super) fn peek_word(&self, expected: &str) -> bool {
        matches!(self.tokens.get(self.cursor), Some(Token::Word(actual)) if actual == expected)
    }

    pub(super) fn peek_doc_comment(&self) -> bool {
        matches!(self.tokens.get(self.cursor), Some(Token::DocComment(_)))
    }

    pub(super) fn peek_double_symbol(&self, expected: char) -> bool {
        matches!(self.tokens.get(self.cursor), Some(Token::Symbol(actual)) if *actual == expected)
            && matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol(actual)) if *actual == expected)
    }

    pub(super) fn peek_assignment_op(&self) -> Option<AssignmentOp> {
        if self.peek_symbol('+')
            && matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol('=')))
        {
            Some(AssignmentOp::AddAssign)
        } else if self.peek_symbol('-')
            && matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol('=')))
        {
            Some(AssignmentOp::SubAssign)
        } else if self.peek_symbol('*')
            && matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol('=')))
        {
            Some(AssignmentOp::MulAssign)
        } else if self.peek_symbol('/')
            && matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol('=')))
        {
            Some(AssignmentOp::DivAssign)
        } else if self.peek_symbol('%')
            && matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol('=')))
        {
            Some(AssignmentOp::RemAssign)
        } else if self.peek_symbol('=')
            && !matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol('=')))
        {
            Some(AssignmentOp::Assign)
        } else {
            None
        }
    }

    pub(super) fn consume_assignment_op(&mut self, op: AssignmentOp) -> Result<(), String> {
        match op {
            AssignmentOp::Assign => self.expect_symbol('='),
            AssignmentOp::AddAssign => {
                self.expect_symbol('+')?;
                self.expect_symbol('=')
            }
            AssignmentOp::SubAssign => {
                self.expect_symbol('-')?;
                self.expect_symbol('=')
            }
            AssignmentOp::MulAssign => {
                self.expect_symbol('*')?;
                self.expect_symbol('=')
            }
            AssignmentOp::DivAssign => {
                self.expect_symbol('/')?;
                self.expect_symbol('=')
            }
            AssignmentOp::RemAssign => {
                self.expect_symbol('%')?;
                self.expect_symbol('=')
            }
        }
    }

    pub(super) fn peek_symbol_pair(&self, first: char, second: char) -> bool {
        matches!(self.tokens.get(self.cursor), Some(Token::Symbol(actual)) if *actual == first)
            && matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol(actual)) if *actual == second)
    }

    pub(super) fn peek_arrow(&self) -> bool {
        matches!(self.tokens.get(self.cursor), Some(Token::Arrow))
    }

    pub(super) fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.cursor).cloned();
        if token.is_some() {
            self.cursor += 1;
        }
        token
    }
}
