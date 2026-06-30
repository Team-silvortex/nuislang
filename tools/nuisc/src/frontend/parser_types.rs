use super::*;

impl Parser {
    pub(super) fn parse_generic_param_decl_list(&mut self) -> Result<Vec<AstGenericParam>, String> {
        self.expect_symbol('<')?;
        let mut params = Vec::new();
        loop {
            let name = self.expect_ident()?;
            let bounds = if self.peek_symbol(':') {
                self.expect_symbol(':')?;
                let mut bounds = vec![self.parse_type_ref()?];
                while self.peek_symbol('+') {
                    self.expect_symbol('+')?;
                    bounds.push(self.parse_type_ref()?);
                }
                bounds
            } else {
                Vec::new()
            };
            params.push(AstGenericParam { name, bounds });
            if self.peek_symbol(',') {
                self.expect_symbol(',')?;
            } else {
                break;
            }
        }
        self.expect_symbol('>')?;
        Ok(params)
    }

    pub(super) fn parse_where_predicate_list(&mut self) -> Result<Vec<AstWherePredicate>, String> {
        self.expect_word("where")?;
        let mut predicates = Vec::new();
        loop {
            let param_name = self.expect_ident()?;
            self.expect_symbol(':')?;
            let mut bounds = vec![self.parse_type_ref()?];
            while self.peek_symbol('+') {
                self.expect_symbol('+')?;
                bounds.push(self.parse_type_ref()?);
            }
            predicates.push(AstWherePredicate { param_name, bounds });
            if self.peek_symbol(',') {
                self.expect_symbol(',')?;
                if self.peek_symbol('{') {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(predicates)
    }

    pub(super) fn parse_param_list(&mut self) -> Result<Vec<AstParam>, String> {
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

    pub(super) fn parse_type_ref(&mut self) -> Result<AstTypeRef, String> {
        let is_ref = if self.peek_word("ref") {
            self.expect_word("ref")?;
            true
        } else {
            false
        };
        let name = self.parse_qualified_ident()?;
        let generic_args = if self.peek_symbol('<') {
            self.parse_type_arg_list()?
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

    pub(super) fn parse_qualified_ident(&mut self) -> Result<String, String> {
        let mut name = self.expect_ident()?;
        while self.peek_symbol('.') {
            self.expect_symbol('.')?;
            name.push('.');
            name.push_str(&self.expect_ident()?);
        }
        Ok(name)
    }

    pub(super) fn qualified_expr_path(&self, expr: &AstExpr) -> Option<String> {
        match expr {
            AstExpr::Var(name) => Some(name.clone()),
            AstExpr::FieldAccess { base, field } => {
                Some(format!("{}.{}", self.qualified_expr_path(base)?, field))
            }
            _ => None,
        }
    }

    pub(super) fn qualified_expr_prefers_constructor(&self, expr: &AstExpr) -> bool {
        self.qualified_expr_path(expr)
            .and_then(|path| path.split('.').next().map(str::to_owned))
            .and_then(|head| head.chars().next())
            .is_some_and(|ch| ch.is_ascii_uppercase())
    }

    pub(super) fn parse_type_arg_list(&mut self) -> Result<Vec<AstTypeRef>, String> {
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
        Ok(args)
    }

    #[rustfmt::skip]
    pub(super) fn parse_optional_type_annotation(&mut self) -> Result<Option<AstTypeRef>, String> { if self.peek_symbol(':') { self.expect_symbol(':')?; Ok(Some(self.parse_type_ref()?)) } else { Ok(None) } }
    #[rustfmt::skip]
    pub(super) fn parse_optional_return_type(&mut self) -> Result<Option<AstTypeRef>, String> { if self.peek_arrow() { self.expect_arrow()?; Ok(Some(self.parse_type_ref()?)) } else { Ok(None) } }
}
