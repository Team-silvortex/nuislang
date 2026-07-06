use super::*;

#[derive(Clone)]
pub(super) struct TestMetadata {
    pub(super) label: Option<String>,
    pub(super) ignored: bool,
    pub(super) should_fail: bool,
    pub(super) reason: Option<String>,
    pub(super) timeout_ms: Option<i64>,
    pub(super) clock_domain: Option<TestClockDomain>,
    pub(super) clock_policy: Option<TestClockPolicy>,
}

#[derive(Clone)]
pub(super) struct BenchmarkMetadata {
    pub(super) label: Option<String>,
    pub(super) warmup_iters: Option<i64>,
    pub(super) measure_iters: Option<i64>,
    pub(super) timeout_ms: Option<i64>,
    pub(super) clock_domain: Option<TestClockDomain>,
    pub(super) clock_policy: Option<TestClockPolicy>,
}

impl Parser {
    pub(super) fn build_test_attribute(&self, metadata: TestMetadata) -> AstAttribute {
        let TestMetadata {
            label,
            ignored,
            should_fail,
            reason,
            timeout_ms,
            clock_domain,
            clock_policy,
        } = metadata;
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

    pub(super) fn parse_test_attribute_metadata(
        &self,
        args: &[AstAttributeArg],
    ) -> Result<TestMetadata, String> {
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

        Ok(TestMetadata {
            label,
            ignored,
            should_fail,
            reason,
            timeout_ms,
            clock_domain,
            clock_policy,
        })
    }

    pub(super) fn build_benchmark_attribute(&self, metadata: BenchmarkMetadata) -> AstAttribute {
        let BenchmarkMetadata {
            label,
            warmup_iters,
            measure_iters,
            timeout_ms,
            clock_domain,
            clock_policy,
        } = metadata;
        let mut args = Vec::new();
        if let Some(label) = label {
            if !label.is_empty() {
                args.push(AstAttributeArg {
                    name: None,
                    value: AstAttributeValue::String(label),
                });
            }
        }
        if let Some(warmup_iters) = warmup_iters {
            args.push(AstAttributeArg {
                name: Some("warmup_iters".to_owned()),
                value: AstAttributeValue::Int(warmup_iters),
            });
        }
        if let Some(measure_iters) = measure_iters {
            args.push(AstAttributeArg {
                name: Some("measure_iters".to_owned()),
                value: AstAttributeValue::Int(measure_iters),
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
            name: "benchmark".to_owned(),
            args,
        }
    }

    pub(super) fn parse_benchmark_attribute_metadata(
        &self,
        args: &[AstAttributeArg],
    ) -> Result<BenchmarkMetadata, String> {
        let mut label = Some(String::new());
        let mut warmup_iters = None;
        let mut measure_iters = None;
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
                            "`@benchmark(...)` accepts at most one positional string label"
                                .to_owned(),
                        );
                    }
                }
                (Some(name), AstAttributeValue::String(value)) => match name.as_str() {
                    "name" => label = Some(value.clone()),
                    "clock_domain" => {
                        clock_domain = Some(TestClockDomain::parse(value).ok_or_else(|| {
                            format!(
                                "unsupported `clock_domain=\"{}\"`; expected `monotonic`, `wall`, or `global`",
                                value
                            )
                        })?)
                    }
                    "clock_policy" => {
                        clock_policy = Some(TestClockPolicy::parse(value).ok_or_else(|| {
                            format!(
                                "unsupported `clock_policy=\"{}\"`; expected `bridge`",
                                value
                            )
                        })?)
                    }
                    other => {
                        return Err(format!(
                            "unknown benchmark metadata key `{other}` in `@benchmark(...)`"
                        ));
                    }
                },
                (Some(name), AstAttributeValue::Int(value)) => match name.as_str() {
                    "warmup_iters" => warmup_iters = Some(*value),
                    "measure_iters" => measure_iters = Some(*value),
                    "timeout_ms" => timeout_ms = Some(*value),
                    other => {
                        return Err(format!(
                            "unknown benchmark metadata key `{other}` in `@benchmark(...)`"
                        ));
                    }
                },
                (Some(name), _) => {
                    return Err(format!(
                        "benchmark metadata key `{name}` has unsupported value shape in `@benchmark(...)`"
                    ));
                }
                (None, _) => {
                    return Err(
                        "expected string label in `@benchmark(...)` positional arguments"
                            .to_owned(),
                    );
                }
            }
        }

        Ok(BenchmarkMetadata {
            label,
            warmup_iters,
            measure_iters,
            timeout_ms,
            clock_domain,
            clock_policy,
        })
    }

    pub(super) fn parse_test_decl_call_syntax(&mut self) -> Result<TestMetadata, String> {
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
        Ok(TestMetadata {
            label,
            ignored,
            should_fail,
            reason,
            timeout_ms,
            clock_domain,
            clock_policy,
        })
    }

    pub(super) fn parse_benchmark_decl_call_syntax(&mut self) -> Result<BenchmarkMetadata, String> {
        self.expect_symbol('(')?;
        let mut label = Some(String::new());
        let mut warmup_iters = None;
        let mut measure_iters = None;
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
                            "name" => label = Some(self.parse_test_meta_string()?),
                            "warmup_iters" => warmup_iters = Some(self.parse_test_meta_int()?),
                            "measure_iters" => measure_iters = Some(self.parse_test_meta_int()?),
                            "timeout_ms" => timeout_ms = Some(self.parse_test_meta_int()?),
                            "clock_domain" => clock_domain = Some(self.parse_test_clock_domain()?),
                            "clock_policy" => clock_policy = Some(self.parse_test_clock_policy()?),
                            _ => {
                                return Err(format!(
                                    "unknown benchmark metadata key `{word}` in `benchmark(...)`"
                                ))
                            }
                        }
                    } else {
                        return Err(format!(
                            "unknown benchmark metadata flag `{word}` in `benchmark(...)`"
                        ));
                    }
                }
                Some(other) => {
                    return Err(format!(
                        "expected string label or benchmark metadata in `benchmark(...)`, found {}",
                        describe_token(&other)
                    ))
                }
                None => return Err("unterminated `benchmark(...)` declaration".to_owned()),
            }
            if self.peek_symbol(',') {
                self.expect_symbol(',')?;
            } else {
                break;
            }
        }
        self.expect_symbol(')')?;
        Ok(BenchmarkMetadata {
            label,
            warmup_iters,
            measure_iters,
            timeout_ms,
            clock_domain,
            clock_policy,
        })
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
        if self.peek_symbol('-') {
            self.expect_symbol('-')?;
            return match self.next() {
                Some(Token::Integer(value)) => Ok(-value),
                Some(other) => Err(format!(
                    "expected integer literal in test metadata, found {}",
                    describe_token(&other)
                )),
                None => {
                    Err("expected integer literal in test metadata, found end of input".to_owned())
                }
            };
        }
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
}
