use super::*;

impl Parser {
    pub(super) fn starts_destructure_let(&self) -> bool {
        matches!(self.tokens.get(self.cursor), Some(Token::Word(_)))
            && (matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol('{')))
                || matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol('<'))))
    }

    pub(super) fn parse_destructure_let_stmt(&mut self) -> Result<AstStmt, String> {
        let type_ref = self.parse_type_ref()?;
        self.expect_symbol('{')?;
        let mut fields = Vec::new();
        while !self.peek_symbol('}') {
            let field = self.expect_ident()?;
            if fields
                .iter()
                .any(|existing: &nuis_semantics::model::AstDestructureField| {
                    existing.field == field
                })
            {
                return Err(format!(
                    "duplicate field `{field}` in destructuring let pattern"
                ));
            }
            let binding = if self.peek_symbol(':') {
                self.expect_symbol(':')?;
                match self.next() {
                    Some(Token::Word(actual)) if actual != "true" && actual != "false" => actual,
                    Some(other) => {
                        return Err(format!(
                            "expected identifier or `_`, found {}",
                            describe_token(&other)
                        ))
                    }
                    None => return Err("expected identifier or `_`, found end of input".to_owned()),
                }
            } else {
                field.clone()
            };
            if binding != "_"
                && fields
                    .iter()
                    .any(|existing: &nuis_semantics::model::AstDestructureField| {
                        existing.binding == binding
                    })
            {
                return Err(format!(
                    "duplicate binding `{binding}` in destructuring let pattern"
                ));
            }
            fields.push(nuis_semantics::model::AstDestructureField { field, binding });
            if !self.peek_symbol(',') {
                break;
            }
            self.expect_symbol(',')?;
        }
        self.expect_symbol('}')?;
        if fields.is_empty() {
            return Err("destructuring let pattern requires at least one field".to_owned());
        }
        self.expect_symbol('=')?;
        let value = self.parse_expr()?;
        self.expect_symbol(';')?;
        Ok(AstStmt::DestructureLet {
            type_ref,
            fields,
            value,
        })
    }
}
