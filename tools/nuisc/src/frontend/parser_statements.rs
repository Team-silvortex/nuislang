use super::*;

impl Parser {
    pub(super) fn parse_stmt(&mut self) -> Result<AstStmt, String> {
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
        if self.peek_word("match") {
            return self.parse_match_stmt();
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

        if let Some(stmt) = self.try_parse_assignment_stmt()? {
            return Ok(stmt);
        }

        let expr = self.parse_expr()?;
        if self.peek_symbol('=') {
            self.expect_symbol('=')?;
            let value = self.parse_expr()?;
            self.expect_symbol(';')?;
            return self.rewrite_assignment_stmt(expr, value);
        }
        self.expect_symbol(';')?;
        match expr {
            AstExpr::Call {
                callee,
                generic_args,
                args,
            } if callee == "print" => {
                if !generic_args.is_empty() {
                    return Err("print does not accept explicit generic arguments".to_owned());
                }
                if args.len() != 1 {
                    return Err("print requires exactly one argument".to_owned());
                }
                let value = args.into_iter().next().expect("checked len == 1");
                Ok(AstStmt::Print(value))
            }
            other => Ok(AstStmt::Expr(other)),
        }
    }

    pub(super) fn try_parse_assignment_stmt(&mut self) -> Result<Option<AstStmt>, String> {
        let checkpoint = self.cursor;
        let target = match self.parse_postfix() {
            Ok(target) => target,
            Err(_) => {
                self.cursor = checkpoint;
                return Ok(None);
            }
        };
        let Some(op) = self.peek_assignment_op() else {
            self.cursor = checkpoint;
            return Ok(None);
        };
        self.consume_assignment_op(op)?;
        let value = self.parse_expr()?;
        self.expect_symbol(';')?;
        let stmt = match op {
            AssignmentOp::Assign => self.rewrite_assignment_stmt(target, value)?,
            AssignmentOp::AddAssign => {
                self.rewrite_compound_assignment_stmt(target, AstBinaryOp::Add, value)?
            }
            AssignmentOp::SubAssign => {
                self.rewrite_compound_assignment_stmt(target, AstBinaryOp::Sub, value)?
            }
            AssignmentOp::MulAssign => {
                self.rewrite_compound_assignment_stmt(target, AstBinaryOp::Mul, value)?
            }
            AssignmentOp::DivAssign => {
                self.rewrite_compound_assignment_stmt(target, AstBinaryOp::Div, value)?
            }
            AssignmentOp::RemAssign => {
                self.rewrite_compound_assignment_stmt(target, AstBinaryOp::Rem, value)?
            }
        };
        Ok(Some(stmt))
    }

    fn rewrite_assignment_stmt(&self, target: AstExpr, value: AstExpr) -> Result<AstStmt, String> {
        let expr = match target {
            AstExpr::Var(name) => {
                return Ok(AstStmt::AssignLocal { name, value });
            }
            AstExpr::Call {
                callee,
                generic_args,
                mut args,
            } if callee == "load_at" && generic_args.is_empty() && args.len() == 2 => {
                args.push(value);
                AstExpr::Call {
                    callee: "store_at".to_owned(),
                    generic_args: Vec::new(),
                    args,
                }
            }
            AstExpr::FieldAccess { base, field } => match field.as_str() {
                "value" => AstExpr::Call {
                    callee: "store_value".to_owned(),
                    generic_args: Vec::new(),
                    args: vec![*base, value],
                },
                "next" => AstExpr::Call {
                    callee: "store_next".to_owned(),
                    generic_args: Vec::new(),
                    args: vec![*base, value],
                },
                "len" => {
                    return Err("`.len` is read-only; assignment currently supports `buffer[index]`, `slice[index]`, `ref Node.value`, and `ref Node.next`".to_owned())
                }
                _ => {
                    return Err(format!(
                        "assignment target `.{field}` is not supported yet; current assignment sugar supports `buffer[index]`, `slice[index]`, `ref Node.value`, and `ref Node.next`"
                    ))
                }
            },
            _ => {
                return Err(
                    "assignment target is not supported yet; current assignment sugar supports `buffer[index]`, `slice[index]`, `ref Node.value`, and `ref Node.next`"
                        .to_owned(),
                )
            }
        };
        Ok(AstStmt::Expr(expr))
    }

    fn rewrite_compound_assignment_stmt(
        &self,
        target: AstExpr,
        op: AstBinaryOp,
        value: AstExpr,
    ) -> Result<AstStmt, String> {
        let current = match &target {
            AstExpr::Var(name) => AstExpr::Var(name.clone()),
            AstExpr::Call {
                callee,
                generic_args,
                args,
            } if callee == "load_at" && generic_args.is_empty() && args.len() == 2 => AstExpr::Call {
                callee: "load_at".to_owned(),
                generic_args: Vec::new(),
                args: args.clone(),
            },
            AstExpr::FieldAccess { base, field } => match field.as_str() {
                "value" => AstExpr::FieldAccess {
                    base: base.clone(),
                    field: field.clone(),
                },
                "next" => {
                    return Err(
                        "compound assignment target `.next` is not supported yet; current compound assignment sugar supports `buffer[index]` and `ref Node.value`"
                            .to_owned(),
                    )
                }
                "len" => {
                    return Err(
                        "`.len` is read-only; compound assignment currently supports `buffer[index]`, `slice[index]`, and `ref Node.value`"
                            .to_owned(),
                    )
                }
                _ => {
                    return Err(format!(
                        "compound assignment target `.{field}` is not supported yet; current compound assignment sugar supports `buffer[index]`, `slice[index]`, and `ref Node.value`"
                    ))
                }
            },
            _ => {
                return Err(
                    "compound assignment target is not supported yet; current compound assignment sugar supports `buffer[index]`, `slice[index]`, and `ref Node.value`"
                        .to_owned(),
                )
            }
        };
        self.rewrite_assignment_stmt(
            target,
            AstExpr::Binary {
                op,
                lhs: Box::new(current),
                rhs: Box::new(value),
            },
        )
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
        if self.starts_destructure_let() {
            return self.parse_destructure_let_stmt();
        }
        let mutable = if self.peek_word("mut") {
            self.expect_word("mut")?;
            true
        } else {
            false
        };
        let name = self.expect_ident()?;
        let ty = self.parse_optional_type_annotation()?;
        self.expect_symbol('=')?;
        let value = self.parse_expr()?;
        self.expect_symbol(';')?;
        Ok(AstStmt::Let {
            mutable,
            name,
            ty,
            value,
        })
    }

    fn parse_const_stmt(&mut self) -> Result<AstStmt, String> {
        self.expect_word("const")?;
        let name = self.expect_ident()?;
        let ty = self.parse_optional_type_annotation()?;
        self.expect_symbol('=')?;
        let value = self.parse_expr()?;
        self.expect_symbol(';')?;
        Ok(AstStmt::Const { name, ty, value })
    }

    fn parse_link_stmt(&mut self) -> Result<AstStmt, String> {
        self.expect_word("link")?;
        let name = self.expect_ident()?;
        let ty = self.parse_optional_type_annotation()?;
        self.expect_symbol('=')?;
        let value = if self.peek_word("output") {
            self.expect_word("output")?;
            let expr = self.parse_expr()?;
            AstExpr::Call {
                callee: "data_output_pipe".to_owned(),
                generic_args: Vec::new(),
                args: vec![expr],
            }
        } else if self.peek_word("input") {
            self.expect_word("input")?;
            let expr = self.parse_expr()?;
            AstExpr::Call {
                callee: "data_input_pipe".to_owned(),
                generic_args: Vec::new(),
                args: vec![expr],
            }
        } else if self.peek_word("marker") {
            self.expect_word("marker")?;
            let expr = self.parse_expr()?;
            AstExpr::Call {
                callee: "data_marker".to_owned(),
                generic_args: Vec::new(),
                args: vec![expr],
            }
        } else {
            return Err(
                "link statement currently expects `output <expr>`, `input <expr>`, or `marker <expr>`"
                    .to_owned(),
            );
        };
        self.expect_symbol(';')?;
        Ok(AstStmt::Let {
            mutable: false,
            name,
            ty,
            value,
        })
    }

    fn parse_if_stmt(&mut self) -> Result<AstStmt, String> {
        self.expect_word("if")?;
        let condition = self.parse_condition_expr()?;
        let then_body = self.parse_stmt_block()?;
        let else_body = if self.peek_word("else") {
            self.expect_word("else")?;
            self.parse_stmt_block()?
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
        let condition = self.parse_condition_expr()?;
        let body = self.parse_stmt_block()?;
        Ok(AstStmt::While { condition, body })
    }

    fn parse_match_stmt(&mut self) -> Result<AstStmt, String> {
        let (value, arms) = self.parse_match_expr_parts(false)?;
        Ok(AstStmt::Match { value, arms })
    }

    pub(super) fn parse_match_expr_parts(
        &mut self,
        allow_tail_expr_in_arm: bool,
    ) -> Result<(AstExpr, Vec<AstMatchArm>), String> {
        self.expect_word("match")?;
        let value = self.parse_match_scrutinee_expr()?;
        self.expect_symbol('{')?;
        let mut arms = Vec::new();
        while !self.peek_symbol('}') {
            let pattern = self.parse_match_pattern()?;
            let guard = if self.peek_word("if") {
                self.expect_word("if")?;
                Some(self.parse_condition_expr()?)
            } else {
                None
            };
            self.expect_symbol('=')?;
            self.expect_symbol('>')?;
            let body = if allow_tail_expr_in_arm {
                self.parse_block_with_tail_expr()?
            } else {
                self.parse_stmt_block()?
            };
            arms.push(AstMatchArm {
                pattern,
                guard,
                body,
            });
            if self.peek_symbol(',') {
                self.expect_symbol(',')?;
            }
        }
        self.expect_symbol('}')?;
        Ok((value, arms))
    }

    pub(super) fn parse_match_scrutinee_expr(&mut self) -> Result<AstExpr, String> {
        let old = self.allow_struct_literals;
        self.allow_struct_literals = false;
        let parsed = self.parse_expr();
        self.allow_struct_literals = old;
        parsed
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
}
