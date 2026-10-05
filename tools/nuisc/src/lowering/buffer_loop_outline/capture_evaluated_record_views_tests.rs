use super::*;

fn module(body: &str, callers: &str) -> NirModule {
    crate::frontend::parse_nuis_module(&format!(
        "mod cpu Main {{
        struct Leaf {{ value: f64, gain: f32, tag: i32, enabled: bool }}
        struct State {{ left: Leaf, right: Leaf, count: i64 }}
        @noinline fn checked(value: i64) -> i64 {{ return 10 / value; }}
        fn relay(state: State) -> State {{ return state; }}
        fn consume(state: State) -> i64 {{ return state.count; }}
        fn helper(state: State, other: State, flag: bool) -> i64 {{ {body} }}
        {callers} }}"
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
    let catalog = scalar_helpers::collect(module);
    normalize_evaluated(
        helper(module),
        &layouts,
        &catalog,
        &BTreeSet::new(),
        &BTreeSet::new(),
    );
    crate::nir_verify::verify_nir_module(module).unwrap();
}

#[test]
fn evaluated_record_views_reuse_local_calls_and_codecs_without_erasing_definitions() {
    let mut module = module(
        "let count = checked(state.count); const spare: i64 = checked(other.count);
         let tag = i32_from_i64(spare);
         let snapshot = State { left: Leaf { value: state.left.value, gain: state.left.gain,
             tag: tag, enabled: state.left.enabled }, right: other.right, count: count };
         let saved = snapshot; return saved.count;",
        "",
    );
    let original = helper(&mut module).body[..3].to_vec();
    normalize(&mut module);
    let body = &helper(&mut module).body;
    assert_eq!(&body[..3], original);
    assert_eq!(body.len(), 4);
    assert_eq!(body[3], NirStmt::Return(Some(NirExpr::Var("count".into()))));
    let before = module.clone();
    normalize(&mut module);
    assert_eq!(module, before);
}

#[test]
fn evaluated_record_views_preserve_exact_mixed_scalar_kinds_and_signed_zero() {
    let mut module = module(
        "let value: f64 = -0.0; let gain: f32 = state.left.gain + state.right.gain;
         let tag: i32 = i32_from_i64(state.count); const enabled: bool = flag;
         let snapshot = State { left: Leaf { value: value, gain: gain, tag: tag, enabled: enabled },
             right: other.right, count: state.count };
         let observed = snapshot.left.value; return snapshot.count;",
        "",
    );
    // Also exercise a typed YIR-origin literal rather than only source unary negation.
    if let NirStmt::Let { value, .. } = &mut helper(&mut module).body[0] {
        *value = NirExpr::F64("-0.0".into());
    }
    let original = helper(&mut module).body[..4].to_vec();
    normalize(&mut module);
    let body = &helper(&mut module).body;
    assert_eq!(&body[..4], original);
    assert_eq!(body.len(), 6);
    assert!(matches!(&body[4], NirStmt::Let { value: NirExpr::Var(name), .. } if name == "value"));
    assert!(format!("{body:?}").contains("-0.0"));
}

#[test]
fn evaluated_record_views_keep_whole_transport_and_opaque_or_changed_versions() {
    for body in [
        "let count = checked(state.count); let snapshot = State { left: state.left, right: other.right, count: count }; return consume(snapshot);",
        "let count = checked(state.count); let count = checked(other.count); let snapshot = State { left: state.left, right: other.right, count: count }; return snapshot.count;",
        "let snapshot = State { left: state.left, right: other.right, count: checked(state.count) }; return snapshot.count;",
        "let local = relay(state); let snapshot = State { left: local.left, right: other.right, count: local.count }; return snapshot.count;",
        "let count = checked(state.count); if flag { let count = other.count; } let snapshot = State { left: state.left, right: other.right, count: count }; return snapshot.count;",
        "let count = checked(state.count); while flag { let count = checked(other.count); break; } let snapshot = State { left: state.left, right: other.right, count: count }; return snapshot.count;",
    ] {
        let mut module = module(body, "");
        let before = module.clone();
        let layouts = control_values::TypedLayouts::collect(&module);
        let catalog = scalar_helpers::collect(&module);
        normalize_evaluated(helper(&mut module), &layouts, &catalog, &BTreeSet::new(), &BTreeSet::new());
        assert_eq!(module, before, "{body}");
    }
}

#[test]
fn evaluated_record_views_require_types_control_identity_transport_and_budget() {
    for mutation in 0..7 {
        let mut module = module(
            "let count: i64 = checked(state.count); let snapshot: State = State {
             left: state.left, right: other.right, count: count }; return snapshot.count;",
            "",
        );
        let mut controls = BTreeSet::new();
        let mut transport = BTreeSet::new();
        match mutation {
            0 => {
                if let NirStmt::Let { ty, .. } = &mut helper(&mut module).body[0] {
                    *ty = Some(scalar_type("i32"));
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
                controls.insert("count".into());
            }
            3 => {
                transport.insert("State".into());
            }
            4 => helper(&mut module).params[0].ty.is_ref = true,
            5 => {
                let param = helper(&mut module).params[0].clone();
                helper(&mut module).params.push(param);
            }
            6 => {}
            _ => unreachable!(),
        }
        let before = module.clone();
        let layouts = control_values::TypedLayouts::collect(&module);
        let catalog = scalar_helpers::collect(&module);
        let result = normalized_mode(
            helper(&mut module),
            &layouts,
            Some(Evaluated {
                catalog: &catalog,
                controls: &controls,
                transport_types: &transport,
                call_results: false,
            }),
            if mutation == 6 { 10 } else { 65_536 },
        );
        if let Some(body) = result {
            helper(&mut module).body = body;
        }
        assert_eq!(module, before, "mutation {mutation}");
    }
}

#[test]
fn evaluated_record_views_keep_loop_local_evaluation_lexical() {
    let mut module = module(
        "let outer = checked(state.count);
         while flag { let inner = checked(other.count);
             let inner_snapshot = State { left: state.left, right: other.right, count: inner };
             if inner_snapshot.count > 0 { break; } continue;
         } let outer_snapshot = State { left: state.left, right: other.right, count: outer };
         return outer_snapshot.count;",
        "",
    );
    normalize(&mut module);
    let body = &helper(&mut module).body;
    assert_eq!(body.len(), 3);
    let NirStmt::While {
        body: iteration, ..
    } = &body[1]
    else {
        panic!()
    };
    assert_eq!(iteration.len(), 3);
    assert!(
        matches!(&iteration[0], NirStmt::Let { name, value: NirExpr::Call { .. }, .. } if name == "inner")
    );
    assert_eq!(body[2], NirStmt::Return(Some(NirExpr::Var("outer".into()))));
}

#[test]
fn evaluated_record_views_reject_deep_rhs_without_installing_partial_views() {
    let mut module = module(
        "let count = checked(state.count); let snapshot = State {
            left: state.left, right: other.right, count: count }; return snapshot.count;",
        "",
    );
    let mut value = NirExpr::Int(0);
    for _ in 0..70 {
        value = NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(value),
            rhs: Box::new(NirExpr::Int(1)),
        };
    }
    if let NirStmt::Let { value: rhs, .. } = &mut helper(&mut module).body[0] {
        *rhs = value;
    }
    let before = module.clone();
    let layouts = control_values::TypedLayouts::collect(&module);
    normalize_evaluated(
        helper(&mut module),
        &layouts,
        &BTreeMap::new(),
        &BTreeSet::new(),
        &BTreeSet::new(),
    );
    assert_eq!(module, before);
}

#[test]
fn evaluated_record_views_project_64_fields_only_after_every_caller_validates() {
    let definition = (0..64)
        .map(|i| format!("f{i}: i64"))
        .collect::<Vec<_>>()
        .join(", ");
    let fields = (0..64)
        .map(|i| {
            format!(
                "f{i}: {}",
                match i {
                    0 => "selected".into(),
                    62 => "unused".into(),
                    _ => format!("value.f{i}"),
                }
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    for computed in [false, true] {
        for reverse in [false, true] {
            let veto = if computed { "relay(value)" } else { "value" };
            let suffix = if computed { " + 0" } else { "" };
            let mut module = crate::frontend::parse_nuis_module(&format!(
                "mod cpu Main {{ struct Payload {{ {definition} }}
                @noinline fn checked(value: i64) -> i64 {{ return 10 / value; }}
                fn relay(value: Payload) -> Payload {{ return value; }}
                fn helper(value: Payload) -> i64 {{ let selected = checked(value.f0);
                    const unused: i64 = checked(value.f62);
                    let snapshot = Payload {{ {fields} }}; return snapshot.f0; }}
                fn entry(value: Payload) -> i64 {{ return helper(value); }}
                fn second(value: Payload) -> i64 {{ return helper({veto}){suffix}; }} }}"
            ))
            .unwrap();
            if reverse {
                module.functions.reverse();
            }
            let before = module.clone();
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
                let helper = helper(&mut module);
                assert_eq!(helper.params.len(), 2);
                assert!(helper.params.iter().all(|p| p.ty == scalar_type("i64")));
                assert_eq!(helper.body.len(), 3);
                crate::nir_verify::verify_nir_module(&module).unwrap();
            }
        }
    }
}

#[test]
fn evaluated_record_views_preserve_selected_failures_with_an_independent_oracle() {
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
                    "mod cpu Main {{ struct State {{ used: i64, unused: i64 }}
                    @noinline fn relay(value: i64) -> i64 {{ return value; }}
                    @noinline fn checked(value: i64) -> i64 {{ return 10 / value; }}
                    fn helper(state: State, other: State, count: i64) -> i64 {{
                        if count == 0 {{ return 0; }} {before}
                        let selected = relay(state.used);
                        const unused: i64 = checked(other.unused - state.unused);
                        let snapshot = State {{ used: selected, unused: unused }};
                        let saved = snapshot; {after} return saved.used;
                    }} fn main() -> i64 {{ return helper(State {{ used: 7, unused: {count} }},
                        State {{ used: 99, unused: {divisor} }}, {count}); }} }}"
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
