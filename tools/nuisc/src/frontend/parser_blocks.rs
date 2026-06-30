use super::*;

impl Parser {
    pub(super) fn parse_block_with_tail_expr(&mut self) -> Result<Vec<AstStmt>, String> {
        self.expect_symbol('{')?;
        let body = self.parse_block_body(true)?;
        self.expect_symbol('}')?;
        Ok(body)
    }

    pub(super) fn parse_stmt_block(&mut self) -> Result<Vec<AstStmt>, String> {
        self.expect_symbol('{')?;
        let body = self.parse_block_body(false)?;
        self.expect_symbol('}')?;
        Ok(body)
    }

    pub(super) fn parse_block_body(
        &mut self,
        allow_tail_expr_stmt: bool,
    ) -> Result<Vec<AstStmt>, String> {
        let mut body = Vec::new();
        while !self.peek_symbol('}') {
            if allow_tail_expr_stmt && self.can_start_tail_expr_stmt() {
                let checkpoint = self.cursor;
                let parsed_expr = self.parse_expr();
                match parsed_expr {
                    Ok(expr) if self.peek_symbol('}') => {
                        body.push(AstStmt::Return(Some(expr)));
                        break;
                    }
                    Ok(_) | Err(_) => {
                        self.cursor = checkpoint;
                    }
                }
            }
            body.push(self.parse_stmt()?);
        }
        Ok(body)
    }

    pub(super) fn can_start_tail_expr_stmt(&self) -> bool {
        !self.peek_symbol('}')
            && !self.peek_word("let")
            && !self.peek_word("const")
            && !self.peek_word("link")
            && !self.peek_word("while")
            && !self.peek_word("loop")
            && !self.peek_word("break")
            && !self.peek_word("continue")
            && !self.peek_word("return")
            && !self.peek_word("mod")
    }

    pub(super) fn parse_struct_field_list(&mut self) -> Result<Vec<(String, AstExpr)>, String> {
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
                if self.peek_symbol('}') {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(fields)
    }
}
