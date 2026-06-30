use super::*;

impl Parser {
    pub(super) fn parse_extern_abi(&mut self) -> Result<String, String> {
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

    pub(super) fn parse_extern_interface(
        &mut self,
        visibility: AstVisibility,
        abi: String,
    ) -> Result<AstExternInterface, String> {
        self.expect_word("interface")?;
        let name = self.expect_ident()?;
        self.expect_symbol('{')?;
        let mut methods = Vec::new();
        while !self.peek_symbol('}') {
            methods.push(self.parse_extern_function_with_abi(
                AstVisibility::Private,
                abi.clone(),
                Some(name.clone()),
            )?);
        }
        self.expect_symbol('}')?;
        Ok(AstExternInterface {
            visibility,
            abi,
            name,
            methods,
        })
    }

    pub(super) fn parse_extern_function_with_abi(
        &mut self,
        visibility: AstVisibility,
        abi: String,
        interface: Option<String>,
    ) -> Result<AstExternFunction, String> {
        let host_symbol = self.parse_extern_host_symbol()?;
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
            visibility,
            abi,
            interface,
            name,
            host_symbol,
            params,
            return_type,
        })
    }

    fn parse_extern_host_symbol(&mut self) -> Result<Option<String>, String> {
        let attributes = self.parse_attribute_list()?;
        if attributes.is_empty() {
            return Ok(None);
        }
        if attributes.len() != 1 || attributes[0].name != "host_symbol" {
            return Err(
                "extern declarations currently only support `@host_symbol(\"...\")` annotations"
                    .to_owned(),
            );
        }
        let attribute = &attributes[0];
        if attribute.args.len() != 1 {
            return Err(
                "extern declaration annotation `@host_symbol` expects exactly one string argument"
                    .to_owned(),
            );
        }
        let arg = &attribute.args[0];
        if arg.name.is_some() {
            return Err(
                "extern declaration annotation `@host_symbol` expects `@host_symbol(\"...\")`"
                    .to_owned(),
            );
        }
        match &arg.value {
            AstAttributeValue::String(value) if !value.is_empty() => Ok(Some(value.clone())),
            AstAttributeValue::String(_) => Err(
                "extern declaration annotation `@host_symbol(\"...\")` requires a non-empty host symbol"
                    .to_owned(),
            ),
            _ => Err(
                "extern declaration annotation `@host_symbol` expects a string literal"
                    .to_owned(),
            ),
        }
    }
}
