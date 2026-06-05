use super::*;

impl Parser {
    pub(super) fn parse_condition_expr(&mut self) -> Result<AstExpr, String> {
        let old = self.allow_struct_literals;
        self.allow_struct_literals = false;
        let parsed = self.parse_expr();
        self.allow_struct_literals = old;
        parsed
    }

    pub(super) fn parse_match_pattern(&mut self) -> Result<AstMatchPattern, String> {
        let mut patterns = vec![self.parse_single_match_pattern()?];
        while self.peek_symbol('|') && !self.peek_symbol_pair('|', '|') {
            self.expect_symbol('|')?;
            patterns.push(self.parse_single_match_pattern()?);
        }
        if patterns.len() == 1 {
            return Ok(patterns.pop().expect("pattern exists"));
        }
        if patterns
            .iter()
            .any(|pattern| matches!(pattern, AstMatchPattern::Wildcard))
        {
            return Err(
                "minimal `match` does not allow `_` inside multi-pattern arms; use a final standalone `_ => ...` arm"
                    .to_owned(),
            );
        }
        Ok(AstMatchPattern::Or(patterns))
    }

    fn parse_payload_struct_match_pattern(
        &mut self,
        type_ref: AstTypeRef,
    ) -> Result<AstMatchPattern, String> {
        self.expect_symbol('(')?;
        if self.peek_symbol(')') {
            return Err(format!(
                "payload-style struct match pattern `{}` requires exactly one inner pattern",
                type_ref.name
            ));
        }
        let payload = self.parse_struct_field_match_pattern()?;
        self.expect_symbol(')')?;
        Ok(AstMatchPattern::PayloadStruct {
            type_ref,
            payload: Box::new(payload),
        })
    }

    fn parse_struct_field_match_pattern(&mut self) -> Result<AstMatchPattern, String> {
        let mut patterns = vec![self.parse_single_struct_field_match_pattern()?];
        while self.peek_symbol('|') && !self.peek_symbol_pair('|', '|') {
            self.expect_symbol('|')?;
            patterns.push(self.parse_single_struct_field_match_pattern()?);
        }
        if patterns.len() == 1 {
            return Ok(patterns.pop().expect("pattern exists"));
        }
        if patterns.iter().any(|pattern| {
            matches!(
                pattern,
                AstMatchPattern::Wildcard | AstMatchPattern::Bind(_)
            )
        }) {
            return Err(
                "minimal struct field match patterns do not allow `_` or bindings inside `|` multi-pattern arms; use a standalone binding arm or move the extra condition into a guard"
                    .to_owned(),
            );
        }
        Ok(AstMatchPattern::Or(patterns))
    }

    fn parse_single_match_pattern(&mut self) -> Result<AstMatchPattern, String> {
        match self.next() {
            Some(Token::Word(word)) if word == "_" => Ok(AstMatchPattern::Wildcard),
            Some(Token::Word(word)) if word == "true" => Ok(AstMatchPattern::Bool(true)),
            Some(Token::Word(word)) if word == "false" => Ok(AstMatchPattern::Bool(false)),
            Some(Token::Word(_word))
                if self.peek_symbol('{') || self.peek_symbol('<') || self.peek_symbol('(') =>
            {
                self.cursor = self.cursor.saturating_sub(1);
                let type_ref = self.parse_type_ref()?;
                if self.peek_symbol('{') {
                    self.parse_struct_match_pattern_with_fields(Some(type_ref))
                } else if self.peek_symbol('(') {
                    self.parse_payload_struct_match_pattern(type_ref)
                } else {
                    Err("expected `{` or `(` after struct match pattern type".to_owned())
                }
            }
            Some(Token::Integer(value)) => {
                if self.peek_symbol('.')
                    && matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol('.')))
                {
                    self.expect_symbol('.')?;
                    self.expect_symbol('.')?;
                    self.expect_symbol('=')?;
                    let end = match self.next() {
                        Some(Token::Integer(value)) => value,
                        Some(other) => {
                            return Err(format!(
                                "expected integer literal after `..=` in match range pattern, found {}",
                                describe_token(&other)
                            ))
                        }
                        None => {
                            return Err(
                                "unexpected end of input after `..=` in match range pattern"
                                    .to_owned(),
                            )
                        }
                    };
                    if value > end {
                        return Err(format!(
                            "inclusive match range `{value}..={end}` must have start <= end"
                        ));
                    }
                    Ok(AstMatchPattern::IntRangeInclusive(value, end))
                } else {
                    Ok(AstMatchPattern::Int(value))
                }
            }
            Some(other) => Err(format!(
                "expected `true`, `false`, integer literal, integer range, struct field pattern, or `_` in match arm pattern, found {}",
                describe_token(&other)
            )),
            None => Err("unexpected end of input in match arm pattern".to_owned()),
        }
    }

    fn parse_single_struct_field_match_pattern(&mut self) -> Result<AstMatchPattern, String> {
        match self.next() {
            Some(Token::Word(word)) if word == "_" => Ok(AstMatchPattern::Wildcard),
            Some(Token::Word(word)) if word == "true" => Ok(AstMatchPattern::Bool(true)),
            Some(Token::Word(word)) if word == "false" => Ok(AstMatchPattern::Bool(false)),
            Some(Token::Symbol('{')) => {
                self.cursor = self.cursor.saturating_sub(1);
                self.parse_struct_match_pattern_with_fields(None)
            }
            Some(Token::Word(_word))
                if self.peek_symbol('{') || self.peek_symbol('<') || self.peek_symbol('(') =>
            {
                self.cursor = self.cursor.saturating_sub(1);
                let type_ref = self.parse_type_ref()?;
                if self.peek_symbol('{') {
                    self.parse_struct_match_pattern_with_fields(Some(type_ref))
                } else if self.peek_symbol('(') {
                    self.parse_payload_struct_match_pattern(type_ref)
                } else {
                    Err("expected `{` or `(` after nested struct match pattern type".to_owned())
                }
            }
            Some(Token::Word(word)) => Ok(AstMatchPattern::Bind(word)),
            Some(Token::Integer(_value)) => {
                self.cursor = self.cursor.saturating_sub(1);
                self.parse_single_match_pattern()
            }
            Some(other) => Err(format!(
                "expected field binding, literal pattern, nested struct pattern, or `_` in struct match field, found {}",
                describe_token(&other)
            )),
            None => Err("unexpected end of input in struct match field pattern".to_owned()),
        }
    }

    fn parse_struct_match_pattern_with_fields(
        &mut self,
        type_ref: Option<AstTypeRef>,
    ) -> Result<AstMatchPattern, String> {
        if !self.peek_symbol('{') {
            return Err("expected `{` after struct match pattern type".to_owned());
        }
        self.expect_symbol('{')?;
        let mut fields = Vec::new();
        while !self.peek_symbol('}') {
            let field_name = match self.next() {
                Some(Token::Word(name)) => name,
                Some(Token::Symbol('}')) => {
                    self.cursor = self.cursor.saturating_sub(1);
                    break;
                }
                Some(other) => {
                    return Err(format!(
                        "expected field name in struct match pattern, found {}",
                        describe_token(&other)
                    ))
                }
                None => {
                    return Err(
                        "unexpected end of input in struct match pattern field list".to_owned()
                    )
                }
            };
            self.expect_symbol(':')?;
            let pattern = self.parse_struct_field_match_pattern()?;
            fields.push((field_name, pattern));
            if self.peek_symbol(',') {
                self.expect_symbol(',')?;
            }
        }
        self.expect_symbol('}')?;
        Ok(AstMatchPattern::StructFields { type_ref, fields })
    }
}
