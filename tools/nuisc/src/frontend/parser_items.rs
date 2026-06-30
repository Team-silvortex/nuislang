use super::*;

impl Parser {
    pub(super) fn parse_struct_def(&mut self) -> Result<AstStructDef, String> {
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

    pub(super) fn parse_const_item(&mut self) -> Result<AstConstItem, String> {
        let (visibility, attributes) = self.parse_visibility_and_attribute_list()?;
        self.expect_word("const")?;
        let name = self.expect_ident()?;
        self.ensure_doc_only_attributes("top-level const", &attributes)?;
        let ty = self.parse_optional_type_annotation()?;
        self.expect_symbol('=')?;
        let value = self.parse_expr()?;
        self.expect_symbol(';')?;
        Ok(AstConstItem {
            visibility,
            attributes,
            name,
            ty,
            value,
        })
    }

    pub(super) fn parse_enum_def(&mut self) -> Result<AstEnumDef, String> {
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
            let attributes = self.parse_leading_attribute_list()?;
            let variant_name = self.expect_ident()?;
            self.ensure_doc_only_attributes("enum variant", &attributes)?;
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
                attributes,
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

    pub(super) fn parse_type_alias_item(&mut self) -> Result<AstTypeAlias, String> {
        let (visibility, attributes) = self.parse_visibility_and_attribute_list()?;
        self.expect_word("type")?;
        let name = self.expect_ident()?;
        self.ensure_doc_only_attributes("top-level type alias", &attributes)?;
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
            attributes,
            name,
            generic_params,
            where_bounds,
            target,
        })
    }

    pub(super) fn parse_trait_def(&mut self) -> Result<AstTraitDef, String> {
        let (visibility, attributes) = self.parse_visibility_and_attribute_list()?;
        self.expect_word("trait")?;
        let name = self.expect_ident()?;
        self.ensure_doc_only_attributes("trait", &attributes)?;
        self.expect_symbol('{')?;
        let mut methods = Vec::new();
        while !self.peek_symbol('}') {
            methods.push(self.parse_trait_method_sig()?);
        }
        self.expect_symbol('}')?;
        Ok(AstTraitDef {
            visibility,
            attributes,
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
        self.ensure_doc_only_attributes("trait method", &attributes)?;
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
            attributes,
            name,
            params,
            return_type,
            default_body,
        })
    }

    pub(super) fn parse_impl_def(&mut self) -> Result<AstImplDef, String> {
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

    pub(super) fn parse_function(&mut self) -> Result<AstFunction, String> {
        let (visibility, mut attributes) = self.parse_visibility_and_attribute_list()?;
        let (
            declared_test_name,
            test_ignored,
            test_should_fail,
            test_reason,
            test_timeout_ms,
            test_clock_domain,
            test_clock_policy,
            declared_benchmark_name,
            benchmark_warmup_iters,
            benchmark_measure_iters,
            benchmark_timeout_ms,
            benchmark_clock_domain,
            benchmark_clock_policy,
        ) = if self.peek_word("test") {
            self.expect_word("test")?;
            if !self.peek_symbol('(') {
                return Err(
                    "test declarations now require `test(...) fn ...`; the older bare-prefix `test ... fn ...` syntax has been retired"
                        .to_owned(),
                );
            }
            let (
                declared_test_name,
                test_ignored,
                test_should_fail,
                test_reason,
                test_timeout_ms,
                test_clock_domain,
                test_clock_policy,
            ) = self.parse_test_decl_call_syntax()?;
            (
                declared_test_name,
                test_ignored,
                test_should_fail,
                test_reason,
                test_timeout_ms,
                test_clock_domain,
                test_clock_policy,
                None,
                None,
                None,
                None,
                None,
                None,
            )
        } else if self.peek_word("benchmark") {
            self.expect_word("benchmark")?;
            if !self.peek_symbol('(') {
                return Err(
                    "benchmark declarations now require `benchmark(...) fn ...`; the older bare-prefix `benchmark ... fn ...` syntax has been retired"
                        .to_owned(),
                );
            }
            let (
                declared_benchmark_name,
                benchmark_warmup_iters,
                benchmark_measure_iters,
                benchmark_timeout_ms,
                benchmark_clock_domain,
                benchmark_clock_policy,
            ) = self.parse_benchmark_decl_call_syntax()?;
            (
                None,
                false,
                false,
                None,
                None,
                None,
                None,
                declared_benchmark_name,
                benchmark_warmup_iters,
                benchmark_measure_iters,
                benchmark_timeout_ms,
                benchmark_clock_domain,
                benchmark_clock_policy,
            )
        } else {
            (
                None, false, false, None, None, None, None, None, None, None, None, None, None,
            )
        };
        let is_async = if self.peek_word("async") {
            self.expect_word("async")?;
            true
        } else {
            false
        };
        if declared_test_name.is_some() && self.peek_word("benchmark") {
            return Err(
                "function cannot be both a test and a benchmark in the current MVP".to_owned(),
            );
        }
        if declared_benchmark_name.is_some() && self.peek_word("test") {
            return Err(
                "function cannot be both a test and a benchmark in the current MVP".to_owned(),
            );
        }
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
        let benchmark_from_attribute = attributes
            .iter()
            .find(|attribute| attribute.name == "benchmark")
            .map(|attribute| self.parse_benchmark_attribute_metadata(&attribute.args))
            .transpose()?;
        if declared_test_name.is_some() && test_from_attribute.is_some() {
            return Err(format!(
                "function `{name}` cannot use both `test(...)` and `@test(...)`; choose one test declaration style"
            ));
        }
        if declared_benchmark_name.is_some() && benchmark_from_attribute.is_some() {
            return Err(format!(
                "function `{name}` cannot use both `benchmark(...)` and `@benchmark(...)`; choose one benchmark declaration style"
            ));
        }
        if (declared_test_name.is_some() || test_from_attribute.is_some())
            && (declared_benchmark_name.is_some() || benchmark_from_attribute.is_some())
        {
            return Err(format!(
                "function `{name}` cannot be both a test and a benchmark in the current MVP"
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
        if declared_benchmark_name.is_some() {
            attributes.push(self.build_benchmark_attribute(
                declared_benchmark_name.clone(),
                benchmark_warmup_iters,
                benchmark_measure_iters,
                benchmark_timeout_ms,
                benchmark_clock_domain,
                benchmark_clock_policy,
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
        let (
            raw_benchmark_name,
            benchmark_warmup_iters,
            benchmark_measure_iters,
            benchmark_timeout_ms,
            benchmark_clock_domain,
            benchmark_clock_policy,
        ) = match benchmark_from_attribute {
            Some(values) => values,
            None => (
                declared_benchmark_name,
                benchmark_warmup_iters,
                benchmark_measure_iters,
                benchmark_timeout_ms,
                benchmark_clock_domain,
                benchmark_clock_policy,
            ),
        };
        let benchmark_name = raw_benchmark_name.map(|label| {
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
            benchmark_name,
            benchmark_warmup_iters,
            benchmark_measure_iters,
            benchmark_timeout_ms,
            benchmark_clock_domain,
            benchmark_clock_policy,
            is_async,
            generic_params,
            where_bounds,
            params,
            return_type,
            body,
        })
    }
}
