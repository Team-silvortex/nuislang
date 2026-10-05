use super::*;
use crate::lowering::scalar_record_shape::source_value;

#[path = "capture_record_scalar_calls_tests.rs"]
mod scalar_calls;

const COPY: &str = "State { left: Leaf { value: state.left.value,
    gain: state.left.gain, tag: state.left.tag, enabled: state.left.enabled },
    right: other.right, count: state.count + other.count }";

fn module(body: &str, callers: &str) -> NirModule {
    crate::frontend::parse_nuis_module(&format!(
        "mod cpu Main {{
        struct Leaf {{ value: f64, gain: f32, tag: i32, enabled: bool }}
        struct State {{ left: Leaf, right: Leaf, count: i64 }}
        fn relay(state: State) -> State {{ return state; }}
        fn consume(state: State) -> i64 {{ return state.count; }}
        fn read_count(value: i64) -> i64 {{ return value; }}
        @noinline fn helper(state: State, other: State) -> i64 {{ {body} }}
        {callers}
    }}"
    ))
    .unwrap()
}

fn helper(module: &mut NirModule) -> &mut NirFunction {
    module
        .functions
        .iter_mut()
        .find(|f| f.name == "helper")
        .unwrap()
}

fn normalize_and_verify(module: &mut NirModule) {
    let layouts = control_values::TypedLayouts::collect(module);
    normalize(helper(module), &layouts, &BTreeSet::new());
    crate::nir_verify::verify_nir_module(module).unwrap();
}

fn inputs(expr: &NirExpr) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    control_values::collect_inputs(expr, &mut names);
    names
}

#[test]
fn private_record_copies_prune_only_unobserved_total_input_fields() {
    for declaration in ["let snapshot =", "const snapshot: State ="] {
        let mut module = module(&format!("{declaration} {COPY}; return snapshot.count;"), "");
        normalize_and_verify(&mut module);
        let (NirStmt::Let { value, .. } | NirStmt::Const { value, .. }) =
            &helper(&mut module).body[0]
        else {
            panic!()
        };
        let NirExpr::StructLiteral { fields, .. } = value else {
            panic!()
        };
        assert!(inputs(&fields[0].1).is_empty());
        assert!(inputs(&fields[1].1).is_empty());
        assert!(matches!(
            fields[2].1,
            NirExpr::Binary {
                op: NirBinaryOp::Add,
                ..
            }
        ));
        assert_eq!(
            inputs(&fields[2].1),
            BTreeSet::from(["state".into(), "other".into()])
        );
        let before = module.clone();
        normalize_and_verify(&mut module);
        assert_eq!(module, before);
    }
}

#[test]
fn private_record_copies_expand_partial_subrecords_and_keep_whole_observations() {
    for observed in ["snapshot.right.tag", "snapshot.right"] {
        let body = format!("let snapshot = {COPY}; let observed = {observed}; return 0;");
        let mut module = module(&body, "");
        normalize_and_verify(&mut module);
        let NirStmt::Let {
            value: NirExpr::StructLiteral { fields, .. },
            ..
        } = &helper(&mut module).body[0]
        else {
            panic!()
        };
        if observed.ends_with(".tag") {
            let NirExpr::StructLiteral { fields, .. } = &fields[1].1 else {
                panic!()
            };
            for (name, value) in fields {
                assert_eq!(inputs(value).is_empty(), name != "tag");
            }
        } else {
            assert_eq!(
                access(&fields[1].1),
                Some(vec!["other".into(), "right".into()])
            );
        }
        assert!(inputs(&fields[0].1).is_empty());
    }
}

#[test]
fn private_record_copies_preserve_computed_checked_call_and_codec_fields_in_order() {
    for count in [
        "state.count + other.count",
        "state.count / (state.count - state.count)",
        "consume(relay(state))",
        "state.count + 0",
    ] {
        let source = COPY.replace("state.count + other.count", count);
        let mut module = module(
            &format!("let snapshot = {source}; let observed = snapshot.right.tag; return 0;"),
            "",
        );
        if count == "state.count + 0" {
            let NirStmt::Let {
                value: NirExpr::StructLiteral { fields, .. },
                ..
            } = &mut helper(&mut module).body[0]
            else {
                panic!()
            };
            fields[2].1 = NirExpr::CastI32ToI64(Box::new(NirExpr::CastI64ToI32(Box::new(
                fields[2].1.clone(),
            ))));
        }
        let original = match &helper(&mut module).body[0] {
            NirStmt::Let {
                value: NirExpr::StructLiteral { fields, .. },
                ..
            } => fields[2].clone(),
            _ => panic!(),
        };
        normalize_and_verify(&mut module);
        let NirStmt::Let {
            value: NirExpr::StructLiteral { fields, .. },
            ..
        } = &helper(&mut module).body[0]
        else {
            panic!()
        };
        assert_eq!(fields[2], original);
        assert!(inputs(&fields[0].1).is_empty());
        assert_eq!(
            fields
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            ["left", "right", "count"]
        );
    }
}

#[test]
fn private_record_copies_preserve_exact_pure_transport_and_codec_reconstructions() {
    for codecs in [false, true] {
        let source = COPY.replace("state.count + other.count", "state.count");
        let mut module = module(
            &format!("let snapshot = {source}; return consume(snapshot);"),
            "",
        );
        if codecs {
            let NirStmt::Let {
                value: NirExpr::StructLiteral { fields, .. },
                ..
            } = &mut helper(&mut module).body[0]
            else {
                panic!()
            };
            fields[2].1 = NirExpr::CastI32ToI64(Box::new(NirExpr::CastI64ToI32(Box::new(
                fields[2].1.clone(),
            ))));
            let NirExpr::StructLiteral { fields, .. } = &mut fields[0].1 else {
                panic!()
            };
            fields[0].1 = NirExpr::UnpackF64Word(Box::new(NirExpr::PackF64Word(Box::new(
                fields[0].1.clone(),
            ))));
        }
        let before = module.clone();
        normalize_and_verify(&mut module);
        assert_eq!(module, before);
    }
}

#[test]
fn private_record_copies_preserve_registered_types_and_immutable_transport_chains() {
    for alias in ["snapshot", "forwarded"] {
        let binding = if alias == "forwarded" {
            "let selected = snapshot; const forwarded: State = selected;"
        } else {
            ""
        };
        let mut module = module(
            &format!("let snapshot = {COPY}; {binding} return consume({alias});"),
            "",
        );
        let before = module.clone();
        normalize_and_verify(&mut module);
        assert_eq!(module, before);
    }
    let mut module = module(
        &format!("let snapshot = {COPY}; return snapshot.count;"),
        "",
    );
    let layouts = control_values::TypedLayouts::collect(&module);
    let before = module.clone();
    normalize(
        helper(&mut module),
        &layouts,
        &BTreeSet::from(["State".into()]),
    );
    assert_eq!(module, before);
    normalize_and_verify(&mut module);
    assert_ne!(module, before);
}

#[test]
fn private_record_copies_keep_escapes_mutable_shadowed_and_local_sources() {
    let source = COPY.replace("other.right", "state.right");
    for body in [
        format!("let snapshot = {source}; return consume(snapshot);"),
        format!("let snapshot = {source}; let saved = snapshot; return saved.count;"),
        format!("let snapshot = {source}; let snapshot = other; return snapshot.count;"),
        format!("let snapshot = {source}; let state = other; return snapshot.count;"),
        format!("if state.count > 0 {{ let state = other; }} let snapshot = {source}; return snapshot.count;"),
        format!("let local = state; let snapshot = {}; return snapshot.count;", source.replace("state.", "local.")),
    ] {
        let mut module = module(&body, "");
        let before = module.clone();
        normalize_and_verify(&mut module);
        assert_eq!(module, before, "{body}");
    }
}

#[test]
fn private_record_copies_require_exact_nominal_types_and_valid_observed_paths() {
    for mutation in 0..11 {
        let mut module = module(
            &format!("let snapshot: State = {COPY}; return snapshot.count;"),
            "",
        );
        let NirStmt::Let {
            ty,
            value:
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                },
            ..
        } = &mut helper(&mut module).body[0]
        else {
            panic!()
        };
        match mutation {
            0 => *type_name = "Leaf".into(),
            1 => fields[1].1 = source_value("state", &["count".into()]),
            2 => fields[1].0 = "left".into(),
            3 => fields[1].0 = "missing".into(),
            4 => *ty = Some(scalar_type("Leaf")),
            5 => type_args.push(scalar_type("i64")),
            6 => helper(&mut module).params[0].ty.is_ref = true,
            7 => helper(&mut module).params[0].ty.is_optional = true,
            8 => helper(&mut module).params[0].ty.name = "Buffer".into(),
            9 => {
                fields.pop();
            }
            10 => {
                let NirStmt::Return(Some(NirExpr::FieldAccess { field, .. })) =
                    helper(&mut module).body.last_mut().unwrap()
                else {
                    panic!()
                };
                *field = "missing".into();
            }
            _ => unreachable!(),
        }
        // Use one input only: unsupported/mutable sources cannot authorize copies.
        if mutation >= 6 && mutation <= 8 {
            let NirStmt::Let { value, .. } = &mut helper(&mut module).body[0] else {
                panic!()
            };
            walk::rewrite_expr(value, |expr| {
                if let NirExpr::Var(name) = expr {
                    if name == "other" {
                        *name = "state".into();
                    }
                }
            });
        }
        let layouts = control_values::TypedLayouts::collect(&module);
        let before = module.clone();
        normalize(helper(&mut module), &layouts, &BTreeSet::new());
        assert_eq!(module, before, "{mutation}");
    }
}

#[test]
fn private_record_copies_keep_all_callers_transactional() {
    for computed in [false, true] {
        let call = if computed { "relay(state)" } else { "state" };
        let suffix = if computed { " + 0" } else { "" };
        let mut module = module(
            &format!("let snapshot = {COPY}; return snapshot.count;"),
            &format!(
                "fn first(state: State) -> i64 {{ return helper(state, state); }}
                fn second(state: State) -> i64 {{ return helper({call}, state){suffix}; }}"
            ),
        );
        let layouts = control_values::TypedLayouts::collect(&module);
        let before = module.clone();
        let result = project(
            &mut module,
            &BTreeSet::from(["helper".into()]),
            &BTreeSet::new(),
            &layouts,
            &BTreeMap::new(),
        );
        assert_eq!(result.changed, !computed);
        if computed {
            assert_eq!(module, before);
        } else {
            assert_eq!(
                helper(&mut module)
                    .params
                    .iter()
                    .map(|p| p.ty.name.as_str())
                    .collect::<Vec<_>>(),
                ["i64", "i64"]
            );
            crate::nir_verify::verify_nir_module(&module).unwrap();
        }
    }
}

#[test]
fn private_record_copies_preserve_guarded_returns_and_unused_checked_work() {
    let execute = |module: &NirModule| {
        let mut yir = crate::lowering::lower_nir_to_yir_builtin_cpu(module).unwrap();
        yir.nodes.reverse();
        yir.functions.reverse();
        for function in &mut yir.functions {
            function.body_nodes.reverse();
        }
        let trace = yir_runtime_host::execute_module_source_with_registry(
            &crate::render::render_yir(&yir),
            &yir_verify::default_registry(),
        )
        .map_err(|_| ())?;
        let main = yir
            .functions
            .iter()
            .find(|f| f.role == yir_core::YirFunctionRole::Entry)
            .unwrap();
        Ok(trace.values[&main.result.as_ref().unwrap().node].clone())
    };
    for count in [0, 1, 2, 3] {
        for divisor in [0, 1, 2, 4] {
            for position in ["none", "before", "after"] {
                let guard = "if state.count == 2 { return state.left.used; }";
                let before_guard = if position == "before" { guard } else { "" };
                let after_guard = if position == "after" { guard } else { "" };
                let mut module = crate::frontend::parse_nuis_module(&format!(
                    "mod cpu Main {{
                    struct Leaf {{ used: i64, unused: i64 }}
                    struct State {{ left: Leaf, right: Leaf, count: i64 }}
                    @noinline fn checked(divisor: i64) -> i64 {{ return 10 / divisor; }}
                    @noinline fn helper(state: State, other: State) -> i64 {{
                        if state.count == 0 {{ return 0; }}
                        {before_guard}
                        let snapshot = State {{
                            left: Leaf {{ used: state.left.used, unused: state.left.unused }},
                            right: state.right, count: checked(other.count - state.count)
                        }};
                        {after_guard}
                        return snapshot.left.used;
                    }}
                    fn main() -> i64 {{ return helper(
                        State {{ left: Leaf {{ used: 7, unused: 99 }}, right: Leaf {{ used: 101, unused: 103 }}, count: {count} }},
                        State {{ left: Leaf {{ used: 0, unused: 0 }}, right: Leaf {{ used: 0, unused: 0 }}, count: {divisor} }});
                    }} }}"
                ))
                .unwrap();
                let expected = if count == 0 {
                    Ok(yir_core::Value::Int(0))
                } else if position == "before" && count == 2 {
                    Ok(yir_core::Value::Int(7))
                } else if divisor == count {
                    Err(())
                } else {
                    Ok(yir_core::Value::Int(7))
                };
                let before = execute(&module);
                assert_eq!(before, expected, "{count}/{divisor}/{position}");
                let original = module.clone();
                normalize_and_verify(&mut module);
                assert_ne!(module, original);
                assert_eq!(execute(&module), expected, "{count}/{divisor}/{position}");
            }
        }
    }
}

#[test]
fn private_record_copies_bound_work_and_depth_without_partial_rewrites() {
    let mut module = module(
        &format!("let snapshot = {COPY}; return snapshot.count;"),
        "",
    );
    let layouts = control_values::TypedLayouts::collect(&module);
    let expected = normalized(helper(&mut module), &layouts, &BTreeSet::new(), 65_536).unwrap();
    assert_ne!(expected, helper(&mut module).body);
    for budget in 0..1_024 {
        if let Some(actual) = normalized(helper(&mut module), &layouts, &BTreeSet::new(), budget) {
            assert_eq!(actual, expected);
        }
    }
    let NirStmt::Return(Some(value)) = helper(&mut module).body.last_mut().unwrap() else {
        panic!()
    };
    for _ in 0..64 {
        *value = NirExpr::FieldAccess {
            base: Box::new(value.clone()),
            field: "left".into(),
        };
    }
    let before = module.clone();
    normalize(helper(&mut module), &layouts, &BTreeSet::new());
    assert_eq!(module, before);
    for _ in 0..64 {
        let body = std::mem::take(&mut helper(&mut module).body);
        helper(&mut module).body = vec![NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: body,
            else_body: Vec::new(),
        }];
    }
    let before = module.clone();
    normalize(helper(&mut module), &layouts, &BTreeSet::new());
    assert_eq!(module, before);
}
