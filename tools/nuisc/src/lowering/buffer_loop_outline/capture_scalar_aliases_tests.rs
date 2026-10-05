use super::*;

fn module(body: &str, callers: &str) -> NirModule {
    crate::frontend::parse_nuis_module(&format!(
        "mod cpu Main {{
        struct Leaf {{ value: f64, gain: f32, tag: i32, enabled: bool }}
        struct State {{ left: Leaf, right: Leaf, count: i64 }}
        @noinline fn checked(value: i64) -> i64 {{ return 10 / value; }}
        @noinline fn helper(state: State, other: State, flag: bool) -> i64 {{ {body} }}
        {callers}
    }}"
    ))
    .unwrap()
}

fn function<'a>(module: &'a NirModule, name: &str) -> &'a NirFunction {
    module
        .functions
        .iter()
        .find(|function| function.name == name)
        .unwrap()
}

fn normalize_and_verify(module: &mut NirModule) {
    let layouts = control_values::TypedLayouts::collect(module);
    let function = module
        .functions
        .iter_mut()
        .find(|f| f.name == "helper")
        .unwrap();
    normalize(function, &layouts, &BTreeSet::new());
    crate::nir_verify::verify_nir_module(module).unwrap();
}

#[test]
fn private_scalar_aliases_inline_exact_input_paths_and_chains() {
    let mut module = module(
        "let count = state.count; const saved: i64 = count; let selected = saved;
         let tag: i32 = state.left.tag; let gain: f32 = state.left.gain;
         let value: f64 = state.right.value; let enabled: bool = state.left.enabled;
         return selected;",
        "",
    );
    normalize_and_verify(&mut module);
    let helper = function(&module, "helper");
    assert_eq!(helper.body.len(), 1);
    assert!(!format!("{:?}", helper.body).contains("selected"));
    assert!(format!("{:?}", helper.body).contains("count"));
    let before = module.clone();
    normalize_and_verify(&mut module);
    assert_eq!(module, before);
}

#[test]
fn private_scalar_aliases_do_not_promote_computed_local_codec_or_changed_versions() {
    for body in [
        "let count = state.count + 0; let saved = count; return saved;",
        "let count = checked(state.count); let saved = count; return saved;",
        "let count = i32_from_i64(state.count); let saved = count; return state.count;",
        "let local = state; let count = local.count; return count;",
        "let count = state.count; let state = other; return count;",
        "let count = state.count; if flag { let count = other.count; } return count;",
        "let count = state.count; while flag { let state = other; break; } return count;",
        "let flag = flag; return state.count;",
    ] {
        let mut module = module(body, "");
        let before = module.clone();
        normalize_and_verify(&mut module);
        assert_eq!(module, before, "{body}");
    }
}

#[test]
fn private_scalar_aliases_keep_branch_and_loop_locals_lexical() {
    let mut module = module(
        "let count = state.count;
         if flag { let selected = state.count; return count + selected; }
         while other.count > 0 { let saved = other.count; if saved > 2 { break; } continue; }
         return count;",
        "",
    );
    normalize_and_verify(&mut module);
    let helper = function(&module, "helper");
    assert_eq!(helper.body.len(), 3);
    let NirStmt::If { then_body, .. } = &helper.body[0] else {
        panic!()
    };
    assert_eq!(then_body.len(), 1);
    let NirStmt::While { body, .. } = &helper.body[1] else {
        panic!()
    };
    assert_eq!(body.len(), 2);
    assert!(!format!("{:?}", helper.body).contains("saved"));
}

#[test]
fn private_scalar_aliases_keep_types_qualifiers_and_shadowing_conservative() {
    for mutation in 0..7 {
        let mut module = module("let count: i64 = state.count; return count;", "");
        let function = module
            .functions
            .iter_mut()
            .find(|f| f.name == "helper")
            .unwrap();
        match mutation {
            0 => function.params[0].ty.is_ref = true,
            1 => function.params[0].ty.is_optional = true,
            2 => function.params[0].ty.name = "Buffer".into(),
            3 => {
                let NirStmt::Let { ty, .. } = &mut function.body[0] else {
                    panic!()
                };
                *ty = Some(scalar_type("i32"));
            }
            4 => {
                let NirStmt::Let { value, .. } = &mut function.body[0] else {
                    panic!()
                };
                *value = NirExpr::FieldAccess {
                    base: Box::new(NirExpr::Var("state".into())),
                    field: "missing".into(),
                };
            }
            5 => function.params[0].name = "count".into(),
            6 => function.params.push(function.params[0].clone()),
            _ => unreachable!(),
        }
        let before = module.clone();
        let layouts = control_values::TypedLayouts::collect(&module);
        let function = module
            .functions
            .iter_mut()
            .find(|f| f.name == "helper")
            .unwrap();
        normalize(function, &layouts, &BTreeSet::new());
        assert_eq!(module, before, "mutation {mutation}");
    }
}

#[test]
fn private_scalar_aliases_keep_registered_control_identities() {
    let mut module = module(
        "let count = state.count; let saved = count; return saved;",
        "",
    );
    let layouts = control_values::TypedLayouts::collect(&module);
    let before = module.clone();
    let function = module
        .functions
        .iter_mut()
        .find(|f| f.name == "helper")
        .unwrap();
    normalize(function, &layouts, &BTreeSet::from(["count".into()]));
    assert_eq!(module, before);
}

#[test]
fn private_scalar_aliases_unlock_partial_record_captures_without_erasing_checked_work() {
    for reverse in [false, true] {
        let mut module = module(
            "let count = state.count;
             let snapshot = State { left: state.left, right: state.right, count: count };
             let checked_value = checked(other.count);
             if snapshot.left.tag == i32_from_i64(7) { return checked_value; } return 0;",
            "fn entry(state: State, other: State, flag: bool) -> i64 { return helper(state, other, flag); }"
        );
        if reverse {
            module.functions.reverse();
        }
        let layouts = control_values::TypedLayouts::collect(&module);
        assert!(
            super::super::project(
                &mut module,
                &BTreeSet::from(["helper".into()]),
                &BTreeSet::new(),
                &layouts,
                &BTreeMap::new()
            )
            .changed
        );
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let helper = function(&module, "helper");
        assert_eq!(
            helper
                .params
                .iter()
                .map(|p| p.ty.name.as_str())
                .collect::<Vec<_>>(),
            ["i32", "i64", "bool"]
        );
        assert!(helper.body.iter().any(|stmt| matches!(stmt, NirStmt::Let { value: NirExpr::Call { callee, .. }, .. } if callee == "checked")));
    }
}

#[test]
fn private_scalar_aliases_remain_transactional_when_any_caller_is_computed() {
    let mut module = module(
        "let count = state.count; let snapshot = State { left: state.left, right: state.right, count: count }; if snapshot.left.tag == i32_from_i64(7) { return 1; } return 0;",
        "fn relay(state: State) -> State { return state; }
         fn entry(state: State, other: State, flag: bool) -> i64 { return helper(state, other, flag); }
         fn veto(state: State, other: State, flag: bool) -> i64 { return 0 + helper(relay(state), other, flag); }"
    );
    let before = module.clone();
    let layouts = control_values::TypedLayouts::collect(&module);
    super::super::project(
        &mut module,
        &BTreeSet::from(["helper".into()]),
        &BTreeSet::new(),
        &layouts,
        &BTreeMap::new(),
    );
    assert_eq!(module, before);
}

#[test]
fn private_scalar_aliases_preserve_guarded_returns_and_unused_checked_work() {
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
                        if state.count == 0 {{ return 0; }} {before_guard}
                        let unused = state.left.unused; const saved: i64 = unused;
                        let snapshot = State {{
                            left: Leaf {{ used: state.left.used, unused: saved }},
                            right: state.right, count: checked(other.count - state.count)
                        }};
                        {after_guard} return snapshot.left.used;
                    }}
                    fn main() -> i64 {{ return helper(
                        State {{ left: Leaf {{ used: 7, unused: 99 }}, right: Leaf {{ used: 101, unused: 103 }}, count: {count} }},
                        State {{ left: Leaf {{ used: 0, unused: 0 }}, right: Leaf {{ used: 0, unused: 0 }}, count: {divisor} }});
                    }} }}"
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
                normalize_and_verify(&mut module);
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

#[test]
fn private_scalar_aliases_reduce_64_input_fields_without_widening_any_caller() {
    let definitions = (0..64)
        .map(|i| format!("f{i}: i64"))
        .collect::<Vec<_>>()
        .join(", ");
    let copies = (0..64)
        .map(|i| format!("let copy{i} = value.f{i}; const saved{i}: i64 = copy{i};"))
        .collect::<Vec<_>>()
        .join(" ");
    let fields = (0..64)
        .map(|i| format!("f{i}: saved{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    for reverse in [false, true] {
        let mut module = crate::frontend::parse_nuis_module(&format!(
            "mod cpu Main {{ struct Payload {{ {definitions} }}
             fn helper(value: Payload) -> i64 {{ {copies} let snapshot = Payload {{ {fields} }}; return snapshot.f0; }}
             fn entry(value: Payload) -> i64 {{ return helper(value); }} }}"
        )).unwrap();
        if reverse {
            module.functions.reverse();
        }
        let layouts = control_values::TypedLayouts::collect(&module);
        // Every input leaf is read through the aliases before normalization;
        // the sparse planner must conservatively keep the full 64-word record.
        assert!(super::super::plan(function(&module, "helper"), None, &layouts).is_none());
        assert!(
            super::super::project(
                &mut module,
                &BTreeSet::from(["helper".into()]),
                &BTreeSet::new(),
                &layouts,
                &BTreeMap::new()
            )
            .changed
        );
        crate::nir_verify::verify_nir_module(&module).unwrap();
        assert_eq!(function(&module, "helper").params.len(), 1);
        assert_eq!(function(&module, "helper").body.len(), 1);
        assert_eq!(function(&module, "entry").params[0].ty.name, "Payload");
    }
}

#[test]
fn private_scalar_aliases_reject_exhausted_and_deep_work_without_partial_rewrite() {
    let mut module = module(
        "let count = state.count; const saved: i64 = count; return saved;",
        "",
    );
    let layouts = control_values::TypedLayouts::collect(&module);
    let helper = function(&module, "helper");
    let accepted = normalized(helper, &layouts, &BTreeSet::new(), 65_536).unwrap();
    for budget in 0..128 {
        if let Some(body) = normalized(helper, &layouts, &BTreeSet::new(), budget) {
            assert_eq!(body, accepted);
        }
    }
    let function = module
        .functions
        .iter_mut()
        .find(|f| f.name == "helper")
        .unwrap();
    let mut body = function.body.clone();
    for _ in 0..64 {
        body = vec![NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: body,
            else_body: Vec::new(),
        }];
    }
    function.body = body;
    let before = module.clone();
    let function = module
        .functions
        .iter_mut()
        .find(|f| f.name == "helper")
        .unwrap();
    normalize(function, &layouts, &BTreeSet::new());
    assert_eq!(module, before);

    // Substitution must also respect the expression ceiling after expansion.
    module = self::module("let count = state.left.tag; return 0;", "");
    let function = module
        .functions
        .iter_mut()
        .find(|f| f.name == "helper")
        .unwrap();
    function.return_type = Some(scalar_type("i32"));
    let mut value = NirExpr::Var("count".into());
    for _ in 0..31 {
        value = NirExpr::CastI64ToI32(Box::new(NirExpr::CastI32ToI64(Box::new(value))));
    }
    *function.body.last_mut().unwrap() = NirStmt::Return(Some(value));
    crate::nir_verify::verify_nir_module(&module).unwrap();
    let before = module.clone();
    let function = module
        .functions
        .iter_mut()
        .find(|f| f.name == "helper")
        .unwrap();
    normalize(function, &layouts, &BTreeSet::new());
    assert_eq!(module, before);

    module = self::module(
        "let count = state.count; print(\"blocked\"); return count;",
        "",
    );
    let before = module.clone();
    let function = module
        .functions
        .iter_mut()
        .find(|f| f.name == "helper")
        .unwrap();
    normalize(function, &layouts, &BTreeSet::new());
    assert_eq!(module, before);
}
