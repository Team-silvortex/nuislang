use nuis_semantics::model::{
    AstBinaryOp, AstExpr, AstFunction, AstModule, AstParam, AstStmt, AstTypeRef,
};

use super::lexer::{describe_token, Token};

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, cursor: 0 }
    }

    pub fn parse_module(&mut self) -> Result<AstModule, String> {
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

        Ok(AstModule {
            domain,
            name,
            functions,
        })
    }

    fn parse_function(&mut self) -> Result<AstFunction, String> {
        self.expect_word("fn")?;
        let name = self.expect_ident()?;
        self.expect_symbol('(')?;
        let params = if self.peek_symbol(')') {
            Vec::new()
        } else {
            self.parse_param_list()?
        };
        self.expect_symbol(')')?;
        let return_type = if self.peek_arrow() {
            self.expect_arrow()?;
            Some(self.parse_type_ref()?)
        } else {
            None
        };
        self.expect_symbol('{')?;

        let mut body = Vec::new();
        while !self.peek_symbol('}') {
            body.push(self.parse_stmt()?);
        }

        self.expect_symbol('}')?;

        Ok(AstFunction {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_param_list(&mut self) -> Result<Vec<AstParam>, String> {
        let mut params = Vec::new();
        loop {
            let name = self.expect_ident()?;
            self.expect_symbol(':')?;
            let ty = self.parse_type_ref()?;
            params.push(AstParam { name, ty });
            if self.peek_symbol(',') {
                self.expect_symbol(',')?;
            } else {
                break;
            }
        }
        Ok(params)
    }

    fn parse_type_ref(&mut self) -> Result<AstTypeRef, String> {
        let is_ref = if self.peek_word("ref") {
            self.expect_word("ref")?;
            true
        } else {
            false
        };
        let name = self.expect_ident()?;
        let generic_args = if self.peek_symbol('<') {
            self.expect_symbol('<')?;
            let mut args = Vec::new();
            loop {
                args.push(self.parse_type_ref()?);
                if self.peek_symbol(',') {
                    self.expect_symbol(',')?;
                } else {
                    break;
                }
            }
            self.expect_symbol('>')?;
            args
        } else {
            Vec::new()
        };
        let is_optional = if self.peek_symbol('?') {
            self.expect_symbol('?')?;
            true
        } else {
            false
        };
        Ok(AstTypeRef {
            name,
            generic_args,
            is_optional,
            is_ref,
        })
    }

    fn parse_stmt(&mut self) -> Result<AstStmt, String> {
        if self.peek_word("let") {
            return self.parse_let_stmt();
        }
        if self.peek_word("const") {
            return self.parse_const_stmt();
        }
        if self.peek_word("if") {
            return self.parse_if_stmt();
        }
        if self.peek_word("return") {
            return self.parse_return_stmt();
        }

        let expr = self.parse_expr()?;
        self.expect_symbol(';')?;
        match expr {
            AstExpr::Call { callee, args } if callee == "print" => {
                if args.len() != 1 {
                    return Err("print requires exactly one argument".to_owned());
                }
                let value = args.into_iter().next().expect("checked len == 1");
                Ok(AstStmt::Print(value))
            }
            other => Err(format!(
                "minimal nuisc frontend currently only supports `let`, `return`, and `print(...)`; found expression statement `{}`",
                render_expr_for_error(&other)
            )),
        }
    }

    fn parse_return_stmt(&mut self) -> Result<AstStmt, String> {
        self.expect_word("return")?;
        if self.peek_symbol(';') {
            self.expect_symbol(';')?;
            return Ok(AstStmt::Return(None));
        }
        let value = self.parse_expr()?;
        self.expect_symbol(';')?;
        Ok(AstStmt::Return(Some(value)))
    }

    fn parse_let_stmt(&mut self) -> Result<AstStmt, String> {
        self.expect_word("let")?;
        let name = self.expect_ident()?;
        let ty = if self.peek_symbol(':') {
            self.expect_symbol(':')?;
            Some(self.parse_type_ref()?)
        } else {
            None
        };
        self.expect_symbol('=')?;
        let value = self.parse_expr()?;
        self.expect_symbol(';')?;
        Ok(AstStmt::Let { name, ty, value })
    }

    fn parse_const_stmt(&mut self) -> Result<AstStmt, String> {
        self.expect_word("const")?;
        let name = self.expect_ident()?;
        self.expect_symbol(':')?;
        let ty = self.parse_type_ref()?;
        self.expect_symbol('=')?;
        let value = self.parse_expr()?;
        self.expect_symbol(';')?;
        Ok(AstStmt::Const { name, ty, value })
    }

    fn parse_if_stmt(&mut self) -> Result<AstStmt, String> {
        self.expect_word("if")?;
        let condition = self.parse_expr()?;
        let then_body = self.parse_block()?;
        let else_body = if self.peek_word("else") {
            self.expect_word("else")?;
            self.parse_block()?
        } else {
            Vec::new()
        };
        Ok(AstStmt::If {
            condition,
            then_body,
            else_body,
        })
    }

    fn parse_expr(&mut self) -> Result<AstExpr, String> {
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Result<AstExpr, String> {
        let mut expr = self.parse_multiplicative()?;
        loop {
            if self.peek_symbol('+') {
                self.expect_symbol('+')?;
                let rhs = self.parse_multiplicative()?;
                expr = AstExpr::Binary {
                    op: AstBinaryOp::Add,
                    lhs: Box::new(expr),
                    rhs: Box::new(rhs),
                };
            } else if self.peek_symbol('-') {
                self.expect_symbol('-')?;
                let rhs = self.parse_multiplicative()?;
                expr = AstExpr::Binary {
                    op: AstBinaryOp::Sub,
                    lhs: Box::new(expr),
                    rhs: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<AstExpr, String> {
        let mut expr = self.parse_postfix()?;
        loop {
            if self.peek_symbol('*') {
                self.expect_symbol('*')?;
                let rhs = self.parse_postfix()?;
                expr = AstExpr::Binary {
                    op: AstBinaryOp::Mul,
                    lhs: Box::new(expr),
                    rhs: Box::new(rhs),
                };
            } else if self.peek_symbol('/') {
                self.expect_symbol('/')?;
                let rhs = self.parse_postfix()?;
                expr = AstExpr::Binary {
                    op: AstBinaryOp::Div,
                    lhs: Box::new(expr),
                    rhs: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_postfix(&mut self) -> Result<AstExpr, String> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.peek_symbol('.') {
                self.expect_symbol('.')?;
                let member = self.expect_ident()?;
                if self.peek_symbol('(') {
                    self.expect_symbol('(')?;
                    let args = self.parse_argument_list(')')?;
                    self.expect_symbol(')')?;
                    expr = AstExpr::MethodCall {
                        receiver: Box::new(expr),
                        method: member,
                        args,
                    };
                } else {
                    expr = AstExpr::FieldAccess {
                        base: Box::new(expr),
                        field: member,
                    };
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<AstExpr, String> {
        match self.next() {
            Some(Token::Word(word)) if word == "true" => Ok(AstExpr::Bool(true)),
            Some(Token::Word(word)) if word == "false" => Ok(AstExpr::Bool(false)),
            Some(Token::String(text)) => Ok(AstExpr::Text(text)),
            Some(Token::Integer(value)) => Ok(AstExpr::Int(value)),
            Some(Token::Word(name)) => {
                if self.peek_symbol('(') {
                    self.expect_symbol('(')?;
                    let args = self.parse_argument_list(')')?;
                    self.expect_symbol(')')?;
                    Ok(AstExpr::Call { callee: name, args })
                } else if self.peek_symbol('{') {
                    self.expect_symbol('{')?;
                    let fields = self.parse_struct_field_list()?;
                    self.expect_symbol('}')?;
                    Ok(AstExpr::StructLiteral {
                        type_name: name,
                        fields,
                    })
                } else {
                    Ok(AstExpr::Var(name))
                }
            }
            Some(Token::Symbol('(')) => {
                let expr = self.parse_expr()?;
                self.expect_symbol(')')?;
                Ok(expr)
            }
            Some(other) => Err(format!(
                "minimal nuisc frontend expected string, integer, identifier, or grouped expression, found {}",
                describe_token(&other)
            )),
            None => Err("minimal nuisc frontend expected value, found end of input".to_owned()),
        }
    }

    fn parse_argument_list(&mut self, terminator: char) -> Result<Vec<AstExpr>, String> {
        let mut args = Vec::new();
        if self.peek_symbol(terminator) {
            return Ok(args);
        }
        loop {
            args.push(self.parse_expr()?);
            if self.peek_symbol(',') {
                self.expect_symbol(',')?;
            } else {
                break;
            }
        }
        Ok(args)
    }

    fn parse_block(&mut self) -> Result<Vec<AstStmt>, String> {
        self.expect_symbol('{')?;
        let mut body = Vec::new();
        while !self.peek_symbol('}') {
            body.push(self.parse_stmt()?);
        }
        self.expect_symbol('}')?;
        Ok(body)
    }

    fn parse_struct_field_list(&mut self) -> Result<Vec<(String, AstExpr)>, String> {
        let mut fields = Vec::new();
        if self.peek_symbol('}') {
            return Ok(fields);
        }
        loop {
            let name = self.expect_ident()?;
            self.expect_symbol(':')?;
            let value = self.parse_expr()?;
            fields.push((name, value));
            if self.peek_symbol(',') {
                self.expect_symbol(',')?;
            } else {
                break;
            }
        }
        Ok(fields)
    }

    fn expect_word(&mut self, expected: &str) -> Result<(), String> {
        match self.next() {
            Some(Token::Word(actual)) if actual == expected => Ok(()),
            Some(other) => Err(format!(
                "expected `{expected}`, found {}",
                describe_token(&other)
            )),
            None => Err(format!("expected `{expected}`, found end of input")),
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.next() {
            Some(Token::Word(actual)) if actual != "true" && actual != "false" => Ok(actual),
            Some(other) => Err(format!(
                "expected identifier, found {}",
                describe_token(&other)
            )),
            None => Err("expected identifier, found end of input".to_owned()),
        }
    }

    fn expect_arrow(&mut self) -> Result<(), String> {
        match self.next() {
            Some(Token::Arrow) => Ok(()),
            Some(other) => Err(format!(
                "expected `->`, found {}",
                describe_token(&other)
            )),
            None => Err("expected `->`, found end of input".to_owned()),
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

    fn peek_arrow(&self) -> bool {
        matches!(self.tokens.get(self.cursor), Some(Token::Arrow))
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.cursor).cloned();
        if token.is_some() {
            self.cursor += 1;
        }
        token
    }
}

fn render_expr_for_error(expr: &AstExpr) -> String {
    match expr {
        AstExpr::Bool(value) => value.to_string(),
        AstExpr::Text(text) => format!("\"{text}\""),
        AstExpr::Int(value) => value.to_string(),
        AstExpr::Var(name) => name.clone(),
        AstExpr::Call { callee, .. } => format!("{callee}(...)"),
        AstExpr::MethodCall { receiver, method, .. } => {
            format!("{}.{}(...)", render_expr_for_error(receiver), method)
        }
        AstExpr::StructLiteral { type_name, .. } => format!("{type_name} {{ ... }}"),
        AstExpr::FieldAccess { base, field } => {
            format!("{}.{}", render_expr_for_error(base), field)
        }
        AstExpr::Binary { op, lhs, rhs } => format!(
            "({} {} {})",
            render_expr_for_error(lhs),
            match op {
                AstBinaryOp::Add => "+",
                AstBinaryOp::Sub => "-",
                AstBinaryOp::Mul => "*",
                AstBinaryOp::Div => "/",
            },
            render_expr_for_error(rhs)
            ),
    }
}
