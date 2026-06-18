use nuis_semantics::model::{
    AstAttribute, AstAttributeArg, AstAttributeValue, AstBinaryOp, AstConstItem, AstEnumDef,
    AstEnumVariant, AstEnumVariantKind, AstExpr, AstExternFunction, AstExternInterface,
    AstFunction, AstGenericParam, AstImplDef, AstImplMethod, AstMatchArm, AstMatchPattern,
    AstModule, AstParam, AstStmt, AstStructDef, AstStructField, AstTraitDef, AstTraitMethodSig,
    AstTypeAlias, AstTypeRef, AstVisibility, AstWherePredicate, TestClockDomain, TestClockPolicy,
};

use super::lexer::{describe_token, Token};

#[path = "parser_destructure.rs"]
mod parser_destructure;
#[path = "parser_match_patterns.rs"]
mod parser_match_patterns;

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
    allow_struct_literals: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssignmentOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            cursor: 0,
            allow_struct_literals: true,
        }
    }

    pub fn parse_module(&mut self) -> Result<AstModule, String> {
        let mut uses = Vec::new();
        let mut externs = Vec::new();
        let mut extern_interfaces = Vec::new();
        while self.peek_word("use") {
            uses.push(self.parse_use_decl()?);
        }
        while self.peek_item_keyword_after_attributes("extern") {
            let (visibility, attributes) = self.parse_visibility_and_attribute_list()?;
            if !attributes.is_empty() {
                return Err(
                    "module-level extern declarations currently only support `pub` before `extern`"
                        .to_owned(),
                );
            }
            let abi = self.parse_extern_abi()?;
            if self.peek_word("interface") {
                extern_interfaces.push(self.parse_extern_interface(visibility, abi)?);
            } else {
                externs.push(self.parse_extern_function_with_abi(visibility, abi, None)?);
            }
        }
        self.expect_word("mod")?;
        let domain = self.expect_ident()?;
        let unit = self.expect_ident()?;
        self.expect_symbol('{')?;

        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let mut traits = Vec::new();
        let mut impls = Vec::new();
        let mut consts = Vec::new();
        let mut type_aliases = Vec::new();
        let mut functions = Vec::new();
        while !self.peek_symbol('}') {
            if self.peek_word("mod") {
                return Err("nested mod definitions are not allowed".to_owned());
            }
            if self.peek_item_keyword_after_attributes("extern") {
                let (visibility, attributes) = self.parse_visibility_and_attribute_list()?;
                if !attributes.is_empty() {
                    return Err(
                        "module-level extern declarations currently only support `pub` before `extern`"
                            .to_owned(),
                    );
                }
                let abi = self.parse_extern_abi()?;
                if self.peek_word("interface") {
                    extern_interfaces.push(self.parse_extern_interface(visibility, abi)?);
                } else {
                    externs.push(self.parse_extern_function_with_abi(visibility, abi, None)?);
                }
            } else if self.peek_item_keyword_after_attributes("struct") {
                structs.push(self.parse_struct_def()?);
            } else if self.peek_item_keyword_after_attributes("enum") {
                enums.push(self.parse_enum_def()?);
            } else if self.peek_item_keyword_after_attributes("trait") {
                traits.push(self.parse_trait_def()?);
            } else if self.peek_item_keyword_after_attributes("impl") {
                impls.push(self.parse_impl_def()?);
            } else if self.peek_item_keyword_after_attributes("const") {
                consts.push(self.parse_const_item()?);
            } else if self.peek_item_keyword_after_attributes("type") {
                type_aliases.push(self.parse_type_alias_item()?);
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
            consts,
            type_aliases,
            structs,
            enums,
            traits,
            impls,
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

    fn peek_item_keyword_after_attributes(&self, keyword: &str) -> bool {
        let mut index = self.cursor;
        let mut saw_pub = false;
        loop {
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

    fn parse_visibility_and_attribute_list(
        &mut self,
    ) -> Result<(AstVisibility, Vec<AstAttribute>), String> {
        let mut visibility = AstVisibility::Private;
        let mut attributes = Vec::new();
        loop {
            if self.peek_word("pub") {
                if visibility == AstVisibility::Public {
                    return Err("duplicate `pub` visibility modifier".to_owned());
                }
                self.expect_word("pub")?;
                visibility = AstVisibility::Public;
                continue;
            }
            if self.peek_symbol('@') {
                attributes.push(self.parse_attribute()?);
                continue;
            }
            break;
        }
        Ok((visibility, attributes))
    }

    fn parse_struct_def(&mut self) -> Result<AstStructDef, String> {
        let (visibility, attributes) = self.parse_visibility_and_attribute_list()?;
        self.expect_word("struct")?;
        let name = self.expect_ident()?;
        let generic_params = if self.peek_symbol('<') {
            self.parse_generic_param_decl_list()?
        } else {
            Vec::new()
        };
        let where_bounds = if self.peek_word("where") {
            self.parse_where_predicate_list()?
        } else {
            Vec::new()
        };
        self.expect_symbol('{')?;
        let mut fields = Vec::new();
        while !self.peek_symbol('}') {
            let (field_visibility, field_attributes) =
                self.parse_visibility_and_attribute_list()?;
            let field_name = self.expect_ident()?;
            self.expect_symbol(':')?;
            let ty = self.parse_type_ref()?;
            fields.push(AstStructField {
                visibility: field_visibility,
                attributes: field_attributes,
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
        Ok(AstStructDef {
            visibility,
            attributes,
            name,
            generic_params,
            where_bounds,
            fields,
        })
    }

    fn parse_const_item(&mut self) -> Result<AstConstItem, String> {
        let (visibility, attributes) = self.parse_visibility_and_attribute_list()?;
        if !attributes.is_empty() {
            return Err(
                "top-level const annotations are not supported in the current frontend".to_owned(),
            );
        }
        self.expect_word("const")?;
        let name = self.expect_ident()?;
        let ty = self.parse_optional_type_annotation()?;
        self.expect_symbol('=')?;
        let value = self.parse_expr()?;
        self.expect_symbol(';')?;
        Ok(AstConstItem {
            visibility,
            name,
            ty,
            value,
        })
    }

    fn parse_enum_def(&mut self) -> Result<AstEnumDef, String> {
        let (visibility, attributes) = self.parse_visibility_and_attribute_list()?;
        self.expect_word("enum")?;
        let name = self.expect_ident()?;
        let generic_params = if self.peek_symbol('<') {
            self.parse_generic_param_decl_list()?
        } else {
            Vec::new()
        };
        let where_bounds = if self.peek_word("where") {
            self.parse_where_predicate_list()?
        } else {
            Vec::new()
        };
        self.expect_symbol('{')?;
        let mut variants = Vec::new();
        while !self.peek_symbol('}') {
            let variant_name = self.expect_ident()?;
            let kind = if self.peek_symbol('(') {
                self.expect_symbol('(')?;
                let mut fields = Vec::new();
                while !self.peek_symbol(')') {
                    fields.push(self.parse_type_ref()?);
                    if self.peek_symbol(',') {
                        self.expect_symbol(',')?;
                    } else {
                        break;
                    }
                }
                self.expect_symbol(')')?;
                AstEnumVariantKind::Tuple(fields)
            } else if self.peek_symbol('{') {
                self.expect_symbol('{')?;
                let mut fields = Vec::new();
                while !self.peek_symbol('}') {
                    let (field_visibility, field_attributes) =
                        self.parse_visibility_and_attribute_list()?;
                    let field_name = self.expect_ident()?;
                    self.expect_symbol(':')?;
                    let ty = self.parse_type_ref()?;
                    fields.push(AstStructField {
                        visibility: field_visibility,
                        attributes: field_attributes,
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
                AstEnumVariantKind::Struct(fields)
            } else {
                AstEnumVariantKind::Unit
            };
            variants.push(AstEnumVariant {
                name: variant_name,
                kind,
            });
            if self.peek_symbol(',') {
                self.expect_symbol(',')?;
            } else {
                break;
            }
        }
        self.expect_symbol('}')?;
        Ok(AstEnumDef {
            visibility,
            attributes,
            name,
            generic_params,
            where_bounds,
            variants,
        })
    }

    fn parse_type_alias_item(&mut self) -> Result<AstTypeAlias, String> {
        let (visibility, attributes) = self.parse_visibility_and_attribute_list()?;
        if !attributes.is_empty() {
            return Err(
                "top-level type alias annotations are not supported in the current frontend"
                    .to_owned(),
            );
        }
        self.expect_word("type")?;
        let name = self.expect_ident()?;
        let generic_params = if self.peek_symbol('<') {
            self.parse_generic_param_decl_list()?
        } else {
            Vec::new()
        };
        let where_bounds = if self.peek_word("where") {
            self.parse_where_predicate_list()?
        } else {
            Vec::new()
        };
        self.expect_symbol('=')?;
        let target = self.parse_type_ref()?;
        self.expect_symbol(';')?;
        Ok(AstTypeAlias {
            visibility,
            name,
            generic_params,
            where_bounds,
            target,
        })
    }

    fn parse_trait_def(&mut self) -> Result<AstTraitDef, String> {
        let (visibility, attributes) = self.parse_visibility_and_attribute_list()?;
        if !attributes.is_empty() {
            return Err("trait annotations are not supported in the current frontend".to_owned());
        }
        self.expect_word("trait")?;
        let name = self.expect_ident()?;
        self.expect_symbol('{')?;
        let mut methods = Vec::new();
        while !self.peek_symbol('}') {
            methods.push(self.parse_trait_method_sig()?);
        }
        self.expect_symbol('}')?;
        Ok(AstTraitDef {
            visibility,
            name,
            methods,
        })
    }

    fn parse_trait_method_sig(&mut self) -> Result<AstTraitMethodSig, String> {
        let (visibility, attributes) = self.parse_visibility_and_attribute_list()?;
        if !matches!(visibility, AstVisibility::Private) {
            return Err(
                "trait methods do not support independent `pub` visibility in the current frontend"
                    .to_owned(),
            );
        }
        if !attributes.is_empty() {
            return Err(
                "trait method annotations are not supported in the current frontend".to_owned(),
            );
        }
        self.expect_word("fn")?;
        let name = self.expect_ident()?;
        self.expect_symbol('(')?;
        let params = if self.peek_symbol(')') {
            Vec::new()
        } else {
            self.parse_param_list()?
        };
        self.expect_symbol(')')?;
        let return_type = self.parse_optional_return_type()?;
        let default_body = if self.peek_symbol('{') {
            Some(self.parse_block_with_tail_expr()?)
        } else {
            self.expect_symbol(';')?;
            None
        };
        Ok(AstTraitMethodSig {
            name,
            params,
            return_type,
            default_body,
        })
    }

    fn parse_impl_def(&mut self) -> Result<AstImplDef, String> {
        self.expect_word("impl")?;
        let generic_params = if self.peek_symbol('<') {
            self.parse_generic_param_decl_list()?
        } else {
            Vec::new()
        };
        let trait_name = self.parse_qualified_ident()?;
        self.expect_word("for")?;
        let for_type = self.parse_type_ref()?;
        let where_bounds = if self.peek_word("where") {
            self.parse_where_predicate_list()?
        } else {
            Vec::new()
        };
        self.expect_symbol('{')?;
        let mut methods = Vec::new();
        while !self.peek_symbol('}') {
            methods.push(self.parse_impl_method()?);
        }
        self.expect_symbol('}')?;
        Ok(AstImplDef {
            generic_params,
            where_bounds,
            trait_name,
            for_type,
            methods,
        })
    }

    fn parse_impl_method(&mut self) -> Result<AstImplMethod, String> {
        let (visibility, attributes) = self.parse_visibility_and_attribute_list()?;
        if !matches!(visibility, AstVisibility::Private) {
            return Err(
                "impl methods do not support independent `pub` visibility in the current frontend"
                    .to_owned(),
            );
        }
        if !attributes.is_empty() {
            return Err(
                "impl method annotations are not supported in the current frontend".to_owned(),
            );
        }
        self.expect_word("fn")?;
        let name = self.expect_ident()?;
        self.expect_symbol('(')?;
        let params = if self.peek_symbol(')') {
            Vec::new()
        } else {
            self.parse_param_list()?
        };
        self.expect_symbol(')')?;
        let return_type = self.parse_optional_return_type()?;
        let body = self.parse_block_with_tail_expr()?;
        Ok(AstImplMethod {
            name,
            params,
            return_type,
            body,
        })
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

    fn parse_extern_interface(
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

    fn parse_extern_function_with_abi(
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

    fn parse_function(&mut self) -> Result<AstFunction, String> {
        let (visibility, mut attributes) = self.parse_visibility_and_attribute_list()?;
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
        let generic_params = if self.peek_symbol('<') {
            self.parse_generic_param_decl_list()?
        } else {
            Vec::new()
        };
        let test_from_attribute = attributes
            .iter()
            .find(|attribute| attribute.name == "test")
            .map(|attribute| self.parse_test_attribute_metadata(&attribute.args))
            .transpose()?;
        if declared_test_name.is_some() && test_from_attribute.is_some() {
            return Err(format!(
                "function `{name}` cannot use both `test(...)` and `@test(...)`; choose one test declaration style"
            ));
        }
        if declared_test_name.is_some() {
            attributes.push(self.build_test_attribute(
                declared_test_name.clone(),
                test_ignored,
                test_should_fail,
                test_reason.clone(),
                test_timeout_ms,
                test_clock_domain,
                test_clock_policy,
            ));
        }
        let (
            raw_test_name,
            test_ignored,
            test_should_fail,
            test_reason,
            test_timeout_ms,
            test_clock_domain,
            test_clock_policy,
        ) = match test_from_attribute {
            Some(values) => values,
            None => (
                declared_test_name,
                test_ignored,
                test_should_fail,
                test_reason,
                test_timeout_ms,
                test_clock_domain,
                test_clock_policy,
            ),
        };
        let test_name = raw_test_name.map(|label| {
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
        let return_type = self.parse_optional_return_type()?;
        let where_bounds = if self.peek_word("where") {
            self.parse_where_predicate_list()?
        } else {
            Vec::new()
        };
        let body = self.parse_block_with_tail_expr()?;

        Ok(AstFunction {
            visibility,
            name,
            attributes,
            test_name,
            test_ignored,
            test_should_fail,
            test_reason,
            test_timeout_ms,
            test_clock_domain,
            test_clock_policy,
            is_async,
            generic_params,
            where_bounds,
            params,
            return_type,
            body,
        })
    }

    fn build_test_attribute(
        &self,
        label: Option<String>,
        ignored: bool,
        should_fail: bool,
        reason: Option<String>,
        timeout_ms: Option<i64>,
        clock_domain: Option<TestClockDomain>,
        clock_policy: Option<TestClockPolicy>,
    ) -> AstAttribute {
        let mut args = Vec::new();
        if let Some(label) = label {
            if !label.is_empty() {
                args.push(AstAttributeArg {
                    name: None,
                    value: AstAttributeValue::String(label),
                });
            }
        }
        if ignored {
            args.push(AstAttributeArg {
                name: Some("ignored".to_owned()),
                value: AstAttributeValue::Bool(true),
            });
        }
        if should_fail {
            args.push(AstAttributeArg {
                name: Some("should_fail".to_owned()),
                value: AstAttributeValue::Bool(true),
            });
        }
        if let Some(reason) = reason {
            args.push(AstAttributeArg {
                name: Some("reason".to_owned()),
                value: AstAttributeValue::String(reason),
            });
        }
        if let Some(timeout_ms) = timeout_ms {
            args.push(AstAttributeArg {
                name: Some("timeout_ms".to_owned()),
                value: AstAttributeValue::Int(timeout_ms),
            });
        }
        if let Some(clock_domain) = clock_domain {
            args.push(AstAttributeArg {
                name: Some("clock_domain".to_owned()),
                value: AstAttributeValue::String(clock_domain.as_str().to_owned()),
            });
        }
        if let Some(clock_policy) = clock_policy {
            args.push(AstAttributeArg {
                name: Some("clock_policy".to_owned()),
                value: AstAttributeValue::String(clock_policy.as_str().to_owned()),
            });
        }
        AstAttribute {
            name: "test".to_owned(),
            args,
        }
    }

    fn parse_test_attribute_metadata(
        &self,
        args: &[AstAttributeArg],
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
        let mut label = Some(String::new());
        let mut ignored = false;
        let mut should_fail = false;
        let mut reason = None;
        let mut timeout_ms = None;
        let mut clock_domain = None;
        let mut clock_policy = None;

        for arg in args {
            match (&arg.name, &arg.value) {
                (None, AstAttributeValue::String(value)) => {
                    if label.is_some() && label.as_ref().is_some_and(|item| item.is_empty()) {
                        label = Some(value.clone());
                    } else {
                        return Err(
                            "`@test(...)` accepts at most one positional string label".to_owned()
                        );
                    }
                }
                (None, AstAttributeValue::Ident(flag)) => match flag.as_str() {
                    "ignored" => ignored = true,
                    "should_fail" => should_fail = true,
                    other => {
                        return Err(format!(
                            "unknown positional test flag `{other}` in `@test(...)`"
                        ));
                    }
                },
                (Some(name), AstAttributeValue::Bool(value)) => match name.as_str() {
                    "ignored" => ignored = *value,
                    "should_fail" => should_fail = *value,
                    other => {
                        return Err(format!(
                            "unknown test metadata key `{other}` in `@test(...)`"
                        ));
                    }
                },
                (Some(name), AstAttributeValue::String(value)) => match name.as_str() {
                    "reason" => reason = Some(value.clone()),
                    "clock_domain" => clock_domain = Some(TestClockDomain::parse(value).ok_or_else(|| format!(
                        "unsupported `clock_domain=\"{}\"`; expected `monotonic`, `wall`, or `global`",
                        value
                    ))?),
                    "clock_policy" => clock_policy = Some(TestClockPolicy::parse(value).ok_or_else(|| format!(
                        "unsupported `clock_policy=\"{}\"`; expected `bridge`",
                        value
                    ))?),
                    "name" => label = Some(value.clone()),
                    other => {
                        return Err(format!(
                            "unknown test metadata key `{other}` in `@test(...)`"
                        ));
                    }
                },
                (Some(name), AstAttributeValue::Int(value)) => match name.as_str() {
                    "timeout_ms" => timeout_ms = Some(*value),
                    other => {
                        return Err(format!(
                            "unknown test metadata key `{other}` in `@test(...)`"
                        ));
                    }
                },
                (Some(name), _) => {
                    return Err(format!(
                        "test metadata key `{name}` has unsupported value shape in `@test(...)`"
                    ));
                }
                (None, _) => {
                    return Err(
                        "expected string label or bare flag in `@test(...)` positional arguments"
                            .to_owned(),
                    );
                }
            }
        }

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

    fn parse_attribute_list(&mut self) -> Result<Vec<AstAttribute>, String> {
        let mut attributes = Vec::new();
        while self.peek_symbol('@') {
            attributes.push(self.parse_attribute()?);
        }
        Ok(attributes)
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

    fn parse_generic_param_decl_list(&mut self) -> Result<Vec<AstGenericParam>, String> {
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

    fn parse_where_predicate_list(&mut self) -> Result<Vec<AstWherePredicate>, String> {
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

    fn parse_qualified_ident(&mut self) -> Result<String, String> {
        let mut name = self.expect_ident()?;
        while self.peek_symbol('.') {
            self.expect_symbol('.')?;
            name.push('.');
            name.push_str(&self.expect_ident()?);
        }
        Ok(name)
    }

    fn qualified_expr_path(&self, expr: &AstExpr) -> Option<String> {
        match expr {
            AstExpr::Var(name) => Some(name.clone()),
            AstExpr::FieldAccess { base, field } => {
                Some(format!("{}.{}", self.qualified_expr_path(base)?, field))
            }
            _ => None,
        }
    }

    fn qualified_expr_prefers_constructor(&self, expr: &AstExpr) -> bool {
        self.qualified_expr_path(expr)
            .and_then(|path| path.split('.').next().map(str::to_owned))
            .and_then(|head| head.chars().next())
            .is_some_and(|ch| ch.is_ascii_uppercase())
    }

    fn parse_type_arg_list(&mut self) -> Result<Vec<AstTypeRef>, String> {
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
    fn parse_optional_type_annotation(&mut self) -> Result<Option<AstTypeRef>, String> { if self.peek_symbol(':') { self.expect_symbol(':')?; Ok(Some(self.parse_type_ref()?)) } else { Ok(None) } }
    #[rustfmt::skip]
    fn parse_optional_return_type(&mut self) -> Result<Option<AstTypeRef>, String> { if self.peek_arrow() { self.expect_arrow()?; Ok(Some(self.parse_type_ref()?)) } else { Ok(None) } }

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

    fn try_parse_assignment_stmt(&mut self) -> Result<Option<AstStmt>, String> {
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

    fn parse_match_expr_parts(
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

    fn parse_match_scrutinee_expr(&mut self) -> Result<AstExpr, String> {
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

    fn parse_expr(&mut self) -> Result<AstExpr, String> {
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

    fn parse_postfix(&mut self) -> Result<AstExpr, String> {
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

    fn try_parse_expr_type_arg_list_followed_by_call_or_literal(
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

    fn try_parse_expr_type_arg_list_followed_by_call_only(
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

    fn parse_lambda_param_list(&mut self) -> Result<Vec<AstParam>, String> {
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

    fn parse_block_with_tail_expr(&mut self) -> Result<Vec<AstStmt>, String> {
        self.expect_symbol('{')?;
        let body = self.parse_block_body(true)?;
        self.expect_symbol('}')?;
        Ok(body)
    }

    fn parse_stmt_block(&mut self) -> Result<Vec<AstStmt>, String> {
        self.expect_symbol('{')?;
        let body = self.parse_block_body(false)?;
        self.expect_symbol('}')?;
        Ok(body)
    }

    fn parse_block_body(&mut self, allow_tail_expr_stmt: bool) -> Result<Vec<AstStmt>, String> {
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

    fn can_start_tail_expr_stmt(&self) -> bool {
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
                if self.peek_symbol('}') {
                    break;
                }
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

    fn peek_assignment_op(&self) -> Option<AssignmentOp> {
        if self.peek_symbol('+')
            && matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol('=')))
        {
            Some(AssignmentOp::AddAssign)
        } else if self.peek_symbol('-')
            && matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol('=')))
        {
            Some(AssignmentOp::SubAssign)
        } else if self.peek_symbol('*')
            && matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol('=')))
        {
            Some(AssignmentOp::MulAssign)
        } else if self.peek_symbol('/')
            && matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol('=')))
        {
            Some(AssignmentOp::DivAssign)
        } else if self.peek_symbol('%')
            && matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol('=')))
        {
            Some(AssignmentOp::RemAssign)
        } else if self.peek_symbol('=')
            && !matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol('=')))
        {
            Some(AssignmentOp::Assign)
        } else {
            None
        }
    }

    fn consume_assignment_op(&mut self, op: AssignmentOp) -> Result<(), String> {
        match op {
            AssignmentOp::Assign => self.expect_symbol('='),
            AssignmentOp::AddAssign => {
                self.expect_symbol('+')?;
                self.expect_symbol('=')
            }
            AssignmentOp::SubAssign => {
                self.expect_symbol('-')?;
                self.expect_symbol('=')
            }
            AssignmentOp::MulAssign => {
                self.expect_symbol('*')?;
                self.expect_symbol('=')
            }
            AssignmentOp::DivAssign => {
                self.expect_symbol('/')?;
                self.expect_symbol('=')
            }
            AssignmentOp::RemAssign => {
                self.expect_symbol('%')?;
                self.expect_symbol('=')
            }
        }
    }

    fn peek_symbol_pair(&self, first: char, second: char) -> bool {
        matches!(self.tokens.get(self.cursor), Some(Token::Symbol(actual)) if *actual == first)
            && matches!(self.tokens.get(self.cursor + 1), Some(Token::Symbol(actual)) if *actual == second)
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
