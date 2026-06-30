use super::*;

impl Parser {
    pub(super) fn parse_expr(&mut self) -> Result<AstExpr, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<AstExpr, String> {
        let mut expr = self.parse_and()?;
        loop {
            if self.peek_symbol_pair('|', '|') {
                self.expect_symbol('|')?;
                self.expect_symbol('|')?;
                let rhs = self.parse_and()?;
                expr = AstExpr::Binary {
                    op: AstBinaryOp::Or,
                    lhs: Box::new(expr),
                    rhs: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<AstExpr, String> {
        let mut expr = self.parse_equality()?;
        loop {
            if self.peek_symbol_pair('&', '&') {
                self.expect_symbol('&')?;
                self.expect_symbol('&')?;
                let rhs = self.parse_equality()?;
                expr = AstExpr::Binary {
                    op: AstBinaryOp::And,
                    lhs: Box::new(expr),
                    rhs: Box::new(rhs),
                };
            } else {
                break;
            }
        }
        Ok(expr)
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
            } else if self.peek_symbol_pair('!', '=') {
                self.expect_symbol('!')?;
                self.expect_symbol('=')?;
                let rhs = self.parse_comparison()?;
                expr = AstExpr::Binary {
                    op: AstBinaryOp::Ne,
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
            if self.peek_symbol_pair('<', '=') {
                self.expect_symbol('<')?;
                self.expect_symbol('=')?;
                let rhs = self.parse_additive()?;
                expr = AstExpr::Binary {
                    op: AstBinaryOp::Le,
                    lhs: Box::new(expr),
                    rhs: Box::new(rhs),
                };
            } else if self.peek_symbol_pair('>', '=') {
                self.expect_symbol('>')?;
                self.expect_symbol('=')?;
                let rhs = self.parse_additive()?;
                expr = AstExpr::Binary {
                    op: AstBinaryOp::Ge,
                    lhs: Box::new(expr),
                    rhs: Box::new(rhs),
                };
            } else if self.peek_symbol('<') {
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
            } else if self.peek_symbol('%') {
                self.expect_symbol('%')?;
                let rhs = self.parse_unary()?;
                expr = AstExpr::Binary {
                    op: AstBinaryOp::Rem,
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
        if self.peek_symbol('!') {
            self.expect_symbol('!')?;
            let operand = self.parse_unary()?;
            return Ok(AstExpr::Unary {
                op: nuis_semantics::model::AstUnaryOp::Not,
                operand: Box::new(operand),
            });
        }
        if self.peek_symbol('-') {
            self.expect_symbol('-')?;
            let operand = self.parse_unary()?;
            return Ok(AstExpr::Unary {
                op: nuis_semantics::model::AstUnaryOp::Neg,
                operand: Box::new(operand),
            });
        }
        if self.peek_symbol('*') {
            self.expect_symbol('*')?;
            let operand = self.parse_unary()?;
            return Ok(AstExpr::Unary {
                op: nuis_semantics::model::AstUnaryOp::Deref,
                operand: Box::new(operand),
            });
        }
        self.parse_postfix()
    }

    pub(super) fn parse_postfix(&mut self) -> Result<AstExpr, String> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.peek_symbol('(') {
                self.expect_symbol('(')?;
                let args = self.parse_argument_list(')')?;
                self.expect_symbol(')')?;
                expr = match expr {
                    AstExpr::Var(callee) => AstExpr::Call {
                        callee,
                        generic_args: Vec::new(),
                        args,
                    },
                    other if self.qualified_expr_prefers_constructor(&other) => AstExpr::Call {
                        callee: self
                            .qualified_expr_path(&other)
                            .expect("qualified constructor path exists"),
                        generic_args: Vec::new(),
                        args,
                    },
                    other => AstExpr::Invoke {
                        callee: Box::new(other),
                        args,
                    },
                };
            } else if self.peek_symbol('.') {
                self.expect_symbol('.')?;
                let member = self.expect_ident()?;
                let explicit_generic_args =
                    self.try_parse_expr_type_arg_list_followed_by_call_only()?;
                if self.peek_symbol('(') {
                    self.expect_symbol('(')?;
                    let args = self.parse_argument_list(')')?;
                    self.expect_symbol(')')?;
                    let namespace_candidate = AstExpr::FieldAccess {
                        base: Box::new(expr.clone()),
                        field: member.clone(),
                    };
                    expr = if self.qualified_expr_prefers_constructor(&namespace_candidate)
                        && (!args.is_empty() || matches!(expr, AstExpr::Var(_)))
                    {
                        AstExpr::Call {
                            callee: self
                                .qualified_expr_path(&namespace_candidate)
                                .expect("qualified constructor path exists"),
                            generic_args: explicit_generic_args.unwrap_or_default(),
                            args,
                        }
                    } else {
                        AstExpr::MethodCall {
                            receiver: Box::new(expr),
                            method: member,
                            generic_args: explicit_generic_args.unwrap_or_default(),
                            args,
                        }
                    };
                } else {
                    expr = AstExpr::FieldAccess {
                        base: Box::new(expr),
                        field: member,
                    };
                }
            } else if self.peek_symbol('[') {
                self.expect_symbol('[')?;
                let index = self.parse_expr()?;
                self.expect_symbol(']')?;
                expr = AstExpr::Call {
                    callee: "load_at".to_owned(),
                    generic_args: Vec::new(),
                    args: vec![expr, index],
                };
            } else if self.allow_struct_literals
                && self.peek_symbol('{')
                && self.qualified_expr_prefers_constructor(&expr)
            {
                let type_name = self
                    .qualified_expr_path(&expr)
                    .expect("qualified constructor path exists");
                self.expect_symbol('{')?;
                let fields = self.parse_struct_field_list()?;
                self.expect_symbol('}')?;
                expr = AstExpr::StructLiteral {
                    type_name,
                    type_args: Vec::new(),
                    fields,
                };
            } else if self.peek_symbol('?') {
                self.expect_symbol('?')?;
                expr = AstExpr::Try(Box::new(expr));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<AstExpr, String> {
        match self.next() {
            Some(Token::Symbol('|')) => {
                let params = if self.peek_symbol('|') {
                    Vec::new()
                } else {
                    self.parse_lambda_param_list()?
                };
                self.expect_symbol('|')?;
                let return_type = self.parse_optional_return_type()?;
                let body = self.parse_block_with_tail_expr()?;
                Ok(AstExpr::Lambda {
                    params,
                    return_type,
                    body,
                })
            }
            Some(Token::Word(word)) if word == "if" => {
                let condition = self.parse_condition_expr()?;
                let then_body = self.parse_block_with_tail_expr()?;
                if !self.peek_word("else") {
                    return Err("`if` expression currently requires `else`".to_owned());
                }
                self.expect_word("else")?;
                let else_body = self.parse_block_with_tail_expr()?;
                Ok(AstExpr::If {
                    condition: Box::new(condition),
                    then_body,
                    else_body,
                })
            }
            Some(Token::Word(word)) if word == "match" => {
                self.cursor = self.cursor.saturating_sub(1);
                let (value, arms) = self.parse_match_expr_parts(true)?;
                Ok(AstExpr::Match {
                    value: Box::new(value),
                    arms,
                })
            }
            Some(Token::Word(word)) if word == "instantiate" => {
                let domain = self.expect_ident()?;
                let unit = self.expect_ident()?;
                Ok(AstExpr::Instantiate { domain, unit })
            }
            Some(Token::Word(word)) if word == "true" => Ok(AstExpr::Bool(true)),
            Some(Token::Word(word)) if word == "false" => Ok(AstExpr::Bool(false)),
            Some(Token::String(text)) => Ok(AstExpr::Text(text)),
            Some(Token::Integer(value)) => Ok(AstExpr::Int(value)),
            Some(Token::Float(value)) => Ok(AstExpr::Float(value)),
            Some(Token::Word(name)) => {
                let explicit_generic_args =
                    self.try_parse_expr_type_arg_list_followed_by_call_or_literal()?;
                if self.peek_symbol('(') {
                    self.expect_symbol('(')?;
                    let args = self.parse_argument_list(')')?;
                    self.expect_symbol(')')?;
                    Ok(AstExpr::Call {
                        callee: name,
                        generic_args: explicit_generic_args.unwrap_or_default(),
                        args,
                    })
                } else if self.allow_struct_literals && self.peek_symbol('{') {
                    self.expect_symbol('{')?;
                    let fields = self.parse_struct_field_list()?;
                    self.expect_symbol('}')?;
                    Ok(AstExpr::StructLiteral {
                        type_name: name,
                        type_args: explicit_generic_args.unwrap_or_default(),
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

    pub(super) fn parse_argument_list(&mut self, terminator: char) -> Result<Vec<AstExpr>, String> {
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

    pub(super) fn try_parse_expr_type_arg_list_followed_by_call_or_literal(
        &mut self,
    ) -> Result<Option<Vec<AstTypeRef>>, String> {
        if !self.peek_symbol('<') {
            return Ok(None);
        }
        let checkpoint = self.cursor;
        let parsed = self.parse_type_arg_list();
        match parsed {
            Ok(type_args)
                if self.peek_symbol('(')
                    || (self.allow_struct_literals && self.peek_symbol('{')) =>
            {
                Ok(Some(type_args))
            }
            Ok(_) => {
                self.cursor = checkpoint;
                Ok(None)
            }
            Err(_) => {
                self.cursor = checkpoint;
                Ok(None)
            }
        }
    }

    pub(super) fn try_parse_expr_type_arg_list_followed_by_call_only(
        &mut self,
    ) -> Result<Option<Vec<AstTypeRef>>, String> {
        if !self.peek_symbol('<') {
            return Ok(None);
        }
        let checkpoint = self.cursor;
        let parsed = self.parse_type_arg_list();
        match parsed {
            Ok(type_args) if self.peek_symbol('(') => Ok(Some(type_args)),
            Ok(_) => {
                self.cursor = checkpoint;
                Ok(None)
            }
            Err(_) => {
                self.cursor = checkpoint;
                Ok(None)
            }
        }
    }

    pub(super) fn parse_lambda_param_list(&mut self) -> Result<Vec<AstParam>, String> {
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
}
