use super::*;

const BODY: &str = "let first = make(Input { used: checked(state.count),
    leaf: Leaf { value: state.leaf.value, gain: state.leaf.gain,
        tag: state.leaf.tag, enabled: state.leaf.enabled } });
    const later: State = make(Input { leaf: state.leaf, used: checked(other.count) });
    let snapshot = State { leaf: later.leaf, count: first.count };
    let saved = snapshot; return saved.count;";

fn module(body: &str) -> NirModule {
    crate::frontend::parse_nuis_module(&format!(
        "mod cpu Main {{ struct Leaf {{ value: f64, gain: f32, tag: i32, enabled: bool }}
        struct State {{ leaf: Leaf, count: i64 }} struct Input {{ used: i64, leaf: Leaf }}
        struct TwinInput {{ used: i64, leaf: Leaf }}
        @noinline fn checked(value: i64) -> i64 {{ return 10 / value; }}
        @noinline fn make(input: Input) -> State {{
            return State {{ leaf: input.leaf, count: input.used }};
        }} fn consume(state: State) -> i64 {{ return state.count; }}
        fn helper(state: State, other: State, flag: bool) -> i64 {{ {body} }} }}"
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

fn normalized(module: &mut NirModule, budget: usize) -> Option<Vec<NirStmt>> {
    let layouts = control_values::TypedLayouts::collect(module);
    let catalog = scalar_helpers::collect_capture_values(module, &layouts);
    normalized_mode(
        helper(module),
        &layouts,
        Some(Evaluated {
            catalog: &catalog,
            controls: &BTreeSet::new(),
            transport_types: &BTreeSet::new(),
            call_results: true,
        }),
        budget,
    )
}

#[test]
fn inline_record_args_keep_full_nested_ordered_arguments_and_independent_calls() {
    let mut module = module(BODY);
    let original = helper(&mut module).body[..2].to_vec();
    let body = normalized(&mut module, 65_536).unwrap();
    assert_eq!(&body[..2], original);
    assert_eq!(body.len(), 3);
    assert!(
        matches!(&body[2], NirStmt::Return(Some(NirExpr::FieldAccess { base, field }))
        if **base == NirExpr::Var("first".into()) && field == "count")
    );
    helper(&mut module).body = body;
    crate::nir_verify::verify_nir_module(&module).unwrap();
    let before = module.clone();
    helper(&mut module).body = normalized(&mut module, 65_536).unwrap();
    assert_eq!(module, before);
}

#[test]
fn inline_record_args_require_exact_nominal_fields_kinds_and_no_generics() {
    for mutation in 0..6 {
        let mut module = module(BODY);
        let NirStmt::Let {
            value: NirExpr::Call { args, .. },
            ..
        } = &mut helper(&mut module).body[0]
        else {
            panic!()
        };
        let NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } = &mut args[0]
        else {
            panic!()
        };
        match mutation {
            0 => {
                fields.pop();
            }
            1 => {
                fields[1].0 = fields[0].0.clone();
            }
            2 => {
                fields[0].1 = NirExpr::Bool(false);
            }
            3 => {
                *type_name = "TwinInput".into();
            }
            4 => {
                type_args.push(scalar_type("i64"));
            }
            5 => {
                fields[1].0 = "unknown".into();
            }
            _ => unreachable!(),
        }
        let before = helper(&mut module).body.clone();
        assert_eq!(
            normalized(&mut module, 65_536).unwrap(),
            before,
            "{mutation}"
        );
    }
}

#[test]
fn inline_record_args_reject_unknown_calls_changed_versions_and_unsupported_fields() {
    for body in [
        format!("let state = other; {BODY}"),
        BODY.replace(
            "checked(state.count)",
            "make(Input { used: 1, leaf: state.leaf }).count",
        ),
    ] {
        let mut module = module(&body);
        let before = helper(&mut module).body.clone();
        assert_eq!(normalized(&mut module, 65_536).unwrap(), before);
    }
    for mutation in 0..3 {
        let mut module = module(BODY);
        if mutation == 0 {
            let NirStmt::Let {
                value: NirExpr::Call { args, .. },
                ..
            } = &mut helper(&mut module).body[0]
            else {
                panic!()
            };
            let NirExpr::StructLiteral { fields, .. } = &mut args[0] else {
                panic!()
            };
            let NirExpr::Call { callee, .. } = &mut fields[0].1 else {
                panic!()
            };
            *callee = "unknown".into();
        } else {
            let checked = module
                .functions
                .iter_mut()
                .find(|f| f.name == "checked")
                .unwrap();
            if mutation == 1 {
                checked.body.insert(0, NirStmt::Print(NirExpr::Int(1)));
            } else {
                checked.body = vec![NirStmt::Return(Some(NirExpr::Call {
                    callee: "checked".into(),
                    args: vec![NirExpr::Var("value".into())],
                }))];
            }
        }
        let before = helper(&mut module).body.clone();
        assert_eq!(
            normalized(&mut module, 65_536).unwrap(),
            before,
            "{mutation}"
        );
    }
}

#[test]
fn inline_record_args_keep_whole_consumers_and_unbound_result_projections_conservative() {
    for body in [
        "let first = make(Input { used: checked(state.count), leaf: state.leaf });
            let saved = first; return consume(saved);",
        "let saved = State { count: make(Input { used: checked(state.count),
            leaf: state.leaf }).count, leaf: state.leaf }; return saved.count;",
    ] {
        let mut module = module(body);
        let before = helper(&mut module).body.clone();
        assert_eq!(normalized(&mut module, 65_536).unwrap(), before);
    }
}

#[test]
fn inline_record_args_keep_lexical_readiness_and_the_original_scalar_only_mode() {
    let mut module = module(BODY);
    let NirStmt::Let {
        value: NirExpr::Call { args, .. },
        ..
    } = &mut helper(&mut module).body[0]
    else {
        panic!()
    };
    let NirExpr::StructLiteral { fields, .. } = &mut args[0] else {
        panic!()
    };
    fields[0].1 = NirExpr::Var("scalar".into());
    let before = helper(&mut module).body.clone();
    assert_eq!(normalized(&mut module, 65_536).unwrap(), before);
    helper(&mut module).body.insert(
        0,
        NirStmt::Let {
            name: "scalar".into(),
            ty: Some(scalar_type("i64")),
            value: NirExpr::Int(2),
        },
    );
    assert_eq!(normalized(&mut module, 65_536).unwrap().len(), 4);

    let mut module = self::module(BODY);
    let before = module.clone();
    let layouts = control_values::TypedLayouts::collect(&module);
    let catalog = scalar_helpers::collect(&module);
    normalize_evaluated(
        helper(&mut module),
        &layouts,
        &catalog,
        &BTreeSet::new(),
        &BTreeSet::new(),
    );
    assert_eq!(module, before);
}

#[test]
fn inline_record_args_exhaustion_and_depth_never_install_partial_views() {
    let mut module = module(BODY);
    let expected = normalized(&mut module, 65_536).unwrap();
    let mut completed = 0;
    let mut rejected = 0;
    for budget in 1..1024 {
        match normalized(&mut module, budget) {
            Some(body) => {
                assert_eq!(body, expected, "{budget}");
                completed += 1;
            }
            None => rejected += 1,
        }
    }
    assert!(completed > 0 && rejected > 0);
    let mut expr = NirExpr::Int(1);
    for _ in 0..70 {
        expr = NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(expr),
            rhs: Box::new(NirExpr::Int(0)),
        };
    }
    let NirStmt::Let {
        value: NirExpr::Call { args, .. },
        ..
    } = &mut helper(&mut module).body[0]
    else {
        panic!()
    };
    let NirExpr::StructLiteral { fields, .. } = &mut args[0] else {
        panic!()
    };
    fields[0].1 = expr;
    assert!(normalized(&mut module, 65_536).is_none());
}

#[test]
fn inline_record_args_preserve_checked_unused_fields_with_an_independent_oracle() {
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
                let guard = "if count == 2 { return 7; }";
                let before = if position == "before" { guard } else { "" };
                let after = if position == "after" { guard } else { "" };
                let mut module = crate::frontend::parse_nuis_module(&format!(
                    "mod cpu Main {{ struct Pair {{ used: i64, unused: i64 }}
                    @noinline fn checked(value: i64) -> i64 {{ return 10 / value; }}
                    @noinline fn make(input: Pair) -> Pair {{ return input; }}
                    fn helper(state: Pair, count: i64) -> i64 {{
                        if count == 0 {{ return 0; }} {before}
                        let first = make(Pair {{ used: 7, unused: checked(1) }});
                        const later: Pair = make(Pair {{ unused: checked(state.unused - count), used: 37 }});
                        let snapshot = Pair {{ used: first.used, unused: later.unused }};
                        let saved = snapshot; {after} return saved.used;
                    }} fn main() -> i64 {{ return helper(Pair {{ used: 99, unused: {divisor} }}, {count}); }} }}"
                )).unwrap();
                let expected = if count == 0 {
                    Ok(yir_core::Value::Int(0))
                } else if position == "before" && count == 2 {
                    Ok(yir_core::Value::Int(7))
                } else if divisor == count {
                    Err(())
                } else {
                    Ok(yir_core::Value::Int(7))
                };
                assert_eq!(
                    execute(&module),
                    expected,
                    "before {count}/{divisor}/{position}"
                );
                let original = module.clone();
                let body = normalized(&mut module, 65_536).unwrap();
                helper(&mut module).body = body;
                assert_ne!(module, original);
                crate::nir_verify::verify_nir_module(&module).unwrap();
                assert_eq!(
                    execute(&module),
                    expected,
                    "after {count}/{divisor}/{position}"
                );
            }
        }
    }
}

#[test]
fn inline_record_args_project_only_after_every_caller_agrees_without_spilling_calls() {
    for computed in [false, true] {
        for reverse in [false, true] {
            let mut module = module(&BODY.replace("state.leaf", "other.leaf"));
            let target = helper(&mut module).clone();
            let producer = module
                .functions
                .iter()
                .find(|f| f.name == "make")
                .unwrap()
                .clone();
            let NirStmt::Let {
                value: NirExpr::Call { args, .. },
                ..
            } = &target.body[0]
            else {
                panic!()
            };
            let computed_arg = NirExpr::Call {
                callee: "make".into(),
                args: args.clone(),
            };
            for name in ["entry", "second"] {
                let mut caller = target.clone();
                caller.name = name.into();
                caller.body = vec![NirStmt::Return(Some(NirExpr::Call {
                    callee: "helper".into(),
                    args: vec![
                        if computed && name == "second" {
                            computed_arg.clone()
                        } else {
                            NirExpr::Var("state".into())
                        },
                        NirExpr::Var("other".into()),
                        NirExpr::Var("flag".into()),
                    ],
                }))];
                if computed && name == "second" {
                    let NirStmt::Return(Some(value)) = &mut caller.body[0] else {
                        panic!()
                    };
                    *value = NirExpr::Binary {
                        op: nuis_semantics::model::NirBinaryOp::Add,
                        lhs: Box::new(NirExpr::Int(0)),
                        rhs: Box::new(value.clone()),
                    };
                }
                module.functions.push(caller);
            }
            if reverse {
                module.functions.reverse();
            }
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let before = module.clone();
            let layouts = control_values::TypedLayouts::collect(&module);
            let result = super::super::project(
                &mut module,
                &BTreeSet::from(["helper".into()]),
                &BTreeSet::new(),
                &layouts,
                &BTreeMap::new(),
            );
            crate::nir_verify::verify_nir_module(&module).unwrap();
            if computed {
                assert!(!result.changed);
                assert_eq!(module, before);
            } else {
                assert!(result.changed);
                let target = helper(&mut module);
                assert_eq!(
                    target
                        .params
                        .iter()
                        .map(|p| p.ty.name.as_str())
                        .collect::<Vec<_>>(),
                    ["i64", "State", "bool"]
                );
                assert_eq!(target.body.len(), 3);
                assert!(target.body[..2].iter().all(|s| matches!(
                    s,
                    NirStmt::Let {
                        value: NirExpr::Call { .. },
                        ..
                    } | NirStmt::Const {
                        value: NirExpr::Call { .. },
                        ..
                    }
                )));
                assert_eq!(
                    module.functions.iter().find(|f| f.name == "make").unwrap(),
                    &producer
                );
            }
        }
    }
}
