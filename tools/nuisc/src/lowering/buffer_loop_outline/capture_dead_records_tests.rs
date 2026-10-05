use super::*;

fn module(body: &str) -> NirModule {
    module_with_callers(body, "")
}

fn module_with_callers(body: &str, callers: &str) -> NirModule {
    crate::frontend::parse_nuis_module(&format!(
        "mod cpu Main {{
        struct Leaf {{ value: f64, gain: f32, tag: i32, enabled: bool }}
        struct State {{ left: Leaf, right: Leaf, count: i64 }}
        fn relay(state: State) -> State {{ return state; }}
        fn helper(state: State, other: State) -> i64 {{ {body} }}
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

const COPY: &str = "State { left: Leaf { value: state.left.value,
    gain: state.left.gain, tag: state.left.tag, enabled: state.left.enabled },
    right: other.right, count: state.count }";

#[test]
fn dead_record_snapshots_drop_only_unobserved_ready_reconstructions() {
    for binding in ["let unused =", "const unused: State ="] {
        for nested in [false, true] {
            let snapshot = format!("{binding} {COPY};");
            let body = if nested {
                format!(
                    "let i = 0; while i < state.count {{
                    if i > 0 {{ {snapshot} }} let i = i + 1;
                }} return 0;"
                )
            } else {
                format!("{snapshot} return 0;")
            };
            let mut module = module(&body);
            let layouts = control_values::TypedLayouts::collect(&module);
            assert_eq!(
                candidates(helper(&mut module), &layouts, 65_536),
                Some(BTreeSet::from(["unused".into()]))
            );
            normalize(helper(&mut module), &layouts);
            let mut writes = BTreeSet::new();
            branches::collect_bindings(&helper(&mut module).body, &mut writes);
            assert!(!writes.contains("unused"));
            let before = module.clone();
            normalize(helper(&mut module), &layouts);
            assert_eq!(module, before);
        }
    }
}

#[test]
fn dead_record_snapshots_preserve_live_writes_shadowed_names_and_local_sources() {
    for body in [
        format!("let unused = {COPY}; return unused.count;"),
        format!("let unused = {COPY}; let unused = other; return 0;"),
        format!(
            "if state.count > 0 {{ let unused = {COPY}; }}
            else {{ let unused = other; }} return 0;"
        ),
        format!("let state = {COPY}; return 0;"),
        format!("let state = other; let unused = {COPY}; return 0;"),
        format!(
            "if state.count > 0 {{ let other = state; }}
            let unused = {COPY}; return 0;"
        ),
        "let local = state; let unused = State { left: local.left,
            right: local.right, count: local.count }; return 0;"
            .into(),
        "let unused = state.count; return 0;".into(),
    ] {
        let mut module = module(&body);
        let layouts = control_values::TypedLayouts::collect(&module);
        let before = module.clone();
        normalize(helper(&mut module), &layouts);
        assert_eq!(module, before, "{body}");
    }
}

#[test]
fn dead_record_snapshots_preserve_calls_checked_fields_and_codecs() {
    for value in [
        "relay(state)".to_owned(),
        COPY.replace("state.count", "state.count / (state.count - state.count)"),
        COPY.replace("state.count", "state.count + other.count"),
        COPY.replace("state.left.tag", "i32_from_i64(state.count)"),
    ] {
        let mut module = module(&format!("let unused = {value}; return 0;"));
        let layouts = control_values::TypedLayouts::collect(&module);
        let before = module.clone();
        normalize(helper(&mut module), &layouts);
        assert_eq!(module, before, "{value}");
    }
}

#[test]
fn dead_record_snapshots_require_exact_nominal_field_and_binding_types() {
    for mutation in 0..7 {
        let mut module = module(&format!("let unused: State = {COPY}; return 0;"));
        let layouts = control_values::TypedLayouts::collect(&module);
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
            1 => fields[2].1 = fields[1].1.clone(),
            2 => fields[2].0 = "left".into(),
            3 => fields[2].0 = "missing".into(),
            4 => {
                fields[2].1 = NirExpr::FieldAccess {
                    base: Box::new(NirExpr::Var("state".into())),
                    field: "missing".into(),
                }
            }
            5 => *ty = Some(scalar_type("Leaf")),
            6 => type_args.push(scalar_type("i64")),
            _ => unreachable!(),
        }
        let before = module.clone();
        normalize(helper(&mut module), &layouts);
        assert_eq!(module, before, "{mutation}");
    }
}

#[test]
fn dead_record_snapshots_do_not_treat_resources_or_borrows_as_ready_values() {
    for kind in 0..3 {
        let mut module = module(&format!("let unused = {COPY}; return 0;"));
        let ty = &mut helper(&mut module).params[0].ty;
        match kind {
            0 => {
                ty.name = "Buffer".into();
                ty.generic_args.push(scalar_type("i64"));
            }
            1 => ty.is_ref = true,
            2 => ty.is_optional = true,
            _ => unreachable!(),
        }
        let layouts = control_values::TypedLayouts::collect(&module);
        let before = module.clone();
        normalize(helper(&mut module), &layouts);
        assert_eq!(module, before, "{kind}");
    }
}

#[test]
fn dead_record_snapshots_keep_all_callers_transactional() {
    for computed in [false, true] {
        let suffix = if computed { " + 0" } else { "" };
        let callers = format!(
            "
            fn first(state: State) -> i64 {{ return helper(state, state); }}
            fn second(state: State) -> i64 {{ return helper({}, state){suffix}; }}
        ",
            if computed { "relay(state)" } else { "state" }
        );
        let mut module = module_with_callers(&format!("let unused = {COPY}; return 0;"), &callers);
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
            assert!(helper(&mut module).params.is_empty());
            assert_eq!(helper(&mut module).body.len(), 1);
        }
    }
}

#[test]
fn dead_record_snapshots_bound_proof_work_without_partial_removal() {
    let mut module = module(&format!("let unused = {COPY}; let deep = state; return 0;"));
    let layouts = control_values::TypedLayouts::collect(&module);
    assert!(candidates(helper(&mut module), &layouts, 0).is_none());
    assert!(candidates(helper(&mut module), &layouts, 3).is_none());
    let NirStmt::Let { value, .. } = &mut helper(&mut module).body[1] else {
        panic!()
    };
    for _ in 0..64 {
        *value = NirExpr::FieldAccess {
            base: Box::new(value.clone()),
            field: "left".into(),
        };
    }
    let before = module.clone();
    normalize(helper(&mut module), &layouts);
    assert_eq!(module, before);
}
