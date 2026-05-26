use nuis_semantics::model::{
    AstBinaryOp, AstExpr, AstExternFunction, AstExternInterface, AstFunction, AstModule, AstParam,
    AstStmt, AstStructDef, AstStructField, AstTypeRef, TestClockDomain, TestClockPolicy,
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
        let mut uses = Vec::new();
        let mut externs = Vec::new();
        let mut extern_interfaces = Vec::new();
        while self.peek_word("use") {
            uses.push(self.parse_use_decl()?);
        }
        while self.peek_word("extern") {
            let abi = self.parse_extern_abi()?;
            if self.peek_word("interface") {
                extern_interfaces.push(self.parse_extern_interface(abi)?);
            } else {
                externs.push(self.parse_extern_function_with_abi(abi, None)?);
            }
        }
        self.expect_word("mod")?;
        let domain = self.expect_ident()?;
        let unit = self.expect_ident()?;
        self.expect_symbol('{')?;

        let mut structs = Vec::new();
        let mut functions = Vec::new();
        while !self.peek_symbol('}') {
            if self.peek_word("mod") {
                return Err("nested mod definitions are not allowed".to_owned());
            }
            if self.peek_word("extern") {
                let abi = self.parse_extern_abi()?;
                if self.peek_word("interface") {
                    extern_interfaces.push(self.parse_extern_interface(abi)?);
                } else {
                    externs.push(self.parse_extern_function_with_abi(abi, None)?);
                }
            } else if self.peek_word("struct") {
                structs.push(self.parse_struct_def()?);
            } else {
                functions.push(self.parse_function()?);
            }
        }

        self.expect_symbol('}')?;
        self.expect_eof()?;

        Ok(AstModule {
            uses,
            domain,
            unit,
            externs,
            extern_interfaces,
            structs,
            functions,
        })
    }

    fn parse_use_decl(&mut self) -> Result<nuis_semantics::model::AstUse, String> {
        self.expect_word("use")?;
        let domain = self.expect_ident()?;
        let unit = self.expect_ident()?;
        self.expect_symbol(';')?;
        Ok(nuis_semantics::model::AstUse { domain, unit })
    }

    fn parse_struct_def(&mut self) -> Result<AstStructDef, String> {
        self.expect_word("struct")?;
        let name = self.expect_ident()?;
        self.expect_symbol('{')?;
        let mut fields = Vec::new();
        while !self.peek_symbol('}') {
            let field_name = self.expect_ident()?;
            self.expect_symbol(':')?;
            let ty = self.parse_type_ref()?;
            fields.push(AstStructField {
                name: field_name,
                ty,
            });
            if self.peek_symbol(',') {
                self.expect_symbol(',')?;
            } else {
                break;
            }
        }
        self.expect_symbol('}')?;
        Ok(AstStructDef { name, fields })
    }

    fn parse_extern_abi(&mut self) -> Result<String, String> {
        self.expect_word("extern")?;
        Ok(match self.tokens.get(self.cursor) {
            Some(Token::String(value)) => {
                let abi = value.clone();
                self.cursor += 1;
                abi
            }
            _ => "nurs".to_owned(),
        })
    }

    fn parse_extern_interface(&mut self, abi: String) -> Result<AstExternInterface, String> {
        self.expect_word("interface")?;
        let name = self.expect_ident()?;
        self.expect_symbol('{')?;
        let mut methods = Vec::new();
        while !self.peek_symbol('}') {
            methods.push(self.parse_extern_function_with_abi(abi.clone(), Some(name.clone()))?);
        }
        self.expect_symbol('}')?;
        Ok(AstExternInterface { abi, name, methods })
    }

    fn parse_extern_function_with_abi(
        &mut self,
        abi: String,
        interface: Option<String>,
    ) -> Result<AstExternFunction, String> {
        self.expect_word("fn")?;
        let name = self.expect_ident()?;
        self.expect_symbol('(')?;
        let params = if self.peek_symbol(')') {
            Vec::new()
        } else {
            self.parse_param_list()?
        };
        self.expect_symbol(')')?;
        self.expect_arrow()?;
        let return_type = self.parse_type_ref()?;
        self.expect_symbol(';')?;
        Ok(AstExternFunction {
            abi,
            interface,
            name,
            params,
            return_type,
        })
    }

    fn parse_function(&mut self) -> Result<AstFunction, String> {
        let (
            declared_test_name,
            test_ignored,
            test_should_fail,
            test_reason,
            test_timeout_ms,
            test_clock_domain,
            test_clock_policy,
        ) = if self.peek_word("test") {
            self.expect_word("test")?;
            if !self.peek_symbol('(') {
                return Err(
                    "test declarations now require `test(...) fn ...`; the older bare-prefix `test ... fn ...` syntax has been retired"
                        .to_owned(),
                );
            }
            self.parse_test_decl_call_syntax()?
        } else {
            (None, false, false, None, None, None, None)
        };
        let is_async = if self.peek_word("async") {
            self.expect_word("async")?;
            true
        } else {
            false
        };
        self.expect_word("fn")?;
        let name = self.expect_ident()?;
        let test_name = declared_test_name.map(|label| {
            if label.is_empty() {
                name.clone()
            } else {
                label
            }
        });
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
            test_name,
            test_ignored,
            test_should_fail,
            test_reason,
            test_timeout_ms,
            test_clock_domain,
            test_clock_policy,
            is_async,
            params,
            return_type,
            body,
        })
    }

    fn parse_test_decl_call_syntax(
        &mut self,
    ) -> Result<
        (
            Option<String>,
            bool,
            bool,
            Option<String>,
            Option<i64>,
            Option<TestClockDomain>,
            Option<TestClockPolicy>,
        ),
        String,
    > {
        self.expect_symbol('(')?;
        let mut label = Some(String::new());
        let mut ignored = false;
        let mut should_fail = false;
        let mut reason = None;
        let mut timeout_ms = None;
        let mut clock_domain = None;
        let mut clock_policy = None;
        while !self.peek_symbol(')') {
            match self.next() {
                Some(Token::String(value)) => {
                    label = Some(value);
                }
                Some(Token::Word(word)) => {
                    if self.peek_symbol('=') {
                        self.expect_symbol('=')?;
                        match word.as_str() {
                            "ignored" => ignored = self.parse_test_meta_bool()?,
                            "should_fail" => should_fail = self.parse_test_meta_bool()?,
                            "reason" => reason = Some(self.parse_test_meta_string()?),
                            "timeout_ms" => timeout_ms = Some(self.parse_test_meta_int()?),
                            "clock_domain" => clock_domain = Some(self.parse_test_clock_domain()?),
                            "clock_policy" => clock_policy = Some(self.parse_test_clock_policy()?),
                            _ => {
                                return Err(format!(
                                    "unknown test metadata key `{word}` in `test(...)`"
                                ))
                            }
                        }
                    } else {
                        match word.as_str() {
                            "ignored" => ignored = true,
                            "should_fail" => should_fail = true,
                            "reason" => {
                                return Err(
                                    "test metadata key `reason` expects `reason=\"...\"` in `test(...)`"
                                        .to_owned(),
                                )
                            }
                            "timeout_ms" => {
                                return Err(
                                    "test metadata key `timeout_ms` expects `timeout_ms=<integer>` in `test(...)`"
                                        .to_owned(),
                                )
                            }
                            "clock_domain" => {
                                return Err(
                                    "test metadata key `clock_domain` expects `clock_domain=\"...\"` in `test(...)`"
                                        .to_owned(),
                                )
                            }
                            "clock_policy" => {
                                return Err(
                                    "test metadata key `clock_policy` expects `clock_policy=\"...\"` in `test(...)`"
                                        .to_owned(),
                                )
                            }
                            _ => {
                                return Err(format!(
                                    "unknown test metadata flag `{word}` in `test(...)`"
                                ))
                            }
                        }
                    }
                }
                Some(other) => {
                    return Err(format!(
                        "expected string label or test metadata in `test(...)`, found {}",
                        describe_token(&other)
                    ))
                }
                None => return Err("unterminated `test(...)` declaration".to_owned()),
            }
            if self.peek_symbol(',') {
                self.expect_symbol(',')?;
            } else {
                break;
            }
        }
        self.expect_symbol(')')?;
        Ok((
            label,
            ignored,
            should_fail,
            reason,
            timeout_ms,
            clock_domain,
            clock_policy,
        ))
    }

    fn parse_test_meta_bool(&mut self) -> Result<bool, String> {
        match self.next() {
            Some(Token::Word(word)) if word == "true" => Ok(true),
            Some(Token::Word(word)) if word == "false" => Ok(false),
            Some(other) => Err(format!(
                "expected `true` or `false` in test metadata, found {}",
                describe_token(&other)
            )),
            None => {
                Err("expected `true` or `false` in test metadata, found end of input".to_owned())
            }
        }
    }

    fn parse_test_meta_string(&mut self) -> Result<String, String> {
        match self.next() {
            Some(Token::String(value)) => Ok(value),
            Some(other) => Err(format!(
                "expected string literal in test metadata, found {}",
                describe_token(&other)
            )),
            None => Err("expected string literal in test metadata, found end of input".to_owned()),
        }
    }

    fn parse_test_meta_int(&mut self) -> Result<i64, String> {
        match self.next() {
            Some(Token::Integer(value)) => Ok(value),
            Some(other) => Err(format!(
                "expected integer literal in test metadata, found {}",
                describe_token(&other)
            )),
            None => Err("expected integer literal in test metadata, found end of input".to_owned()),
        }
    }

    fn parse_test_clock_domain(&mut self) -> Result<TestClockDomain, String> {
        let raw = self.parse_test_meta_string()?;
        TestClockDomain::parse(&raw).ok_or_else(|| {
            format!(
                "unsupported `clock_domain=\"{}\"`; expected `monotonic`, `wall`, or `global`",
                raw
            )
        })
    }

    fn parse_test_clock_policy(&mut self) -> Result<TestClockPolicy, String> {
        let raw = self.parse_test_meta_string()?;
        TestClockPolicy::parse(&raw)
            .ok_or_else(|| format!("unsupported `clock_policy=\"{}\"`; expected `bridge`", raw))
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
        if self.peek_word("link") {
            return self.parse_link_stmt();
        }
        if self.peek_word("if") {
            return self.parse_if_stmt();
        }
        if self.peek_word("while") {
            return self.parse_while_stmt();
        }
        if self.peek_word("loop") {
            return Err(
                "`loop` is not supported yet; current control flow supports `if` branches, but indefinite loop syntax still needs explicit AST/NIR/YIR loop support"
                    .to_owned(),
            );
        }
        if self.peek_word("break") {
            return self.parse_break_stmt();
        }
        if self.peek_word("continue") {
            return self.parse_continue_stmt();
        }
        if self.peek_word("return") {
            return self.parse_return_stmt();
        }
        if self.peek_word("await") {
            return self.parse_await_stmt();
        }
        if self.peek_word("mod") {
            return Err("nested mod definitions are not allowed".to_owned());
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
            other => Ok(AstStmt::Expr(other)),
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

    fn parse_await_stmt(&mut self) -> Result<AstStmt, String> {
        self.expect_word("await")?;
        let value = self.parse_expr()?;
        self.expect_symbol(';')?;
        Ok(AstStmt::Await(value))
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

    fn parse_link_stmt(&mut self) -> Result<AstStmt, String> {
        self.expect_word("link")?;
        let name = self.expect_ident()?;
        let ty = if self.peek_symbol(':') {
            self.expect_symbol(':')?;
            Some(self.parse_type_ref()?)
        } else {
            None
        };
        self.expect_symbol('=')?;
        let value = if self.peek_word("output") {
            self.expect_word("output")?;
            let expr = self.parse_expr()?;
            AstExpr::Call {
                callee: "data_output_pipe".to_owned(),
                args: vec![expr],
            }
        } else if self.peek_word("input") {
            self.expect_word("input")?;
            let expr = self.parse_expr()?;
            AstExpr::Call {
                callee: "data_input_pipe".to_owned(),
                args: vec![expr],
            }
        } else if self.peek_word("marker") {
            self.expect_word("marker")?;
            let expr = self.parse_expr()?;
            AstExpr::Call {
                callee: "data_marker".to_owned(),
                args: vec![expr],
            }
        } else {
            return Err(
                "link statement currently expects `output <expr>`, `input <expr>`, or `marker <expr>`"
                    .to_owned(),
            );
        };
        self.expect_symbol(';')?;
        Ok(AstStmt::Let { name, ty, value })
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

    fn parse_while_stmt(&mut self) -> Result<AstStmt, String> {
        self.expect_word("while")?;
        let condition = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(AstStmt::While { condition, body })
    }

    fn parse_break_stmt(&mut self) -> Result<AstStmt, String> {
        self.expect_word("break")?;
        self.expect_symbol(';')?;
        Ok(AstStmt::Break)
    }

    fn parse_continue_stmt(&mut self) -> Result<AstStmt, String> {
        self.expect_word("continue")?;
        self.expect_symbol(';')?;
        Ok(AstStmt::Continue)
    }

    fn parse_expr(&mut self) -> Result<AstExpr, String> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<AstExpr, String> {
        let mut expr = self.parse_comparison()?;
        loop {
            if self.peek_double_symbol('=') {
                self.expect_symbol('=')?;
                self.expect_symbol('=')?;
                let rhs = self.parse_comparison()?;
                expr = AstExpr::Binary {
                    op: AstBinaryOp::Eq,
                    lhs: Box::new(expr),
                    rhs: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<AstExpr, String> {
        let mut expr = self.parse_additive()?;
        loop {
            if self.peek_symbol('<') {
                self.expect_symbol('<')?;
                let rhs = self.parse_additive()?;
                expr = AstExpr::Binary {
                    op: AstBinaryOp::Lt,
                    lhs: Box::new(expr),
                    rhs: Box::new(rhs),
                };
            } else if self.peek_symbol('>') {
                self.expect_symbol('>')?;
                let rhs = self.parse_additive()?;
                expr = AstExpr::Binary {
                    op: AstBinaryOp::Gt,
                    lhs: Box::new(expr),
                    rhs: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(expr)
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
        let mut expr = self.parse_unary()?;
        loop {
            if self.peek_symbol('*') {
                self.expect_symbol('*')?;
                let rhs = self.parse_unary()?;
                expr = AstExpr::Binary {
                    op: AstBinaryOp::Mul,
                    lhs: Box::new(expr),
                    rhs: Box::new(rhs),
                };
            } else if self.peek_symbol('/') {
                self.expect_symbol('/')?;
                let rhs = self.parse_unary()?;
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

    fn parse_unary(&mut self) -> Result<AstExpr, String> {
        if self.peek_word("await") {
            self.expect_word("await")?;
            let value = self.parse_unary()?;
            return Ok(AstExpr::Await(Box::new(value)));
        }
        self.parse_postfix()
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
            Some(Token::Word(word)) if word == "instantiate" => {
                let domain = self.expect_ident()?;
                let unit = self.expect_ident()?;
                Ok(AstExpr::Instantiate { domain, unit })
            }
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
                "minimal nuisc frontend expected instantiate, string, integer, identifier, or grouped expression, found {}",
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
            Some(other) => Err(format!("expected `->`, found {}", describe_token(&other))),
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

    fn peek_double_symbol(&self, expected: char) -> bool {
        matches!(self.tokens.get(self.cursor), Some(Token::Symbol(actual)) if *actual == expected)
            && matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol(actual)) if *actual == expected)
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
