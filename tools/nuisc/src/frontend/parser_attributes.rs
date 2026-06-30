use super::*;

impl Parser {
    pub(super) fn peek_item_keyword_after_attributes(&self, keyword: &str) -> bool {
        let mut index = self.cursor;
        let mut saw_pub = false;
        loop {
            if matches!(self.tokens.get(index), Some(Token::DocComment(_))) {
                index += 1;
                continue;
            }
            if matches!(self.tokens.get(index), Some(Token::Word(word)) if word == "pub") {
                if saw_pub {
                    return false;
                }
                saw_pub = true;
                index += 1;
                continue;
            }
            if !matches!(self.tokens.get(index), Some(Token::Symbol('@'))) {
                break;
            }
            index += 1;
            if !matches!(self.tokens.get(index), Some(Token::Word(_))) {
                return false;
            }
            index += 1;
            if !matches!(self.tokens.get(index), Some(Token::Symbol('('))) {
                continue;
            }
            let mut depth = 0usize;
            while let Some(token) = self.tokens.get(index) {
                match token {
                    Token::Symbol('(') => depth += 1,
                    Token::Symbol(')') => {
                        depth -= 1;
                        if depth == 0 {
                            index += 1;
                            break;
                        }
                    }
                    _ => {}
                }
                index += 1;
            }
        }
        matches!(self.tokens.get(index), Some(Token::Word(word)) if word == keyword)
    }

    pub(super) fn parse_visibility_and_attribute_list(
        &mut self,
    ) -> Result<(AstVisibility, Vec<AstAttribute>), String> {
        let mut visibility = AstVisibility::Private;
        let mut attributes = self.parse_leading_attribute_list()?;
        loop {
            if self.peek_word("pub") {
                if visibility == AstVisibility::Public {
                    return Err("duplicate `pub` visibility modifier".to_owned());
                }
                self.expect_word("pub")?;
                visibility = AstVisibility::Public;
                attributes.extend(self.parse_leading_attribute_list()?);
                continue;
            }
            break;
        }
        Ok((visibility, attributes))
    }

    pub(super) fn parse_leading_attribute_list(&mut self) -> Result<Vec<AstAttribute>, String> {
        let mut attributes = Vec::new();
        loop {
            if self.peek_doc_comment() {
                attributes.push(self.parse_doc_comment_attribute()?);
                continue;
            }
            if self.peek_symbol('@') {
                attributes.push(self.parse_attribute()?);
                continue;
            }
            break;
        }
        Ok(attributes)
    }

    pub(super) fn parse_attribute_list(&mut self) -> Result<Vec<AstAttribute>, String> {
        let mut attributes = Vec::new();
        while self.peek_symbol('@') {
            attributes.push(self.parse_attribute()?);
        }
        Ok(attributes)
    }

    pub(super) fn parse_doc_comment_attribute(&mut self) -> Result<AstAttribute, String> {
        match self.tokens.get(self.cursor) {
            Some(Token::DocComment(text)) => {
                let text = text.clone();
                self.cursor += 1;
                Ok(AstAttribute {
                    name: "doc".to_owned(),
                    args: vec![AstAttributeArg {
                        name: None,
                        value: AstAttributeValue::String(text),
                    }],
                })
            }
            Some(token) => Err(format!(
                "expected doc comment, found {}",
                describe_token(token)
            )),
            None => Err("expected doc comment, found end of file".to_owned()),
        }
    }

    fn parse_attribute(&mut self) -> Result<AstAttribute, String> {
        self.expect_symbol('@')?;
        let name = self.expect_ident()?;
        let args = if self.peek_symbol('(') {
            self.parse_attribute_arg_list()?
        } else {
            Vec::new()
        };
        Ok(AstAttribute { name, args })
    }

    fn parse_attribute_arg_list(&mut self) -> Result<Vec<AstAttributeArg>, String> {
        self.expect_symbol('(')?;
        let mut args = Vec::new();
        if self.peek_symbol(')') {
            self.expect_symbol(')')?;
            return Ok(args);
        }
        loop {
            let (name, value) = if let Some(Token::Word(word)) = self.tokens.get(self.cursor) {
                if matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol('='))) {
                    let label = word.clone();
                    self.cursor += 1;
                    self.expect_symbol('=')?;
                    (Some(label), self.parse_attribute_value()?)
                } else {
                    (None, self.parse_attribute_value()?)
                }
            } else {
                (None, self.parse_attribute_value()?)
            };
            args.push(AstAttributeArg { name, value });
            if self.peek_symbol(',') {
                self.expect_symbol(',')?;
            } else {
                break;
            }
        }
        self.expect_symbol(')')?;
        Ok(args)
    }

    fn parse_attribute_value(&mut self) -> Result<AstAttributeValue, String> {
        match self.tokens.get(self.cursor) {
            Some(Token::Word(word)) if word == "true" => {
                self.cursor += 1;
                Ok(AstAttributeValue::Bool(true))
            }
            Some(Token::Word(word)) if word == "false" => {
                self.cursor += 1;
                Ok(AstAttributeValue::Bool(false))
            }
            Some(Token::Word(word)) => {
                let ident = word.clone();
                self.cursor += 1;
                Ok(AstAttributeValue::Ident(ident))
            }
            Some(Token::Integer(value)) => {
                let value = *value;
                self.cursor += 1;
                Ok(AstAttributeValue::Int(value))
            }
            Some(Token::String(value)) => {
                let value = value.clone();
                self.cursor += 1;
                Ok(AstAttributeValue::String(value))
            }
            Some(token) => Err(format!(
                "expected annotation attribute value, found {}",
                describe_token(token)
            )),
            None => Err("expected annotation attribute value, found end of file".to_owned()),
        }
    }

    pub(super) fn ensure_doc_only_attributes(
        &self,
        context: &str,
        attributes: &[AstAttribute],
    ) -> Result<(), String> {
        for attribute in attributes {
            if attribute.name != "doc" {
                return Err(format!(
                    "{context} currently only supports doc comments or `@doc(...)` annotations"
                ));
            }
        }
        Ok(())
    }

    pub(super) fn consume_module_leading_doc_comments(
        &mut self,
    ) -> Result<Vec<AstAttribute>, String> {
        let mut attributes = Vec::new();
        while self.peek_doc_comment() {
            attributes.push(self.parse_doc_comment_attribute()?);
        }
        Ok(attributes)
    }
}
