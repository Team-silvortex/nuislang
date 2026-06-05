use super::*;
use nuis_semantics::model::{AstDestructureBinding, AstDestructureField};

impl Parser {
    pub(super) fn starts_destructure_let(&self) -> bool {
        matches!(self.tokens.get(self.cursor), Some(Token::Word(_)))
            && (matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol('{')))
                || matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol('<'))))
    }

    pub(super) fn parse_destructure_let_stmt(&mut self) -> Result<AstStmt, String> {
        let type_ref = self.parse_type_ref()?;
        let fields = self.parse_destructure_fields()?;
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

    fn parse_destructure_fields(&mut self) -> Result<Vec<AstDestructureField>, String> {
        self.expect_symbol('{')?;
        let mut fields = Vec::new();
        while !self.peek_symbol('}') {
            let field = self.expect_ident()?;
            if fields
                .iter()
                .any(|existing: &AstDestructureField| existing.field == field)
            {
                return Err(format!(
                    "duplicate field `{field}` in destructuring let pattern"
                ));
            }
            let binding = if self.peek_symbol(':') {
                self.expect_symbol(':')?;
                self.parse_destructure_binding()?
            } else {
                AstDestructureBinding::Bind(field.clone())
            };
            let mut bindings = Vec::new();
            collect_destructure_bindings(&binding, &mut bindings);
            for binding_name in bindings {
                if fields
                    .iter()
                    .any(|existing: &AstDestructureField| field_binds_name(existing, &binding_name))
                {
                    return Err(format!(
                        "duplicate binding `{binding_name}` in destructuring let pattern"
                    ));
                }
            }
            fields.push(AstDestructureField { field, binding });
            if !self.peek_symbol(',') {
                break;
            }
            self.expect_symbol(',')?;
        }
        self.expect_symbol('}')?;
        Ok(fields)
    }

    fn parse_destructure_binding(&mut self) -> Result<AstDestructureBinding, String> {
        if matches!(self.tokens.get(self.cursor), Some(Token::Word(word)) if word == "_") {
            self.cursor += 1;
            return Ok(AstDestructureBinding::Ignore);
        }
        if self.peek_symbol('{') {
            let fields = self.parse_destructure_fields()?;
            if fields.is_empty() {
                return Err(
                    "nested destructuring let pattern requires at least one field".to_owned(),
                );
            }
            return Ok(AstDestructureBinding::Nested {
                type_ref: None,
                fields,
            });
        }
        if self.starts_destructure_let() {
            let type_ref = self.parse_type_ref()?;
            let fields = self.parse_destructure_fields()?;
            if fields.is_empty() {
                return Err(
                    "nested destructuring let pattern requires at least one field".to_owned(),
                );
            }
            return Ok(AstDestructureBinding::Nested {
                type_ref: Some(type_ref),
                fields,
            });
        }
        match self.next() {
            Some(Token::Word(actual)) if actual != "true" && actual != "false" => {
                Ok(AstDestructureBinding::Bind(actual))
            }
            Some(other) => Err(format!(
                "expected identifier, `_`, or nested destructuring pattern, found {}",
                describe_token(&other)
            )),
            None => Err(
                "expected identifier, `_`, or nested destructuring pattern, found end of input"
                    .to_owned(),
            ),
        }
    }
}

fn collect_destructure_bindings(binding: &AstDestructureBinding, bindings: &mut Vec<String>) {
    match binding {
        AstDestructureBinding::Bind(name) => bindings.push(name.clone()),
        AstDestructureBinding::Ignore => {}
        AstDestructureBinding::Nested { fields, .. } => {
            for field in fields {
                collect_destructure_bindings(&field.binding, bindings);
            }
        }
    }
}

fn field_binds_name(field: &AstDestructureField, binding_name: &str) -> bool {
    let mut bindings = Vec::new();
    collect_destructure_bindings(&field.binding, &mut bindings);
    bindings
        .into_iter()
        .any(|existing| existing == binding_name)
}
