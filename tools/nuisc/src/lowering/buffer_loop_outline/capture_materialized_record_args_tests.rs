use super::*;

const BODY: &str = "let input = Input { used: checked(state.count), leaf: state.leaf };
    let alias = input; let first = make(alias);
    const later_input: Input = Input { leaf: other.leaf, used: checked(other.count) };
    let later_alias = later_input; const later: State = make(later_alias);
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
fn materialized_record_args_keep_constructors_aliases_and_calls_at_original_sites() {
    let mut module = module(BODY);
    let prefix = helper(&mut module).body[..6].to_vec();
    let body = normalized(&mut module, 65_536).unwrap();
    assert_eq!(&body[..6], prefix);
    assert_eq!(body.len(), 7);
    assert!(
        matches!(&body[6], NirStmt::Return(Some(NirExpr::FieldAccess { base, field }))
        if **base == NirExpr::Var("first".into()) && field == "count")
    );
    helper(&mut module).body = body;
    crate::nir_verify::verify_nir_module(&module).unwrap();
    let before = module.clone();
    helper(&mut module).body = normalized(&mut module, 65_536).unwrap();
    assert_eq!(module, before);
}

#[test]
fn materialized_record_args_admit_total_views_without_erasing_whole_call_operands() {
    let body = BODY
        .replace("checked(state.count)", "state.count")
        .replace("checked(other.count)", "other.count");
    let mut module = module(&body);
    let result = normalized(&mut module, 65_536).unwrap();
    // Total aliases retain the stored call operands, while their original
    // now-unread reconstruction bindings remain eligible for existing pruning.
    assert_eq!(result.len(), 5);
    for name in ["alias", "later_alias", "first", "later"] {
        assert!(result.iter().any(|s| matches!(s,
            NirStmt::Let { name: n, .. } | NirStmt::Const { name: n, .. } if n == name)));
    }
    assert!(
        matches!(&result[1], NirStmt::Let { value: NirExpr::Call { args, .. }, .. }
        if args == &[NirExpr::Var("alias".into())])
    );
    helper(&mut module).body = result;
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

#[test]
fn materialized_record_args_reject_nominal_kind_effect_and_version_mismatches() {
    for mutation in 0..8 {
        let mut module = module(BODY);
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
                *ty = Some(scalar_type("State"));
            }
            6 => {
                let checked = module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "checked")
                    .unwrap();
                checked.body.insert(0, NirStmt::Print(NirExpr::Int(1)));
            }
            7 => {
                let mut rebound = helper(&mut module).body[0].clone();
                if let NirStmt::Let {
                    value: NirExpr::StructLiteral { fields, .. },
                    ..
                } = &mut rebound
                {
                    fields[0].1 = NirExpr::Int(0);
                }
                helper(&mut module).body.insert(1, rebound);
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
fn materialized_record_args_keep_scope_control_transport_and_scalar_mode_boundaries() {
    let mut module = module(BODY);
    let input = helper(&mut module).body.remove(0);
    helper(&mut module).body.insert(
        0,
        NirStmt::If {
            condition: NirExpr::Var("flag".into()),
            then_body: vec![input],
            else_body: vec![],
        },
    );
    let before = helper(&mut module).body.clone();
    assert_eq!(normalized(&mut module, 65_536).unwrap(), before);

    for boundary in 0..3 {
        let mut module = self::module(BODY);
        let layouts = control_values::TypedLayouts::collect(&module);
        let catalog = scalar_helpers::collect_capture_values(&module, &layouts);
        let before = helper(&mut module).body.clone();
        let result = normalized_mode(
            helper(&mut module),
            &layouts,
            Some(Evaluated {
                catalog: &catalog,
                controls: &if boundary == 0 {
                    BTreeSet::from(["input".into()])
                } else {
                    BTreeSet::new()
                },
                transport_types: &if boundary == 1 {
                    BTreeSet::from(["Input".into()])
                } else {
                    BTreeSet::new()
                },
                call_results: boundary != 2,
            }),
            65_536,
        )
        .unwrap();
        assert_eq!(result, before, "{boundary}");
    }
}

#[test]
fn materialized_record_args_keep_whole_consumers_and_checked_unused_constructors() {
    for body in [
        "let input = Input { used: checked(state.count), leaf: state.leaf };
            let first = make(input); let saved = first; return consume(saved);",
        "let input = Input { used: checked(state.count), leaf: state.leaf }; return state.count;",
    ] {
        let mut module = module(body);
        let before = helper(&mut module).body.clone();
        assert_eq!(normalized(&mut module, 65_536).unwrap(), before);
    }
}

#[test]
fn materialized_record_args_budget_and_depth_roll_back_the_whole_body() {
    let mut module = module(BODY);
    let expected = normalized(&mut module, 65_536).unwrap();
    let mut accepted = 0;
    let mut rejected = 0;
    for budget in 1..2048 {
        match normalized(&mut module, budget) {
            Some(body) => {
                assert_eq!(body, expected, "{budget}");
                accepted += 1;
            }
            None => rejected += 1,
        }
    }
    assert!(accepted > 0 && rejected > 0);
    let mut expr = NirExpr::Int(1);
    for _ in 0..70 {
        expr = NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(expr),
            rhs: Box::new(NirExpr::Int(0)),
        };
    }
    let NirStmt::Let {
        value: NirExpr::StructLiteral { fields, .. },
        ..
    } = &mut helper(&mut module).body[0]
    else {
        panic!()
    };
    fields[0].1 = expr;
    assert!(normalized(&mut module, 65_536).is_none());
}

#[test]
fn materialized_record_args_preserve_checked_order_with_an_independent_oracle() {
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
        let entry = yir
            .functions
            .iter()
            .find(|f| f.role == yir_core::YirFunctionRole::Entry)
            .unwrap();
        Ok(trace.values[&entry.result.as_ref().unwrap().node].clone())
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
                        let input = Pair {{ used: 7, unused: checked(1) }};
                        let alias = input; let first = make(alias);
                        const next: Pair = Pair {{ unused: checked(state.unused - count), used: 37 }};
                        let later_alias = next; const later: Pair = make(later_alias);
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
                helper(&mut module).body = normalized(&mut module, 65_536).unwrap();
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
fn materialized_record_args_commit_only_after_every_caller_agrees() {
    for computed in [false, true] {
        for reverse in [false, true] {
            let mut module = module(&BODY.replace("leaf: state.leaf", "leaf: other.leaf"));
            let target = helper(&mut module).clone();
            let maker = module
                .functions
                .iter()
                .find(|f| f.name == "make")
                .unwrap()
                .clone();
            for name in ["entry", "second"] {
                let mut caller = target.clone();
                caller.name = name.into();
                caller.body = vec![NirStmt::Return(Some(NirExpr::Call {
                    callee: "helper".into(),
                    args: vec![
                        if computed && name == "second" {
                            NirExpr::Call {
                                callee: "make".into(),
                                args: vec![NirExpr::StructLiteral {
                                    type_name: "Input".into(),
                                    type_args: vec![],
                                    fields: vec![
                                        ("used".into(), NirExpr::Int(2)),
                                        (
                                            "leaf".into(),
                                            NirExpr::FieldAccess {
                                                base: Box::new(NirExpr::Var("other".into())),
                                                field: "leaf".into(),
                                            },
                                        ),
                                    ],
                                }],
                            }
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
                assert_eq!(helper(&mut module).body.len(), 7);
                assert_eq!(
                    helper(&mut module)
                        .params
                        .iter()
                        .map(|p| p.ty.name.as_str())
                        .collect::<Vec<_>>(),
                    ["i64", "State", "bool"]
                );
                assert_eq!(
                    module.functions.iter().find(|f| f.name == "make").unwrap(),
                    &maker
                );
            }
        }
    }
}
