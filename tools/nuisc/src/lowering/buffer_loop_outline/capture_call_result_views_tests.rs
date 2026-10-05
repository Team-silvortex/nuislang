use super::*;

fn module(body: &str) -> NirModule {
    crate::frontend::parse_nuis_module(&format!(
        "mod cpu Main {{
        struct Leaf {{ value: f64, gain: f32, tag: i32, enabled: bool }}
        struct State {{ left: Leaf, right: Leaf, count: i64 }}
        @noinline fn checked(value: i64) -> i64 {{ return 10 / value; }}
        @noinline fn make(state: State, other: State) -> State {{
            return State {{ left: state.left, right: other.right, count: checked(state.count) }};
        }}
        fn consume(state: State) -> i64 {{ return state.count; }}
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

fn normalize(module: &mut NirModule) {
    let layouts = control_values::TypedLayouts::collect(module);
    let catalog = scalar_helpers::collect_capture_values(module, &layouts);
    normalize_call_results(
        helper(module),
        &layouts,
        &catalog,
        &BTreeSet::new(),
        &BTreeSet::new(),
    );
    crate::nir_verify::verify_nir_module(module).unwrap();
}

const BODY: &str = "let first = make(state, other); const later: State = make(other, state);
    let snapshot = State { left: first.left, right: later.right, count: first.count };
    let saved = snapshot; return saved.count;";

#[test]
fn call_result_views_reuse_completed_records_without_erasing_or_duplicating_calls() {
    let mut module = module(BODY);
    let original = helper(&mut module).body[..2].to_vec();
    normalize(&mut module);
    let body = &helper(&mut module).body;
    assert_eq!(&body[..2], original);
    assert_eq!(body.len(), 3);
    assert_eq!(
        body[2],
        NirStmt::Return(Some(NirExpr::FieldAccess {
            base: Box::new(NirExpr::Var("first".into())),
            field: "count".into(),
        }))
    );
    let before = module.clone();
    normalize(&mut module);
    assert_eq!(module, before);
}

#[test]
fn call_result_views_keep_unused_calls_and_exact_mixed_field_kinds() {
    let mut module = module(
        "let returned = make(state, other); const unused: State = make(other, state);
         let value = returned.left.value; let gain = returned.left.gain;
         let tag = returned.left.tag; let enabled = returned.left.enabled;
         let leaf = Leaf { value: value, gain: gain, tag: tag, enabled: enabled };
         let copied = unused; return returned.count;",
    );
    let original = helper(&mut module).body[..6].to_vec();
    normalize(&mut module);
    let body = &helper(&mut module).body;
    assert_eq!(&body[..6], original);
    assert_eq!(body.len(), 7);
}

#[test]
fn call_result_views_do_not_grant_whole_escape_inline_call_or_changed_version_authority() {
    for body in [
        "let returned = make(state, other); let saved = returned; return consume(saved);",
        "let returned = make(state, other); let returned = make(other, state); let saved = returned; return saved.count;",
        "let returned = make(state, other); if flag { let returned = make(other, state); } let saved = returned; return saved.count;",
        "let snapshot = State { left: make(state, other).left, right: other.right, count: state.count }; return snapshot.count;",
    ] {
        let mut module = module(body);
        let before = module.clone();
        normalize(&mut module);
        assert_eq!(module, before, "{body}");
    }
}

#[test]
fn call_result_views_require_completed_nonrecursive_owned_value_catalog() {
    for mutation in 0..5 {
        let mut module = module(BODY);
        let make = module
            .functions
            .iter_mut()
            .find(|f| f.name == "make")
            .unwrap();
        match mutation {
            0 => make.is_async = true,
            1 => make.return_type.as_mut().unwrap().is_ref = true,
            2 => make.return_type.as_mut().unwrap().is_optional = true,
            3 => make.body.insert(0, NirStmt::Print(NirExpr::Int(1))),
            4 => {
                make.body = vec![NirStmt::Return(Some(NirExpr::Call {
                    callee: "make".into(),
                    args: vec![NirExpr::Var("state".into()), NirExpr::Var("other".into())],
                }))]
            }
            _ => unreachable!(),
        }
        let before = module.clone();
        let layouts = control_values::TypedLayouts::collect(&module);
        let catalog = scalar_helpers::collect_capture_values(&module, &layouts);
        assert!(!catalog.contains_key("make"));
        normalize_call_results(
            helper(&mut module),
            &layouts,
            &catalog,
            &BTreeSet::new(),
            &BTreeSet::new(),
        );
        assert_eq!(module, before, "mutation {mutation}");
    }
}

#[test]
fn call_result_views_require_nominal_kinds_control_wire_and_budget_proofs() {
    for mutation in 0..6 {
        let mut module = module(BODY);
        let mut controls = BTreeSet::new();
        let mut transport = BTreeSet::new();
        match mutation {
            0 => {
                if let NirStmt::Let { ty, .. } = &mut helper(&mut module).body[0] {
                    *ty = Some(scalar_type("Leaf"));
                }
            }
            1 => {
                if let NirStmt::Let {
                    value: NirExpr::Call { callee, .. },
                    ..
                } = &mut helper(&mut module).body[0]
                {
                    *callee = "unknown".into();
                }
            }
            2 => {
                controls.extend(["first".into(), "later".into()]);
            }
            3 => {
                transport.insert("State".into());
            }
            4 => {
                if let NirStmt::Let {
                    value: NirExpr::Call { args, .. },
                    ..
                } = &mut helper(&mut module).body[0]
                {
                    args[0] = NirExpr::Int(0);
                }
            }
            5 => {}
            _ => unreachable!(),
        }
        let before = module.clone();
        let layouts = control_values::TypedLayouts::collect(&module);
        let catalog = scalar_helpers::collect_capture_values(&module, &layouts);
        let result = normalized_mode(
            helper(&mut module),
            &layouts,
            Some(Evaluated {
                catalog: &catalog,
                controls: &controls,
                transport_types: &transport,
                call_results: true,
            }),
            if mutation == 5 { 10 } else { 65_536 },
        );
        if let Some(body) = result {
            helper(&mut module).body = body;
        }
        assert_eq!(module, before, "mutation {mutation}");
    }
}

#[test]
fn call_result_views_never_export_child_roots_or_read_results_before_definition() {
    let mut module = module(BODY);
    let first = helper(&mut module).body.remove(0);
    helper(&mut module).body.insert(2, first);
    let before = module.clone();
    let layouts = control_values::TypedLayouts::collect(&module);
    let catalog = scalar_helpers::collect_capture_values(&module, &layouts);
    normalize_call_results(
        helper(&mut module),
        &layouts,
        &catalog,
        &BTreeSet::new(),
        &BTreeSet::new(),
    );
    assert_eq!(module, before);

    let mut module = self::module(
        "while flag { let returned = make(state, other); let copied = returned;
            if copied.count > 0 { break; } continue; } return state.count;",
    );
    normalize(&mut module);
    let body = &helper(&mut module).body;
    let NirStmt::While {
        body: iteration, ..
    } = &body[0]
    else {
        panic!()
    };
    assert_eq!(iteration.len(), 3);
    assert!(
        matches!(&iteration[0], NirStmt::Let { value: NirExpr::Call { callee, .. }, .. } if callee == "make")
    );
    assert_eq!(body.len(), 2);
}

#[test]
fn call_result_views_bound_depth_and_never_commit_a_budget_partial_body() {
    let mut module = module(BODY);
    let layouts = control_values::TypedLayouts::collect(&module);
    let catalog = scalar_helpers::collect_capture_values(&module, &layouts);
    let controls = BTreeSet::new();
    let transport = BTreeSet::new();
    let evaluate = |function: &NirFunction, budget| {
        normalized_mode(
            function,
            &layouts,
            Some(Evaluated {
                catalog: &catalog,
                controls: &controls,
                transport_types: &transport,
                call_results: true,
            }),
            budget,
        )
    };
    let expected = evaluate(helper(&mut module), 65_536).unwrap();
    let mut rolled_back = 0;
    let mut completed = 0;
    for budget in 1..512 {
        if let Some(body) = evaluate(helper(&mut module), budget) {
            assert_eq!(body, expected, "budget {budget}");
            completed += 1;
        } else {
            rolled_back += 1;
        }
    }
    assert!(rolled_back > 0 && completed > 0);
    let mut value = NirExpr::Int(0);
    for _ in 0..70 {
        value = NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(value),
            rhs: Box::new(NirExpr::Int(1)),
        };
    }
    if let NirStmt::Let {
        value: NirExpr::Call { args, .. },
        ..
    } = &mut helper(&mut module).body[0]
    {
        args[0] = value;
    }
    assert!(evaluate(helper(&mut module), 65_536).is_none());
}

#[test]
fn call_result_views_project_wide_captures_transactionally_without_changing_result_layout() {
    let definition = (0..64)
        .map(|i| format!("f{i}: i64"))
        .collect::<Vec<_>>()
        .join(", ");
    let produced = (0..64)
        .map(|i| format!("f{i}: {}", if i == 62 { "unused" } else { "value" }))
        .collect::<Vec<_>>()
        .join(", ");
    let fields = (0..64)
        .map(|i| {
            format!(
                "f{i}: {}",
                match i {
                    0 => "first.f0".into(),
                    62 => "later.f62".into(),
                    _ => format!("value.f{i}"),
                }
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    for computed in [false, true] {
        for reverse in [false, true] {
            let suffix = if computed { " + 0" } else { "" };
            let operand = if computed {
                "produce(value.f0, value.f62)"
            } else {
                "value"
            };
            let mut module = crate::frontend::parse_nuis_module(&format!(
                "mod cpu Main {{ struct Payload {{ {definition} }}
                @noinline fn produce(value: i64, unused: i64) -> Payload {{ return Payload {{ {produced} }}; }}
                fn helper(value: Payload) -> i64 {{ let first = produce(value.f0, value.f62);
                    const later: Payload = produce(value.f0 + 30, value.f62);
                    let snapshot = Payload {{ {fields} }}; let saved = snapshot; return saved.f0; }}
                fn entry(value: Payload) -> i64 {{ return helper(value); }}
                fn second(value: Payload) -> i64 {{ return helper({operand}){suffix}; }} }}"
            )).unwrap();
            if reverse {
                module.functions.reverse();
            }
            let before = module.clone();
            let producer = module
                .functions
                .iter()
                .find(|f| f.name == "produce")
                .unwrap()
                .clone();
            let layouts = control_values::TypedLayouts::collect(&module);
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
                assert_eq!(helper(&mut module).params.len(), 2);
                assert_eq!(helper(&mut module).body.len(), 3);
                assert_eq!(
                    module
                        .functions
                        .iter()
                        .find(|f| f.name == "produce")
                        .unwrap(),
                    &producer
                );
                crate::nir_verify::verify_nir_module(&module).unwrap();
            }
        }
    }
}

#[test]
fn call_result_views_preserve_selected_checks_and_independent_results_differentially() {
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
                    @noinline fn relay(value: i64) -> i64 {{ return value; }}
                    @noinline fn checked(value: i64) -> i64 {{ return 10 / value; }}
                    @noinline fn make(value: i64, divisor: i64) -> Pair {{
                        return Pair {{ used: relay(value), unused: checked(divisor) }};
                    }} fn helper(state: Pair, other: Pair, count: i64) -> i64 {{
                        if count == 0 {{ return 0; }} {before}
                        let first = make(state.used, other.unused - state.unused);
                        const later: Pair = make(state.used + 30, 1);
                        let snapshot = Pair {{ used: first.used, unused: later.unused }};
                        let saved = snapshot; {after} return saved.used;
                    }} fn main() -> i64 {{ return helper(Pair {{ used: 7, unused: {count} }},
                        Pair {{ used: 99, unused: {divisor} }}, {count}); }} }}"
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
                assert_eq!(
                    execute(&module),
                    expected,
                    "before {count}/{divisor}/{position}"
                );
                let original = module.clone();
                normalize(&mut module);
                assert_ne!(module, original);
                assert_eq!(
                    execute(&module),
                    expected,
                    "after {count}/{divisor}/{position}"
                );
            }
        }
    }
}
