use super::*;

const BODY: &str =
    "let first = make(Input { used: checked(state.count), leaf: other.leaf }).payload.state;
    const later: State = make(Input { leaf: other.leaf, used: checked(other.count) }).payload.state;
    let snapshot = State { leaf: later.leaf, count: first.count };
    let saved = snapshot; return saved.count;";

fn module(body: &str) -> NirModule {
    crate::frontend::parse_nuis_module(&format!(
        "mod cpu Main {{ struct Leaf {{ value: f64, gain: f32, tag: i32, enabled: bool }}
        struct State {{ leaf: Leaf, count: i64 }} struct Input {{ used: i64, leaf: Leaf }}
        struct Layer {{ state: State }} struct Envelope {{ payload: Layer, unused: i64 }}
        @noinline fn checked(value: i64) -> i64 {{ return 10 / value; }}
        @noinline fn make(input: Input) -> Envelope {{
            return Envelope {{ payload: Layer {{ state: State {{ leaf: input.leaf, count: input.used }} }}, unused: checked(input.used) }};
        }} fn consume(state: State) -> i64 {{ return state.count; }}
        fn helper(state: State, other: State, flag: bool) -> i64 {{ {body} }} }}"
    )).unwrap()
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
fn stored_projection_keeps_nested_call_rhs_and_independent_bindings_unchanged() {
    let mut module = module(BODY);
    let prefix = helper(&mut module).body[..2].to_vec();
    let body = normalized(&mut module, 65_536).unwrap();
    assert_eq!(body.len(), 3);
    assert_eq!(&body[..2], prefix);
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
fn stored_projection_retains_scalar_mixed_kinds_and_every_original_call() {
    let mut module = module(
        "let value = make(Input { used: state.count, leaf: other.leaf }).payload.state.leaf.value;
        let gain = make(Input { used: state.count, leaf: other.leaf }).payload.state.leaf.gain;
        let tag = make(Input { used: state.count, leaf: other.leaf }).payload.state.leaf.tag;
        let enabled = make(Input { used: state.count, leaf: other.leaf }).payload.state.leaf.enabled;
        let unused = State { leaf: Leaf { value: value, gain: gain, tag: tag, enabled: enabled }, count: state.count }; return state.count;"
    );
    let prefix = helper(&mut module).body[..4].to_vec();
    let body = normalized(&mut module, 65_536).unwrap();
    assert_eq!(&body[..4], prefix);
    assert_eq!(body.len(), 5);
    helper(&mut module).body = body;
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

#[test]
fn stored_projection_requires_exact_fields_declared_kinds_and_completed_catalog() {
    for mutation in 0..7 {
        let mut module = module(BODY);
        let NirStmt::Let {
            ty,
            value: NirExpr::FieldAccess { field, .. },
            ..
        } = &mut helper(&mut module).body[0]
        else {
            panic!()
        };
        match mutation {
            0 => *field = "missing".into(),
            1 => *ty = Some(scalar_type("Layer")),
            2 => {
                module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "make")
                    .unwrap()
                    .is_async = true
            }
            3 => {
                module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "make")
                    .unwrap()
                    .return_type
                    .as_mut()
                    .unwrap()
                    .is_ref = true
            }
            4 => {
                module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "make")
                    .unwrap()
                    .return_type
                    .as_mut()
                    .unwrap()
                    .is_optional = true
            }
            5 => module
                .functions
                .iter_mut()
                .find(|f| f.name == "make")
                .unwrap()
                .body
                .insert(0, NirStmt::Print(NirExpr::Int(1))),
            6 => {
                let make = module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "make")
                    .unwrap();
                make.body = vec![NirStmt::Return(Some(NirExpr::Call {
                    callee: "make".into(),
                    args: vec![NirExpr::Var("input".into())],
                }))];
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
fn stored_projection_does_not_authorize_unbound_inline_projections_or_scalar_mode() {
    for body in [
        "let snapshot = State { leaf: other.leaf, count: make(Input { used: state.count, leaf: other.leaf }).payload.state.count }; return snapshot.count;",
        "let snapshot = State { leaf: make(Input { used: state.count, leaf: other.leaf }).payload.state.leaf, count: state.count }; return snapshot.count;",
    ] {
        let mut module = module(body);
        let before = helper(&mut module).body.clone();
        assert_eq!(normalized(&mut module, 65_536).unwrap(), before);
    }
    let mut module = module(BODY);
    let before = module.clone();
    let layouts = control_values::TypedLayouts::collect(&module);
    let catalog = scalar_helpers::collect_capture_values(&module, &layouts);
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
fn stored_projection_keeps_lexical_version_control_and_transport_exclusions() {
    for body in [
        format!("let state = other; {BODY}"),
        BODY.replace("let snapshot", "let first = later; let snapshot"),
    ] {
        let mut module = module(&body);
        let before = helper(&mut module).body.clone();
        assert_eq!(normalized(&mut module, 65_536).unwrap(), before);
    }
    let mut module = module(BODY);
    let first = helper(&mut module).body.remove(0);
    helper(&mut module).body.insert(
        0,
        NirStmt::If {
            condition: NirExpr::Var("flag".into()),
            then_body: vec![first],
            else_body: vec![],
        },
    );
    let before = helper(&mut module).body.clone();
    assert_eq!(normalized(&mut module, 65_536).unwrap(), before);
    for boundary in ["control", "State", "Layer", "Envelope"] {
        let mut module = self::module(BODY);
        let layouts = control_values::TypedLayouts::collect(&module);
        let catalog = scalar_helpers::collect_capture_values(&module, &layouts);
        let before = helper(&mut module).body.clone();
        let body = normalized_mode(
            helper(&mut module),
            &layouts,
            Some(Evaluated {
                catalog: &catalog,
                controls: &if boundary == "control" {
                    BTreeSet::from(["first".into()])
                } else {
                    BTreeSet::new()
                },
                transport_types: &if boundary == "control" {
                    BTreeSet::new()
                } else {
                    BTreeSet::from([boundary.into()])
                },
                call_results: true,
            }),
            65_536,
        )
        .unwrap();
        assert_eq!(body, before, "{boundary}");
    }
}

#[test]
fn stored_projection_scalar_reads_cannot_strip_protected_parent_kinds() {
    let source = "let scalar = make(Input { used: state.count, leaf: other.leaf }).payload.state.leaf.value;
        let leaf = Leaf { value: scalar, gain: other.leaf.gain, tag: other.leaf.tag, enabled: other.leaf.enabled };
        let saved = State { leaf: leaf, count: state.count }; return saved.count;";
    for protected in ["Envelope", "Layer", "State", "Leaf"] {
        let mut module = module(source);
        let before = helper(&mut module).body.clone();
        let layouts = control_values::TypedLayouts::collect(&module);
        let catalog = scalar_helpers::collect_capture_values(&module, &layouts);
        let normalized = normalized_mode(
            helper(&mut module),
            &layouts,
            Some(Evaluated {
                catalog: &catalog,
                controls: &BTreeSet::new(),
                transport_types: &BTreeSet::from([protected.into()]),
                call_results: true,
            }),
            65_536,
        )
        .unwrap();
        assert_eq!(normalized, before, "{protected}");
    }
}

#[test]
fn stored_projection_keeps_whole_consumers_and_unused_fallible_rhs() {
    for body in [
        "let first = make(Input { used: state.count, leaf: other.leaf }).payload.state; let saved = first; return consume(saved);",
        "let first = make(Input { used: state.count, leaf: other.leaf }).payload.state; return state.count;",
    ] {
        let mut module = module(body);
        let before = helper(&mut module).body.clone();
        assert_eq!(normalized(&mut module, 65_536).unwrap(), before);
    }
}

#[test]
fn stored_projection_budget_and_depth_roll_back_the_complete_candidate() {
    let mut module = module(BODY);
    let expected = normalized(&mut module, 65_536).unwrap();
    let mut accepted = 0;
    let mut rejected = 0;
    for budget in 1..1024 {
        match normalized(&mut module, budget) {
            Some(body) => {
                assert_eq!(body, expected, "{budget}");
                accepted += 1;
            }
            None => rejected += 1,
        }
    }
    assert!(accepted > 0 && rejected > 0);
    let NirStmt::Let { value, .. } = &mut helper(&mut module).body[0] else {
        panic!()
    };
    for _ in 0..70 {
        *value = NirExpr::FieldAccess {
            base: Box::new(value.clone()),
            field: "missing".into(),
        };
    }
    assert!(normalized(&mut module, 65_536).is_none());
}

#[test]
fn stored_projection_preserves_unselected_result_checks_with_an_independent_oracle() {
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
                    struct Envelope {{ selected: Pair, ignored: i64 }}
                    @noinline fn checked(value: i64) -> i64 {{ return 10 / value; }}
                    @noinline fn make(input: Pair) -> Envelope {{ return Envelope {{ selected: input, ignored: checked(input.unused) }}; }}
                    fn helper(state: Pair, count: i64) -> i64 {{
                        if count == 0 {{ return 0; }} {before}
                        let first = make(Pair {{ used: 7, unused: 1 }}).selected;
                        const later: Pair = make(Pair {{ unused: state.unused - count, used: 37 }}).selected;
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
fn stored_projection_requires_every_caller_in_both_function_storage_orders() {
    for computed in [false, true] {
        for reverse in [false, true] {
            let mut module = module(BODY);
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
                            let NirStmt::Let { value, .. } = &target.body[0] else {
                                panic!()
                            };
                            value.clone()
                        } else {
                            NirExpr::Var("state".into())
                        },
                        NirExpr::Var("other".into()),
                        NirExpr::Var("flag".into()),
                    ],
                }))];
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
                assert_eq!(helper(&mut module).body.len(), 3);
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
